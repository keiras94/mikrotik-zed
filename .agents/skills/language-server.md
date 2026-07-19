# Skill: RSC Language Server (Phase 2)

## Purpose

Guide for implementing the Rust/WASM language server that provides autocompletion and hover documentation for MikroTik RouterOS Script files in Zed.

## Status

**Not yet started.** Phase 1 (tree-sitter grammar) is in progress. This skill documents the planned implementation.

## Architecture

```
Zed Editor
  └── Zed Extension API (zed_extension_api crate)
        └── MikrotikRscExtension (lib.rs)
              ├── language_server_command() — launches LS binary
              └── RSC Language Server (separate binary)
                    ├── textDocument/completion — looks up commands.toml
                    ├── textDocument/hover — returns property docs
                    └── data/commands.toml — embedded command table
```

## Implementation Checklist

### 1. Project Setup

- [ ] Create `Cargo.toml` at project root with `crate-type = ["cdylib"]`
- [ ] Add `zed_extension_api` dependency (check latest version)
- [ ] Add `serde`, `toml` dependencies for parsing `commands.toml`
- [ ] Set edition to `2024` and target to `wasm32-wasip1`

### 2. Extension Registration

- [ ] Create `src/lib.rs` with `MikrotikRscExtension` struct
- [ ] Implement `zed::Extension` trait
- [ ] Implement `language_server_command()` to locate/download LS binary
- [ ] Register with `zed::register_extension!(MikrotikRscExtension)`
- [ ] Add `[language_servers.rsc-ls]` to `extension.toml`

### 3. Command Table Parsing

- [ ] Define TOML structs matching `commands.toml` schema:
  ```rust
  #[derive(Deserialize)]
  struct CommandTable {
      menus: Vec<MenuEntry>,
  }

  #[derive(Deserialize)]
  struct MenuEntry {
      path: String,
      #[serde(rename = "type")]
      entry_type: String,
      #[serde(default)]
      flags: Vec<Flag>,
      #[serde(default)]
      arguments: Vec<Argument>,
      #[serde(default)]
      read_only: Vec<Argument>,
  }

  #[derive(Deserialize)]
  struct Argument {
      name: String,
      #[serde(rename = "type")]
      arg_type: String,
      #[serde(default)]
      description: String,
      #[serde(default)]
      required: bool,
  }
  ```
- [ ] Embed `commands.toml` at compile time with `include_str!` or `include_bytes!`
- [ ] Parse into `CommandTable` on extension startup

### 4. Menu Path Detection

- [ ] Given a cursor position (line, column), extract the current menu context
- [ ] Parse lines upward from cursor to find the most recent `/path` prefix
- [ ] Handle nested menus: `/ip firewall filter` → `/ip/firewall/filter`
- [ ] Handle command substitution context: inside `[...]` use parent context

### 5. Completion Provider

- [ ] Register for `textDocument/completion`
- [ ] At cursor position, determine completion kind:
  - After `/` → menu path segments
  - After menu path → argument names
  - After `key=` → argument values (enum options, type hints)
- [ ] Return `CompletionList` with:
  - Menu paths as `Folder` kind
  - Argument names as `Property` kind
  - Enum values as `Enum` kind
  - Include `documentation` from `commands.toml` descriptions

### 6. Hover Provider

- [ ] Register for `textDocument/hover`
- [ ] On hover over argument name, return:
  - Property name and type
  - Description from `commands.toml`
  - Whether it's required
  - Whether it can be unset
- [ ] On hover over menu path, return:
  - Menu type (Directory/Command)
  - Available flags
  - Summary of arguments

### 7. No Diagnostics

RSC is a runtime scripting language — there are no compile-time errors to report. The LS should NOT implement `textDocument/diagnostic` or `textDocument/publishDiagnostics`.

## Key Design Decisions

### Why a separate binary, not inline?

The Zed extension API expects a language server binary. The extension code (`lib.rs`) is the bridge that launches it. The LS binary could be:
- Bundled in the extension (large download)
- Downloaded on first use (recommended)
- Built from source (complex)

### Why static command table, not dynamic?

- RouterOS CLI is stable within a major version
- `commands.toml` is ~10KB compressed — trivial to embed
- No network dependency at runtime
- Deterministic behavior

### Why no diagnostics?

- RSC is interpreted, not compiled
- Errors are runtime (on the router)
- False positives would be worse than no diagnostics
- Focus on completion and hover only

## WASM Compilation

```bash
# Build for Zed
cargo build --target wasm32-wasip1 --release

# The output .wasm file is what Zed loads
```

## Testing

1. **Unit tests:** Parse `commands.toml`, test menu path detection
2. **Integration:** Install dev extension in Zed, test completion in `.rsc` file
3. **Edge cases:** Empty files, nested command substitution, multiple menu paths

## Reference

- Zed extension API: https://docs.rs/crate/zed_extension_api/latest
- Zed extension docs: https://zed.dev/docs/extensions/developing-extensions
- Zed language server docs: https://zed.dev/docs/extensions/languages
