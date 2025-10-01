## Variables
PACKAGE := bunner_cors_rs
TARGET_DIR := target
DOC_DIR := $(TARGET_DIR)/doc

## Default / Meta targets
.PHONY: all

all: format lint test

## Code Quality
.PHONY: lint format

lint:
	cargo clippy --workspace --all-features --lib --bins -- -D warnings -D clippy::dbg_macro -D clippy::todo -D clippy::unimplemented -D clippy::panic -D clippy::print_stdout -D clippy::print_stderr
	cargo clippy --workspace --all-features --tests --examples --benches -- -D warnings -A dead_code -A clippy::panic -A clippy::print_stdout -A clippy::print_stderr

format:
	cargo fmt --all

## Testing
.PHONY: test

test:
	INSTA_UPDATE=always RUSTFLAGS="-A dead_code" cargo nextest run --package $(PACKAGE); \

## Docs
.PHONY: doc doc-open

doc:
	cargo doc --all-features --no-deps --document-private-items

doc-open: doc
	@xdg-open $(DOC_DIR)/$(PACKAGE)/index.html 2>/dev/null || true

## Security / Quality (optional tools: cargo-audit, cargo-deny)
.PHONY: audit
audit:
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
	else \
		echo "cargo-audit not installed. Install with: cargo install cargo-audit" >&2; \
	fi

## Coverage
.PHONY: coverage
coverage:
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		cargo llvm-cov --ignore-filename-regex '($(CURDIR)/tests/.*|$(CURDIR)/src/.*_test\.rs$$)'; \
	else \
		echo "cargo-llvm-cov not installed. Install with: cargo install cargo-llvm-cov" >&2; \
	fi

## Release / Publish
.PHONY: release
release:
	cargo build --release --all-features

## Cleanup
.PHONY: clean distclean
clean:
	cargo clean

distclean: clean
	rm -rf $(TARGET_DIR)

