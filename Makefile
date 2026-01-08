.PHONY: all build test bench clean fmt lint doc install-tools pre-commit build-bindings

CARGO := cargo
PYTHON := python3

all: fmt lint test build

build:
	$(CARGO) build --workspace --release

build-debug:
	$(CARGO) build --workspace

test:
	$(CARGO) test --workspace --all-features

test-no-std:
	$(CARGO) test --package airten-core --no-default-features

bench:
	$(CARGO) bench --workspace

clean:
	$(CARGO) clean
	rm -rf target/
	rm -rf crates/airten-python/target/
	rm -rf crates/airten-wasm/pkg/

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

doc:
	$(CARGO) doc --workspace --all-features --no-deps --open

doc-build:
	$(CARGO) doc --workspace --all-features --no-deps

install-tools:
	rustup component add rustfmt clippy llvm-tools-preview
	cargo install cargo-deny cargo-audit cargo-outdated
	cargo install maturin wasm-pack cbindgen
	cargo install cargo-llvm-cov criterion

pre-commit: fmt-check lint test
	$(CARGO) deny check
	$(CARGO) audit

build-python:
	cd crates/airten-python && maturin build --release

build-wasm:
	cd crates/airten-wasm && wasm-pack build --release --target web

build-ffi:
	$(CARGO) build --package airten-ffi --release
	cd crates/airten-ffi && cbindgen --config cbindgen.toml --output include/airten.h

build-bindings: build-ffi build-python build-wasm

coverage:
	$(CARGO) llvm-cov --workspace --html

audit:
	$(CARGO) audit
	$(CARGO) deny check

outdated:
	$(CARGO) outdated --workspace

release-patch:
	./scripts/release.sh patch

release-minor:
	./scripts/release.sh minor

release-major:
	./scripts/release.sh major
