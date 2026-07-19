// ── MikroTik RouterOS Script Language Server ─────────────────────────
// LSP over stdio, provides completions and hover for /ip, /ipv6,
// /interface, and /routing menus.  Runs as a Node.js script launched by
// the Zed WASM extension.
//
// Usage: node ls.mjs <path-to-commands.toml>
//
// ── LSP handlers ─────────────────────────────────────────────────────
//   textDocument/completion  – menu path, command verb, and property
//   textDocument/hover        – description for commands and properties

import { readFileSync } from "node:fs";

// ── TOML parsing (minimal, covers our subset only) ───────────────────

function parseToml(text) {
  const menus = [];
  let currentMenu = null;
  // "menu" | "arg" | "flag" | "child"
  let section = null;

  for (const rawLine of text.split("\n")) {
    const line = rawLine.trim();

    if (line.startsWith("#") || line === "") continue;

    // Section headers
    if (line.startsWith("[[menus.children]]")) {
      currentMenu.children.push({ name: "" });
      section = "child";
      continue;
    }
    if (line.startsWith("[[menus.flags]]")) {
      currentMenu.flags.push({ name: "", description: "" });
      section = "flag";
      continue;
    }
    if (line.startsWith("[[menus.arguments]]")) {
      currentMenu.arguments.push({ name: "", type: "" });
      section = "arg";
      continue;
    }
    if (line.startsWith("[[menus]]")) {
      currentMenu = { path: "", type: "", arguments: [], flags: [], children: [] };
      menus.push(currentMenu);
      section = "menu";
      continue;
    }

    if (!currentMenu) continue;

    // Key = "value" lines
    const kv = line.match(/^(?<key>\w+)\s*=\s*"(?<value>.*)"$/);
    if (!kv) continue;

    const { key, value } = kv.groups;

    if (section === "menu") {
      if (key === "path") currentMenu.path = value;
      if (key === "type") currentMenu.type = value;
    } else if (section === "arg") {
      const last = currentMenu.arguments[currentMenu.arguments.length - 1];
      if (last) {
        if (key === "name") last.name = value;
        if (key === "type") last.type = value;
      }
    } else if (section === "flag") {
      const last = currentMenu.flags[currentMenu.flags.length - 1];
      if (last) {
        if (key === "name") last.name = value;
        if (key === "description") last.description = value;
      }
    } else if (section === "child") {
      const last = currentMenu.children[currentMenu.children.length - 1];
      if (last && key === "name") last.name = value;
    }
  }

  return menus;
}

// ── Data loading ─────────────────────────────────────────────────────

const commandsPath = process.argv[2];
if (!commandsPath) {
  console.error("Usage: node ls.mjs <path-to-commands.toml>");
  process.exit(1);
}

const raw = readFileSync(commandsPath, "utf-8");
const allMenus = parseToml(raw);
// Data loaded — menus index built below

// Build lookups
const menuByPath = new Map();
for (const m of allMenus) {
  menuByPath.set(m.path, m);
}

/**
 * Build parent→children index by scanning ALL paths and extracting the
 * *immediate* next segment from each.  This catches intermediate menus
 * (e.g. /ip/firewall) even when there is no explicit menu entry for them.
 *
 * For every menu path like /ip/firewall/filter, it records:
 *   parent = /ip/firewall → child "filter"
 *   parent = /ip          → child "firewall"
 *
 * The index is used for sub-menu completion suggestions.
 */
const childNamesByParent = new Map();
for (const m of allMenus) {
  const parts = m.path.split("/"); // e.g. ["", "ip", "firewall", "filter"]

  // Build intermediate parent→child relationships for every prefix
  for (let i = 2; i < parts.length; i++) {
    // parts[1..i] is the parent path (e.g. "/ip" or "/ip/firewall")
    const parentPath = "/" + parts.slice(1, i).join("/");
    const childName = parts[i]; // the next segment

    if (!childNamesByParent.has(parentPath)) {
      childNamesByParent.set(parentPath, new Map());
    }
    const children = childNamesByParent.get(parentPath);

    // Keep the first type we see for a child (prefer Directory over Command)
    if (!children.has(childName)) {
      children.set(childName, { name: childName, path: "/" + parts.slice(1, i + 1).join("/"), type: m.type });
    } else if (m.type === "Directory" || m.type === "Settings Directory") {
      // Upgrade to Directory type if we later see it
      children.set(childName, { name: childName, path: "/" + parts.slice(1, i + 1).join("/"), type: m.type });
    }
  }
}

// Build root→child index from the first segment of every path.
// This ensures root menus like /ip, /interface, /routing are
// available even when they don't have their own TOML entry.
const rootEntries = new Map();
for (const m of allMenus) {
  const parts = m.path.split("/"); // ["", "ip", "address"]
  if (parts.length >= 2) {
    const rootName = parts[1];
    if (!rootEntries.has(rootName)) {
      rootEntries.set(rootName, {
        name: rootName,
        path: "/" + rootName,
        type: "Directory",
      });
    }
  }
}
childNamesByParent.set("", [...rootEntries.values()]);

// Convert Map-of-Maps to Map-of-arrays for easier consumption
for (const [parent, children] of childNamesByParent) {
  childNamesByParent.set(parent, [...children.values()]);
}

// Standard RouterOS verbs available on most Directory-type menus
const STANDARD_VERBS = [
  "add", "remove", "set", "get", "print", "enable", "disable",
  "find", "comment", "move", "export", "import", "edit",
];

// ── LSP primitives ───────────────────────────────────────────────────

function sendMessage(msg) {
  const json = JSON.stringify(msg);
  const encoder = new TextEncoder();
  const header = `Content-Length: ${encoder.encode(json).length}\r\n\r\n`;
  process.stdout.write(header + json);
}

function log(...args) {
  // Stderr is for logging, stdout is for LSP
  process.stderr.write("[rsc-ls] " + args.join(" ") + "\n");
}

// ── Completion logic ─────────────────────────────────────────────────

const QUOTED_TOKEN_RE = /"(?:[^"\\]|\\.)*"/;
const LINE_TOKEN_RE = /(?:"(?:[^"\\]|\\.)*"|[/]\S*|\S+)/g;

function tokenize(text) {
  const tokens = [];
  let match;
  while ((match = LINE_TOKEN_RE.exec(text)) !== null) {
    tokens.push(match[0]);
  }
  return tokens;
}

/**
 * Build the "before cursor" context spanning multiple lines.
 *
 * RouterOS commands can span multiple lines – properties on subsequent
 * lines are continuations of the same command.  This function walks
 * backwards from the cursor line, collecting all lines that belong to
 * the current command, stopping when it hits:
 *   1. A blank/empty line (command boundary)
 *   2. A line starting with `/` or `:` (new command start)
 *
 * Example:
 *   /ip address add address=10.0.0.1
 *       network=10.0.0.0         ← cursor at end
 *       comment="test"
 *   /interface print              ← stop here
 *
 * Returns a single string suitable for parseLine().
 */
function buildBeforeCursor(doc, lineIndex, character) {
  const lines = doc.split("\n");
  const parts = [];

  // Current line (up to cursor position)
  const currentPart = lines[lineIndex].slice(0, character);

  // Cursor on a blank/empty line → no active command
  if (currentPart.trim() === "") return "";

  parts.push(currentPart);

  // Walk backwards collecting continuation lines
  for (let i = lineIndex - 1; i >= 0; i--) {
    const line = lines[i];
    const trimmed = line.trim();

    // Empty line → previous command boundary
    if (trimmed === "") break;

    // Line starting with `/` or `:` → this IS the command start; include it, then stop
    if (trimmed.startsWith("/") || trimmed.startsWith(":")) {
      parts.unshift(line);
      break;
    }

    // Otherwise it's a continuation line → include and keep going
    parts.unshift(line);
  }

  return parts.join(" ").trim();
}

/**
 * Parse a line of RouterOS script into its structural components.
 *
 * Example inputs:
 *   "/ip address add address=10.0.0.1/24 interface=ether1"
 *   "/ip firewall filter add chain=input"
 *   "/interface bridge port print"
 */
function parseLine(beforeCursor) {
  const tokens = tokenize(beforeCursor);
  const pathParts = [];
  let command = null;
  const properties = new Map(); // name → value (or empty if just key=)
  let partialKey = null; // key being typed before the =

  // Helper: check if a token is a known sub-menu at current path
  const currentPath = () => (pathParts.length === 0 ? "" : "/" + pathParts.join("/"));
  const isKnownSubMenu = (name) => {
    const candidate = currentPath() + "/" + name;
    return menuByPath.has(candidate);
  };

  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i];

    // Token starting with / is always a path segment
    if (token.startsWith("/")) {
      pathParts.push(token.replace(/^\//, ""));
      continue;
    }

    // Token containing = is a property assignment
    const eqIdx = token.indexOf("=");
    if (eqIdx >= 0) {
      const key = token.slice(0, eqIdx);
      const value = token.slice(eqIdx + 1);
      properties.set(key, value);
      continue;
    }

    // If we have a path segment, check if this token is a known sub-menu or a command
    if (pathParts.length > 0) {
      if (isKnownSubMenu(token)) {
        pathParts.push(token);
      } else {
        command = token;
      }
      continue;
    }

    // No path yet – could be a global command, variable, or just text
    command = token;
  }

  return {
    path: currentPath(),
    command,
    properties,
    lastToken: tokens.length > 0 ? tokens[tokens.length - 1] : "",
  };
}

function getInsertText(arg) {
  // For enum types, insert with = and placeholder
  if (arg.type && arg.type.startsWith("enum")) {
    return `${arg.name}=$1`;
  }
  if (arg.type === "bool") {
    return `${arg.name}=$1`;
  }
  if (arg.type === "string") {
    return `${arg.name}="$1"`;
  }
  return `${arg.name}=$1`;
}

/**
 * Parse enum values from an argument type string.
 * "enum (value1 | value2)" → ["value1", "value2"]
 * "enum (accept | drop | jump)" → ["accept", "drop", "jump"]
 */
function parseEnumValues(typeStr) {
  if (!typeStr || !typeStr.startsWith("enum")) return [];
  const match = typeStr.match(/^enum\s*\((.*)\)/);
  if (!match) return [];
  return match[1].split("|").map((s) => s.trim()).filter(Boolean);
}

/**
 * Return value completions for a property whose = was just typed.
 * This checks the current menu context for the argument definition and
 * returns enum choices, bool options, or a type placeholder.
 */
function getValueCompletions(context, propertyKey) {
  const menu = menuByPath.get(context.path);
  if (!menu) return [];

  const arg = menu.arguments.find((a) => a.name === propertyKey);
  if (!arg || !arg.type) return [];

  const items = [];

  // Enum values → suggest each as a completion
  const enumValues = parseEnumValues(arg.type);
  for (const val of enumValues) {
    items.push({
      label: val,
      kind: 12, // EnumMember
      detail: `enum value — ${arg.type}`,
      insertText: val,
      insertTextFormat: 1,
    });
  }

  // Boolean → yes / no / true / false
  if (arg.type === "bool" || arg.type === "boolean") {
    for (const val of ["yes", "no", "true", "false"]) {
      items.push({
        label: val,
        kind: 12,
        detail: "bool value",
        insertText: val,
        insertTextFormat: 1,
      });
    }
  }

  // Interface references
  if (arg.type.startsWith("iface_enum")) {
    items.push({
      label: "ether1",
      kind: 12,
      detail: "common interface name",
      insertText: "ether1",
      insertTextFormat: 1,
    });
    items.push({
      label: "bridge",
      kind: 12,
      detail: "common interface name",
      insertText: "bridge",
      insertTextFormat: 1,
    });
  }

  // For ipAddr / ipPrefix, show type hint as a completion
  if (arg.type.startsWith("ipAddr") || arg.type.startsWith("ipPrefix") || arg.type === "address") {
    items.push({
      label: "0.0.0.0/0",
      kind: 12,
      detail: `type: ${arg.type}`,
      insertText: "0.0.0.0/0",
      insertTextFormat: 1,
    });
  }

  return items;
}

function getDetail(arg) {
  if (!arg.type) return "property";
  return `type: ${arg.type}`;
}

function getArgCompletionItems(context, cursorBeforeLastToken) {
  const menu = menuByPath.get(context.path);
  if (!menu) return [];

  const used = new Set(context.properties.keys());

  const items = [];
  for (const arg of menu.arguments) {
    if (used.has(arg.name)) continue;
    const insertText = getInsertText(arg);
    items.push({
      label: arg.name,
      kind: 5, // Property
      detail: getDetail(arg),
      insertText,
      insertTextFormat: 2, // snippet
      data: { source: "rsc-arg", path: context.path, arg: arg.name },
    });
  }

  for (const flag of menu.flags) {
    items.push({
      label: flag.name,
      kind: 14, // Constant
      detail: `${flag.name}: ${flag.description}`,
      insertText: flag.name,
      insertTextFormat: 1,
    });
  }

  return items;
}

function getVerbCompletionItems(context) {
  const items = [];

  // Standard RouterOS verbs
  for (const verb of STANDARD_VERBS) {
    items.push({
      label: verb,
      kind: 3, // Function (command)
      detail: `${verb} — standard command`,
      insertText: verb,
      insertTextFormat: 1,
      data: { source: "rsc-verb", path: context.path, verb },
    });
  }

  // Action commands (type = "Command" entries under this path)
  const childCommands = childNamesByParent.get(context.path) || [];
  for (const child of childCommands) {
    if (child.type === "Command") {
      items.push({
        label: child.name,
        kind: 3, // Function
        detail: `action command`,
        insertText: child.name,
        insertTextFormat: 1,
        data: { source: "rsc-action", path: child.path },
      });
    }
  }

  return items;
}

function getSubMenuCompletionItems(context) {
  const children = childNamesByParent.get(context.path) || [];
  const items = [];

  for (const child of children) {
    if (child.type === "Directory" || child.type === "Settings Directory") {
      items.push({
        label: child.name,
        kind: 9, // Class (sub-menu)
        detail: `sub-menu — ${child.path}`,
        insertText: child.name,
        insertTextFormat: 1,
        data: { source: "rsc-submenu", path: child.path },
      });
    }
  }

  return items;
}

function getRootCompletionItems() {
  const roots = childNamesByParent.get("");
  if (!roots) return [];
  return roots.map((r) => ({
    label: r.path,
    kind: 9, // Class
    detail: `root menu — ${r.path}`,
    insertText: r.path,
    insertTextFormat: 1,
    data: { source: "rsc-root", path: r.path },
  }));
}

/**
 * Return ALL possible completions for the current cursor context.
 *
 * The strategy is to always return everything that could match —
 * sub-menus, standard verbs, and command arguments — and let Zed's
 * built-in fuzzy filtering narrow it down based on what the user has
 * typed so far.  This makes the LS feel "predictive": as you type
 * `/ip a` it will suggest both "address" (sub-menu) and "add" (verb),
 * because the path is known and the `a` prefix matches both.
 *
 * The only exception is when the cursor sits right after `property=`,
 * where we switch to value suggestions (enum values, bool choices, …).
 */
function computeCompletions(beforeCursor) {
  const context = parseLine(beforeCursor);

  // No path yet (or just a bare "/") → suggest root menus
  if (!context.path || context.path === "" || context.path === "/") {
    return getRootCompletionItems();
  }

  // Typing a property value right after = → suggest enum/bool/type values.
  // Only trigger when the = is the last character (user just typed name=);
  // if there's already a value (name=value), fall through to general completions.
  const lastEq = context.lastToken.indexOf("=");
  if (lastEq >= 0 && lastEq === context.lastToken.length - 1) {
    const key = context.lastToken.slice(0, lastEq);
    return getValueCompletions(context, key);
  }

  // Everything else: gather ALL candidate types and let Zed filter.
  const items = [];

  // 1. Sub-menus reachable from the current path
  items.push(...getSubMenuCompletionItems(context));

  // 2. Standard RouterOS verbs (add, remove, set, print, …)
  items.push(...getVerbCompletionItems(context));

  // 3. Command arguments (properties + flags) for the current menu
  items.push(...getArgCompletionItems(context, beforeCursor));

  return items;
}

// ── Hover logic ──────────────────────────────────────────────────────

/**
 * @param {object} doc  – object with lineAt(n) → {text: string}
 * @param {object} pos  – {line, character}
 * @param {string} rawDoc – raw document text (for multi-line context)
 */
function computeHover(doc, position, rawDoc) {
  const line = doc.lineAt(position.line).text;
  const wordStart = findWordStart(line, position.character);
  const wordEnd = findWordEnd(line, position.character);
  const word = line.slice(wordStart, wordEnd);
  if (!word) return null;

  // Check if the word is a menu path (with leading /)
  if (word.startsWith("/")) {
    const menu = menuByPath.get(word);
    if (menu) {
      const args = menu.arguments.map((a) => `  ${a.name}: ${a.type || "(any)"}`).join("\n");
      const flags = menu.flags.map((f) => `  ${f.name} — ${f.description || ""}`).join("\n");
      let md = `### ${word}\n\n**Type:** ${menu.type || "Directory"}`;
      if (args) md += `\n\n**Arguments:**\n${args}`;
      if (flags) md += `\n\n**Flags:**\n${flags}`;
      return { contents: { kind: "markdown", value: md } };
    }
  }

  // Check if it's a property name – scan the current menu (multi-line aware)
  const beforeCursor = buildBeforeCursor(rawDoc, position.line, position.character);
  const context = parseLine(beforeCursor);
  const menu = menuByPath.get(context.path);
  if (menu) {
    const arg = menu.arguments.find((a) => a.name === word);
    if (arg) {
      let md = `**${arg.name}**\n\nType: \`${arg.type || "any"}\``;
      return { contents: { kind: "markdown", value: md } };
    }
    const flag = menu.flags.find((f) => f.name === word);
    if (flag) {
      let md = `**${flag.name}**\n\n${flag.description || ""}`;
      return { contents: { kind: "markdown", value: md } };
    }
  }

  // Check if it's a standard verb
  if (STANDARD_VERBS.includes(word) || word === "add" || word === "remove" || word === "set") {
    return {
      contents: { kind: "markdown", value: `**${word}**\n\nStandard RouterOS command.` },
    };
  }

  return null;
}

function findWordStart(line, pos) {
  let i = pos;
  while (i > 0 && /\w/.test(line[i - 1])) i--;
  return i;
}

function findWordEnd(line, pos) {
  let i = pos;
  while (i < line.length && /\w/.test(line[i])) i++;
  return i;
}

// ── LSP message handling ─────────────────────────────────────────────

let messageBuffer = "";
let contentLength = 0;
let requestId = 0;
let docs = new Map(); // URI → document text

function handleMessage(msg) {
  if (msg.method === "initialize") {
    requestId = msg.id;
    return {
      id: msg.id,
      result: {
        capabilities: {
          textDocumentSync: 1, // full
          completionProvider: {
            triggerCharacters: ["/", " ", "="],
          },
          hoverProvider: true,
        },
        serverInfo: {
          name: "mikrotik-rsc-ls",
          version: "0.1.0",
        },
      },
    };
  }

  if (msg.method === "shutdown") {
    return { id: msg.id, result: null };
  }

  if (msg.method === "exit") {
    process.exit(0);
  }

  if (msg.method === "textDocument/didOpen") {
    const uri = msg.params.textDocument.uri;
    docs.set(uri, msg.params.textDocument.text);
    return null;
  }

  if (msg.method === "textDocument/didChange") {
    const uri = msg.params.textDocument.uri;
    docs.set(uri, msg.params.contentChanges[0].text);
    return null;
  }

  if (msg.method === "textDocument/didClose") {
    docs.delete(msg.params.textDocument.uri);
    return null;
  }

  if (msg.method === "textDocument/completion") {
    const uri = msg.params.textDocument.uri;
    const pos = msg.params.position;
    const doc = docs.get(uri);
    if (!doc) return { id: msg.id, result: { items: [] } };

    // Walk backwards from cursor to collect the full multi-line command
    // (properties on subsequent lines belong to the same command).
    const beforeCursor = buildBeforeCursor(doc, pos.line, pos.character);
    const items = computeCompletions(beforeCursor);

    return {
      id: msg.id,
      result: {
        isIncomplete: false,
        items,
      },
    };
  }

  if (msg.method === "textDocument/hover") {
    const uri = msg.params.textDocument.uri;
    const pos = msg.params.position;
    const doc = docs.get(uri);
    if (!doc) return { id: msg.id, result: null };

    // Build simple document with line access
    const lines = doc.split("\n");
    const docObj = {
      lineAt: (n) => ({ text: lines[n] || "" }),
    };

    const hover = computeHover(docObj, pos, doc);
    return { id: msg.id, result: hover };
  }

  // Unknown method
  return null;
}

// ── Input processing ─────────────────────────────────────────────────

const decoder = new TextDecoder();

process.stdin.on("data", (chunk) => {
  messageBuffer += decoder.decode(chunk, { stream: true });

  while (true) {
    // Parse header
    if (!contentLength) {
      const headerEnd = messageBuffer.indexOf("\r\n\r\n");
      if (headerEnd === -1) return;
      const header = messageBuffer.slice(0, headerEnd);
      const match = header.match(/Content-Length:\s*(\d+)/);
      if (!match) {
        messageBuffer = messageBuffer.slice(headerEnd + 4);
        continue;
      }
      contentLength = parseInt(match[1], 10);
      messageBuffer = messageBuffer.slice(headerEnd + 4);
    }

    // Wait for full body
    if (messageBuffer.length < contentLength) return;

    const body = messageBuffer.slice(0, contentLength);
    messageBuffer = messageBuffer.slice(contentLength);
    contentLength = 0;

    // Parse and respond
    try {
      const msg = JSON.parse(body);
      log("←", msg.method || msg.id);
      const response = handleMessage(msg);
      if (response) sendMessage(response);
    } catch (err) {
      log("parse error:", err.message);
    }
  }
});

log("language server started, pid=" + process.pid);
