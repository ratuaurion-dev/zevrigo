.PHONY: fmt fmt-check lint check-forbidden test test-slow check-all

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

check-forbidden:
	@echo "Checking forbidden tokens (f32/f64/unsafe)..."
	@dirs=""; \
	for d in crates apps tests; do \
		if [ -d "$$d" ]; then dirs="$$dirs $$d"; fi; \
	done; \
	if [ -z "$$dirs" ]; then exit 0; fi; \
	files=$$(find $$dirs -type f -name '*.rs' | sort); \
	if [ -z "$$files" ]; then exit 0; fi; \
	bad=0; \
	if grep -Hn -wE 'f32|f64' $$files; then \
		echo "ERROR: ditemukan tipe floating point (f32/f64) di kode proyek!"; \
		bad=1; \
	fi; \
	for f in $$files; do \
		if sed 's|//.*||' "$$f" | grep -n -w 'unsafe' | sed "s|^|$$f:|" | grep .; then \
			echo "ERROR: ditemukan kata kunci unsafe di luar komentar pada $$f!"; \
			bad=1; \
		fi; \
	done; \
	exit $$bad

test:
	cargo test --workspace

test-slow:
	cargo test --workspace --release -- --ignored

check-all: fmt-check lint check-forbidden test
