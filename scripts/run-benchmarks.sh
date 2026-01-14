#!/bin/bash
set -euo pipefail

echo "=== Running AirTen Benchmarks ==="

# Run Criterion benchmarks
echo "Running Criterion benchmarks..."
cargo bench --workspace

# Run CLI benchmark
echo "Running CLI latency benchmark..."
cargo build --package airten-cli --release
./target/release/airten benchmark --iterations 10000 --frame-size 512

echo "=== Benchmarks Complete ==="
