.PHONY: help install dev down build check test fmt lint clean

help:
	@echo "make install  - install landing/ frontend deps"
	@echo "make dev      - run the desktop app in dev mode (hot-reloading UI)"
	@echo "make down     - stop a dev session left running in the background"
	@echo "make build    - production build (Rust + bundled UI)"
	@echo "make check    - cargo check the Rust core"
	@echo "make test     - cargo test the Rust core"
	@echo "make fmt      - cargo fmt the Rust core"
	@echo "make lint     - cargo clippy, warnings as errors"
	@echo "make clean    - remove build artifacts (target/, dist/, gen/)"

install:
	npm install --prefix landing

# Run from the repo root, not landing/: the Tauri CLI only scans downward from
# its cwd for src-tauri/, and landing/ + src-tauri/ are siblings here.
dev:
	npx --prefix landing tauri dev

down:
	@echo "Stopping dev processes..."
	-pkill -f 'target/debug/bio-digital-defense' 2>/dev/null
	-pkill -f 'tauri dev' 2>/dev/null
	-lsof -ti:5173 | xargs kill 2>/dev/null
	@echo "Done."

build:
	npx --prefix landing tauri build

check:
	cargo check --manifest-path src-tauri/Cargo.toml

test:
	cargo test --manifest-path src-tauri/Cargo.toml

fmt:
	cargo fmt --manifest-path src-tauri/Cargo.toml

lint:
	cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

clean:
	rm -rf src-tauri/target src-tauri/gen landing/dist landing/.vite
