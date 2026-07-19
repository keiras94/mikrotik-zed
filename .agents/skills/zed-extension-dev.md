# Skill: Zed Extension Development

## Purpose

Guide for developing the MikroTik RouterOS Script extension for Zed. Covers both Phase 1 (tree-sitter grammar) and Phase 2 (language server).

## Project Identity

- **Extension ID:** `mikrotik-rsc`
- **Extension Name:** `MikroTik RouterOS Script`
- **Language:** RSC (RouterOS Script)
- **File suffix:** `.rsc`
- **Target RouterOS version:** 7.22+

## Critical Constraints

1. **Never clone or build `zed-industries/zed`**. Only depend on `zed_extension_api`.
2. **Never use `std::env::var` or `cfg` directives** in Rust code — they don't work under WASM. Use `zed_extension_api::current_platform` and `Worktree` methods.
3. **Extension `id` and `name` must NOT contain "zed" or "extension".**
4. **All Rust code must compile to `wasm32-wasip1`.**
5. **Do not bundle the LS binary** — download or build at install time.
6. **Valid OSS license mandatory** in repo root (Apache-2.0 already present).

## Repository Layout

```
mikrotik-zed/
├── extension.toml              # Zed extension manifest
├── Cargo.toml                  # (Phase 2) cdylib crate
├── src/lib.rs                  # (Phase 2) Extension trait + LS command
├── grammars/rsc/               # Tree-sitter grammar (in-tree)
│   ├── grammar.js              # Grammar rules
│   ├── src/parser.c            # Generated C parser
│   └── tree-sitter.json        # Tree-sitter metadata
├── languages/rsc/
│   ├── config.toml             # Language config
│   ├── highlights.scm          # Syntax highlighting
│   ├── brackets.scm            # Bracket matching
│   ├── indents.scm             # Indentation rules
│   ├── outline.scm             # Outline/symbol view
│   └── injections.scm          # Code injections (placeholder)
├── data/commands.toml          # Command table for LS
├── scripts/extract_commands.py # Extracts commands.toml
├── llms-full.txt               # RouterOS docs (truth source)
└── llms.txt                    # Doc index
```

## extension.toml Format

```toml
id = "mikrotik-rsc"
name = "MikroTik RouterOS Script"
description = "RSC language support for MikroTik RouterOS (7.22+)."
version = "0.1.0"
schema_version = 1
authors = ["Francisco"]
repository = "https://github.com/fravic/mikrotik-zed"

[grammars.rsc]
repository = "https://github.com/fravic/mikrotik-rsc-grammar"
rev = "<commit-hash>"

# Phase 2: Language server registration
[language_servers.rsc-ls]
name = "RSC Language Server"
languages = ["RSC"]
```

## Phase 1: Tree-sitter Grammar (Current)

### Query Files

| File | Purpose | Priority |
|------|---------|----------|
| `highlights.scm` | Syntax highlighting tokens | Highest |
| `brackets.scm` | Bracket matching pairs | High |
| `indents.scm` | Auto-indent rules | Medium |
| `outline.scm` | Outline/symbol view | Medium |
| `injections.scm` | Language injections | Low (placeholder) |

### highlights.scm Capture Types

```scheme
@comment            — Comments (# ...)
@keyword            — Global commands (:put, :error, etc.)
@keyword.control    — Control flow (:if, :while, :for, :foreach, :do, :else)
@keyword.storage.type — Variable declarations (:local, :global, :set)
@string             — Double-quoted strings
@string.special.path — Menu paths (/ip/firewall, identifiers in paths)
@number             — Numeric literals
@boolean            — true/false/yes/no
@constant           — IP addresses, IP prefixes
@constant.builtin   — nil
@variable           — $variable references
@punctuation.special — $ prefix, line continuation backslash
@punctuation.bracket — ( ) [ ] { }
@punctuation.delimiter — ; statement separator
@property           — Named parameter keys (key=value)
@operator           — + - * / % = != < > etc.
```

### Grammar Node Types

Key nodes in `grammar.js`:

- `source_file` — Top-level program
- `menu_command` — `/path param=value ...`
- `global_command` — `:command body? params...`
- `named_param` — `key=value` pairs
- `block` — `{ ... }` code blocks
- `command_substitution` — `[cmd]` inline execution
- `variable_reference` — `$name`
- `array` — `{ key=value; ... }`

## Phase 2: Language Server (Planned)

### Cargo.toml

```toml
[package]
name = "mikrotik-rsc-ls"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
zed_extension_api = "0.5"  # Check latest on crates.io
serde = { version = "1", features = ["derive"] }
toml = "0.8"
```

### Extension Trait Implementation

```rust
use zed_extension_api::{self as zed, Result};

struct MikrotikRscExtension;

impl zed::Extension for MikrotikRscExtension {
    fn new() -> Self { Self }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Locate or download the LS binary
        // Use worktree.path() — never std::env::var
        todo!()
    }
}

zed::register_extension!(MikrotikRscExtension);
```

### LS Capabilities to Implement

1. **`textDocument/completion`** — Look up `commands.toml` based on cursor menu path
2. **`textDocument/hover`** — Return property description from `commands.toml`
3. **No diagnostics** — RSC is a scripting language, no compile-time errors to report

## Testing

1. **Local dev:** Zed > "Install Dev Extension" > point to project root
2. **Logs:** `zed --foreground` or `zed: open log` action
3. **Grammar tests:** `cd grammars/rsc && tree-sitter test`
4. **Command extraction:** `python3 scripts/extract_commands.py`

## Publishing

1. Grammar repo (`fravic/mikrotik-rsc-grammar`) must be pushed with a stable rev
2. Update `extension.toml` grammar `rev` to the real commit hash
3. Submit PR to `zed-industries/extensions` with:
   - Submodule entry
   - `extensions.toml` entry
   - Run `pnpm sort-extensions`
