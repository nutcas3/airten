#!/bin/bash
set -euo pipefail

echo "=== Cross-Compiling AirTen ==="

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-gnu"
    "wasm32-unknown-unknown"
)

# Install targets
echo "Installing cross-compilation targets..."
for target in "${TARGETS[@]}"; do
    rustup target add "$target" 2>/dev/null || true
done

# Build for each target
for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    
    if [[ "$target" == "wasm32-unknown-unknown" ]]; then
        cargo build --package airten-wasm --target "$target" --release || echo "Failed: $target"
    else
        cargo build --package airten-core --target "$target" --release || echo "Failed: $target"
        cargo build --package airten-ffi --target "$target" --release || echo "Failed: $target"
    fi
done

echo "=== Cross-Compilation Complete ==="
