#!/bin/bash
set -euo pipefail

echo "=== Building AirTen ==="

# Build all Rust crates
echo "Building Rust crates..."
cargo build --workspace --release

# Build FFI with header generation
echo "Building FFI bindings..."
cargo build --package airten-ffi --release
if command -v cbindgen &> /dev/null; then
    cd crates/airten-ffi
    cbindgen --config cbindgen.toml --output include/airten.h
    cd ../..
fi

# Build Python wheels if maturin is available
if command -v maturin &> /dev/null; then
    echo "Building Python wheels..."
    cd crates/airten-python
    maturin build --release
    cd ../..
else
    echo "Skipping Python build (maturin not installed)"
fi

# Build WASM if wasm-pack is available
if command -v wasm-pack &> /dev/null; then
    echo "Building WASM package..."
    cd crates/airten-wasm
    wasm-pack build --release --target web
    cd ../..
else
    echo "Skipping WASM build (wasm-pack not installed)"
fi

echo "=== Build Complete ==="
