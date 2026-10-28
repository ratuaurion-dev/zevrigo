.PHONY: fmt fmt-check lint check-forbidden test test-slow deps check-all

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

check-forbidden:
	bash scripts/check-forbidden.sh

test:
	cargo test --workspace

test-slow:
	cargo test --workspace --release -- --ignored

deps:
	cargo tree --workspace --depth 1 -e normal

check-all: fmt-check lint check-forbidden test
