## Variables
PACKAGE := bunner_cors_rs
TARGET_DIR := target
DOC_DIR := $(TARGET_DIR)/doc

CLIPPY_FLAGS_STRICT = --workspace --all-features -- -D warnings -D clippy::dbg_macro -D clippy::todo -D clippy::unimplemented -D clippy::panic -D clippy::print_stdout -D clippy::print_stderr
CLIPPY_FLAGS_RELAXED = --workspace --all-features --all-targets -- -D warnings -A clippy::panic -A clippy::print_stdout -A clippy::print_stderr

## Default / Meta targets
.PHONY: help all check

help: ## Show this help
	@grep -E '^[a-zA-Z0-9_\-]+:.*?## ' $(MAKEFILE_LIST) | sed -E 's/:.*?## /\t- /'

all: format lint test ## Format check, lint and run tests

## Code Quality
.PHONY: lint format

lint: ## Run strict + relaxed clippy (tests relaxed)
	cargo clippy $(CLIPPY_FLAGS_STRICT)
	cargo clippy $(CLIPPY_FLAGS_RELAXED)

format: ## Auto-format all code
	cargo fmt --all

## Testing
.PHONY: test

test: ## Run test suite (nextest if available, fallback to cargo test)
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		cargo nextest run --package $(PACKAGE); \
	else \
		cargo test --package $(PACKAGE); \
	fi

## Docs
.PHONY: doc doc-open

doc: ## Build documentation (including private items)
	cargo doc --all-features --no-deps --document-private-items

doc-open: doc ## Build docs then open in browser
	@xdg-open $(DOC_DIR)/$(PACKAGE)/index.html 2>/dev/null || true

## Security / Quality (optional tools: cargo-audit, cargo-deny)
.PHONY: audit
audit: ## Run security audit (requires cargo-audit)
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
	else \
		echo "cargo-audit not installed. Install with: cargo install cargo-audit" >&2; \
	fi

## Coverage
.PHONY: coverage
coverage: ## Generate coverage report (requires cargo-llvm-cov)
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		cargo llvm-cov; \
	else \
		echo "cargo-llvm-cov not installed. Install with: cargo install cargo-llvm-cov" >&2; \
	fi

## Release / Publish
.PHONY: release publish-dry-run
release: ## Build optimized release artifacts
	cargo build --release --all-features

## Cleanup
.PHONY: clean distclean
clean: ## Clean cargo artifacts
	cargo clean

distclean: clean ## Remove target directory entirely (alias)
	rm -rf $(TARGET_DIR)

