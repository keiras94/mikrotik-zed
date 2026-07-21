.PHONY: help generate test test-grammar test-rust test-all parse highlight extract build check clean install-dev validate

# ── Tree-sitter grammar ────────────────────────────────────────

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

generate: ## Regenerate parser.c from grammar.js
	cd grammars/rsc && npx tree-sitter generate

test: test-grammar ## Alias for test-grammar

test-grammar: ## Run tree-sitter grammar tests (corpus tests)
	cd grammars/rsc && npx tree-sitter test

test-rust: ## Run Rust unit tests
	cargo test --lib

test-python: ## Run Python extraction tests
	python3 -m pytest tests/ -v

test-all: test-grammar test-rust test-python ## Run all tests

parse: ## Parse a file (usage: make parse FILE=grammars/rsc/test/example.rsc)
	cd grammars/rsc && npx tree-sitter parse ../..$(FILE)

highlight: ## Highlight a file (usage: make highlight FILE=grammars/rsc/test/example.rsc)
	cd grammars/rsc && npx tree-sitter highlight ../..$(FILE)

# ── Command extraction ─────────────────────────────────────────

extract: ## Regenerate data/commands.toml from llms-full.txt
	python3 scripts/extract_commands.py

# ── Build (Phase 2) ────────────────────────────────────────────

build: ## Build language server WASM (Phase 2)
	cargo build --target wasm32-wasip1 --release
	cp target/wasm32-wasip1/release/mikrotik_zed.wasm extension.wasm

check: ## Quick compile verification (no output)
	cargo check --target wasm32-wasip1

# ── Cleanup ────────────────────────────────────────────────────

clean: ## Remove all build artifacts and generated files
	rm -rf target/
	rm -f extension.wasm
	rm -f Cargo.lock
	cd grammars/rsc && rm -rf target/ build/ src/grammar.json src/node-types.json src/parser.c

# ── Development ────────────────────────────────────────────────

install-dev: ## Point Zed to this directory (manual: Zed > Install Dev Extension)
	@echo "Open Zed → Command Palette → 'Install Dev Extension' → select this directory"

validate: generate test-all extract ## Full validation: regenerate + all tests + extract
	@echo "All checks passed."
