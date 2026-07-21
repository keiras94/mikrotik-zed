.PHONY: help generate test test-grammar parse highlight extract build clean install-dev validate

# ── Tree-sitter grammar ────────────────────────────────────────

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

generate: ## Regenerate parser.c from grammar.js
	cd grammars/rsc && npx tree-sitter generate

test: test-grammar ## Alias for test-grammar

test-grammar: ## Run tree-sitter grammar tests (corpus tests)
	cd grammars/rsc && npx tree-sitter test

parse: ## Parse a file and show syntax tree (usage: make parse FILE=test/example.rsc)
	cd grammars/rsc && npx tree-sitter parse $(FILE)

highlight: ## Highlight a file (usage: make highlight FILE=test/example.rsc)
	cd grammars/rsc && npx tree-sitter highlight $(FILE)

# ── Command extraction ─────────────────────────────────────────

extract: ## Regenerate data/commands.toml from llms-full.txt
	python3 scripts/extract_commands.py

# ── Build (Phase 2) ────────────────────────────────────────────

build: ## Build language server WASM (Phase 2)
	cargo build --target wasm32-wasip1 --release

# ── Cleanup ────────────────────────────────────────────────────

clean: ## Remove build artifacts
	rm -rf target/
	cd grammars/rsc && rm -rf target/ build/

# ── Development ────────────────────────────────────────────────

install-dev: ## Point Zed to this directory (manual: Zed > Install Dev Extension)
	@echo "Open Zed → Command Palette → 'Install Dev Extension' → select this directory"

validate: generate test-grammar extract ## Full validation: regenerate + test + extract
	@echo "All checks passed."
