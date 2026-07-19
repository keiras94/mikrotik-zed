# Skill: Development Workflow

## Purpose

Quick reference for day-to-day development commands. For deep dives, see the other skills.

## Prerequisites

```bash
# Tree-sitter CLI (needed for grammar work)
npm install -g tree-sitter-cli

# Python 3 (needed for command extraction and test generation)
python3 --version
```

## Makefile Commands

All commands are in the Makefile. Run `make help` for the list.

| Command | What it does |
|---------|-------------|
| `make generate` | Regenerate `parser.c` after editing `grammar.js` |
| `make test` | Run tree-sitter corpus tests |
| `make parse FILE=test/example.rsc` | Show parse tree for a file |
| `make highlight FILE=test/example.rsc` | Show highlight captures for a file |
| `make extract` | Regenerate `data/commands.toml` from `llms-full.txt` |
| `make build` | Build LS WASM (Phase 2, needs Rust + wasm32-wasip1) |
| `make clean` | Remove build artifacts |
| `make validate` | Full pipeline: generate → test → extract |

## Corpus Test Management

The grammar uses tree-sitter's corpus test format in `grammars/rsc/test/corpus/`. Three scripts help manage them:

| Script | Purpose |
|--------|---------|
| `scripts/generate_tests.py` | Generate corpus from inline test definitions (outputs `grammars/rsc/test/corpus/*.txt`) |
| `scripts/clean_tests.py` | Remove test cases that produce ERROR/MISSING nodes |
| `scripts/regenerate_tests.py` | Regenerate expected parse trees for all corpus tests |

```bash
# After editing grammar.js, update expected outputs:
make generate        # Regenerate parser.c
make update-tests    # Regenerate expected trees in corpus
make test            # Verify all pass

# If some tests fail due to grammar changes:
make clean-tests     # Remove failing tests
make test            # Verify remaining pass
```

### Corpus file format

```
==========
Test case name
==========

input code here

---

(expected parse tree here)
```

## Common Workflows

### Editing the grammar

```bash
# 1. Edit grammar.js
vim grammars/rsc/grammar.js

# 2. Regenerate parser
make generate

# 3. Update expected test outputs
make update-tests

# 4. Run tests
make test

# 5. Test specific file
make parse FILE=test/example.rsc

# 6. If test files fail, investigate:
#    - Run with --debug to see parsing log
#    - Check if grammar rule needs updating
```

### Regenerating commands.toml

```bash
# 1. Optionally update llms-full.txt from upstream
curl -o llms-full.txt https://manual.mikrotik.com/llms-full.txt

# 2. Regenerate
make extract

# 3. Verify
rg -c '^\[\[menus\]\]' data/commands.toml
rg 'path = "/ip/firewall/filter"' data/commands.toml
```

### Testing in Zed

```bash
# 1. Open Zed
open -a Zed

# 2. Command Palette → "Install Dev Extension"
# 3. Select the mikrotik-zed/ directory
# 4. Open a .rsc file to test

# Check logs if something goes wrong:
# Command Palette → "zed: open log"
# Or from terminal:
zed --foreground
```

### Full validation before commit

```bash
make validate
git add -A
git commit -m "..."
```

## Troubleshooting

### `npx tree-sitter generate` fails

- Check `grammars/rsc/grammar.js` for syntax errors
- Ensure `tree-sitter-cli` is installed: `npx tree-sitter --version`

### `tree-sitter test` shows parse failures

- Run `make parse FILE=<failing-file>.rsc` to see the tree
- Compare expected vs actual node types
- Run `make update-tests` to regenerate expected output
- Check if grammar rule needs updating

### `extract_commands.py` finds 0 entries

- Verify `llms-full.txt` exists in project root
- Check that target menus haven't changed format
- Run with debug: add `print(line)` in the parser loop

### Zed doesn't show highlighting

- Check `extension.toml` grammar `rev` is not `0000...`
- Verify `languages/rsc/config.toml` exists and is correct
- Check Zed logs for extension loading errors
- Try "Install Dev Extension" again (sometimes needs refresh)

## File Locations Quick Reference

| What | Where |
|------|-------|
| Grammar rules | `grammars/rsc/grammar.js` |
| Generated parser | `grammars/rsc/src/parser.c` |
| Syntax highlighting | `languages/rsc/highlights.scm` |
| Bracket matching | `languages/rsc/brackets.scm` |
| Indentation | `languages/rsc/indents.scm` |
| Outline/symbols | `languages/rsc/outline.scm` |
| Language config | `languages/rsc/config.toml` |
| Command table | `data/commands.toml` |
| Extraction script | `scripts/extract_commands.py` |
| Test corpus | `grammars/rsc/test/corpus/*.txt` |
| Test generators | `scripts/generate_tests.py`, `clean_tests.py`, `regenerate_tests.py` |
| RouterOS docs | `llms-full.txt` |
| Doc index | `llms.txt` |
| Extension manifest | `extension.toml` |
