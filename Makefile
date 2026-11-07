.PHONY: fmt fmt-check lint check-forbidden test test-slow deps check-all web-check wallet-check docker-build

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

# Antarmuka web explorer (DEC-26): paket dari package-lock.json tanpa skrip pasang, lalu typecheck, test, dan
# next build (static export ke apps/explorer/web/out).
web-check:
	cd apps/explorer/web && npm ci --ignore-scripts --no-audit --no-fund && npm run check

# Antarmuka wallet desktop (MOD-15, DEC-31): paket dari package-lock.json tanpa skrip pasang, lalu typecheck, test, dan
# vite build (apps/wallet/dist, dipakai Tauri untuk aplikasi rilis). Crate Rust wallet ikut check-all.
wallet-check:
	cd apps/wallet && npm ci --ignore-scripts --no-audit --no-fund && npm run check

# Paket Docker regtest (TSK-M33): image zevrigo:regtest berisi node, explorer, miner, CLI, dan antarmuka web.
docker-build:
	docker build -t zevrigo:regtest .
