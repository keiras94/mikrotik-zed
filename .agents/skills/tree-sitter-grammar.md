# Skill: Tree-sitter Grammar for RSC

## Purpose

Maintain and extend the tree-sitter grammar for MikroTik RouterOS Script (RSC). Covers grammar rules, query files, testing, and generation.

## Grammar Location

- **Source:** `grammars/rsc/grammar.js` (244 lines)
- **Generated:** `grammars/rsc/src/parser.c`, `grammar.json`, `node-types.json`
- **Metadata:** `grammars/rsc/tree-sitter.json`
- **Tests:** `grammars/rsc/test/simple.rsc`, `grammars/rsc/test/example.rsc`

## Grammar Architecture

### Top-Level Structure

```
source_file
├── _statement (optional, first)
├── _terminated_statement (repeat)
│   ├── _statement_separator (; or \n)
│   └── _statement
│       ├── menu_command      — /path params
│       ├── global_command    — :cmd body params
│       ├── _value            — expression
│       ├── line_continuation — backslash
│       └── parent_navigation — ..
└── optional terminator (; or \n)
```

### Statement Types

| Node | Syntax | Example |
|------|--------|---------|
| `menu_command` | `/path key=value ...` | `/ip address add address=192.168.1.1/24 interface=ether1` |
| `global_command` | `:name body? params...` | `:if (condition) do={ ... }` |
| `named_param` | `key=value` | `name=ether1` |
| `block` | `{ ... }` | `do={ :put "hello" }` |
| `command_substitution` | `[cmd]` | `[find where name=ether1]` |
| `variable_reference` | `$name` | `$myVar` |
| `array` | `{ key=val; ... }` | `{1;2;3}` or `{a=1;b=2}` |
| `function_call` | `$func args` | `$execute script=backup` |

### Control Flow (via global_command)

```rsc
:if (condition) do={ ... }
:while (condition) do={ ... }
:foreach i in=[find] do={ ... }
:do { ... } while=(condition)
:for i from=0 to=10 step=1 do={ ... }
```

The grammar parses these as `global_command` with `_command_body` containing `do_block`, `else_block`, `while_condition`, or `for_in_clause`.

## Query Files

### highlights.scm

Captures syntax tokens for highlighting. Key patterns:

```scheme
; Menu paths — special string highlighting
(menu_prefix) @string.special.path
(menu_command (identifier) @string.special.path ...)

; Global commands — keyword coloring
(global_command_name) @keyword

; Control flow — distinct from other keywords
(global_command (global_command_name) @keyword.control
  (#match? @keyword.control ":(do|while|if|for|foreach|return|error)$"))

; Variable declarations
(global_command (global_command_name) @keyword.storage.type
  (#match? @keyword.storage.type ":(local|global|set)$"))

; Named parameters — property highlighting
(named_param name: (identifier) @property)
```

### brackets.scm

Matches pairs for bracket matching:

```scheme
("(" @opening ")" @closing)    ; Subexpressions
("[" @opening "]" @closing)    ; Command substitution
("{" @opening "}" @closing)    ; Blocks and arrays
("\"" @opening "\"" @closing)  ; Strings
```

### indents.scm

Auto-indent rules:

```scheme
; Indent after opening brace
(block "{" @indent.begin)
(block "}" @indent.end)

; Line continuation keeps indent
(line_continuation) @indent.continue

; else dedents
(else_block "else" @indent.dedent)
```

### outline.scm

Symbol view entries:

```scheme
; All menu commands appear in outline
(menu_command) @item

; Key global commands (:local, :global, :if, :for, etc.)
(global_command
  (global_command_name) @name
  (#match? @name ":(local|global|if|for|foreach|do|while)")
) @item
```

## Making Grammar Changes

### Step 1: Edit `grammar.js`

```bash
cd grammars/rsc
# Edit grammar.js
```

### Step 2: Regenerate

```bash
npx tree-sitter generate
# Or: npm run generate
```

### Step 3: Test

```bash
npx tree-sitter test
# Or: npm test
```

### Step 4: Update query files if needed

If you add new node types, update the corresponding `.scm` files in `languages/rsc/`.

### Step 5: Build WASM (for Zed)

```bash
npx tree-sitter build --wasm
```

## Common Patterns

### Adding a new keyword group

1. Add the keyword to `grammar.js` rules (or match existing `identifier`)
2. Add a `#match?` pattern in `highlights.scm`:

```scheme
(global_command
  (global_command_name) @keyword.type
  (#match? @keyword.type ":(new-keyword)$"))
```

### Adding a new node type

1. Define in `grammar.js` `rules: { ... }`
2. Reference from `_statement` or appropriate parent
3. Add highlighting in `highlights.scm`
4. Add bracket matching if it introduces delimiters
5. Add indentation if it introduces blocks
6. Add outline entry if it should appear in symbol view

## Testing Resources

- `grammars/rsc/test/simple.rsc` — Minimal: `/ip route print`
- `grammars/rsc/test/example.rsc` — Rich: variables, control flow, menus, arrays
- Online playground: `npm run playground` (starts local server)

## Debugging

```bash
# Parse a file and show the tree
npx tree-sitter parse test/example.rsc

# Highlight a file
npx tree-sitter highlight test/example.rsc

# Show node types
npx tree-sitter symbols test/example.rsc
```
