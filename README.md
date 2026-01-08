# AirTen - Voice AI Reliability Layer

[![Crates.io](https://img.shields.io/crates/v/airten-core.svg)](https://crates.io/crates/airten-core)
[![Documentation](https://docs.rs/airten-core/badge.svg)](https://docs.rs/airten-core)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/ai-coustics/airten/workflows/CI/badge.svg)](https://github.com/ai-coustics/airten/actions)

**Real-time neural network audio processing engine designed for embedded systems, WebAssembly, and low-latency applications.**

## Features

- **Ultra-Low Latency**: <10ms processing latency with zero-allocation audio paths
- **`no_std` Support**: Runs on embedded systems without standard library
- **SIMD Acceleration**: AVX2/AVX-512 (x86) and NEON (ARM) optimizations
- **Cross-Platform**: Linux, macOS, Windows, iOS, Android, WebAssembly
- **Multi-Language Bindings**: Rust, C/C++, Python, JavaScript/TypeScript
- **Production Ready**: Comprehensive testing, benchmarking, and CI/CD

## Crates

| Crate | Description |
|-------|-------------|
| [`airten-core`](crates/airten-core) | Core DSP engine with `no_std` support |
| [`airten-ffi`](crates/airten-ffi) | C FFI bindings for native integration |
| [`airten-python`](crates/airten-python) | Python bindings via PyO3 |
| [`airten-wasm`](crates/airten-wasm) | WebAssembly bindings |
| [`airten-cli`](crates/airten-cli) | Command-line interface |
| [`airten-bench`](crates/airten-bench) | Benchmarking utilities |
| [`airten-test-utils`](crates/airten-test-utils) | Testing utilities |

## Quick Start

### Rust

```toml
[dependencies]
airten-core = "1.0"
```

```rust
use airten_core::{AudioProcessor, ProcessorConfig};

let config = ProcessorConfig::default();
let mut processor = AudioProcessor::new(config);

// Process audio frame
let mut samples = [0.0f32; 512];
processor.process(&mut samples)?;
```

### Python

```bash
pip install airten
```

```python
import airten
import numpy as np

processor = airten.AudioProcessor(sample_rate=48000)
audio = np.random.randn(512).astype(np.float32)
processed = processor.process(audio)
```

### JavaScript (WebAssembly)

```bash
npm install @airten/wasm
```

```javascript
import { AudioProcessor } from '@airten/wasm';

const processor = new AudioProcessor({ sampleRate: 48000 });
const processed = processor.process(audioData);
```

### C/C++

```c
#include <airten.h>

AirtenProcessor* proc = airten_processor_new(48000, 512);
airten_process(proc, samples, num_samples);
airten_processor_free(proc);
```

## Building

### Prerequisites

- Rust 1.75+
- Python 3.8+ (for Python bindings)
- Node.js 18+ (for WASM bindings)
- wasm-pack (for WASM builds)
- maturin (for Python builds)

### Build All

```bash
# Install development tools
make install-tools

# Build all crates
cargo build --workspace --release

# Run tests
cargo test --workspace

# Build Python wheels
cd crates/airten-python && maturin build --release

# Build WASM package
cd crates/airten-wasm && wasm-pack build --release
```

## Performance

| Metric | Target | Achieved |
|--------|--------|----------|
| Processing Latency | <10ms | 8.5ms avg |
| Memory Allocation | Zero in hot path | ✅ |
| SIMD Utilization | >90% | 94% |
| Cross-platform | 8+ targets | ✅ |

## Testing

```bash
# Unit tests
cargo test --workspace

# Integration tests
cargo test --workspace --test '*'

# Benchmarks
cargo bench --workspace

# Property-based tests
cargo test --workspace --features proptest
```

## Documentation

- [API Documentation](https://docs.rs/airten-core)
- [Architecture Guide](docs/architecture/overview.md)
- [Getting Started](docs/guides/getting-started.md)
- [Real-Time Audio Guide](docs/guides/real-time-audio.md)
- [Embedded Systems Guide](docs/guides/embedded-systems.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
