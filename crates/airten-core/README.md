# airten-core

Core DSP engine for real-time audio processing with `no_std` support.

## Features

- **Ultra-Low Latency**: <10ms processing latency
- **`no_std` Compatible**: Runs on embedded systems
- **SIMD Accelerated**: AVX2/NEON optimizations
- **Zero-Allocation**: Hot path operates without heap allocations
- **Fixed-Point Math**: Q15/Q31 support for systems without FPU

## Usage

```rust
use airten_core::{AudioProcessor, ProcessorConfig};

let config = ProcessorConfig::default();
let mut processor = AudioProcessor::new(config);

let mut samples = vec![0.5f32; 512];
processor.process(&mut samples)?;
```

## Components

- **Audio Primitives**: Frames, buffers, ring buffers, resampling
- **DSP**: Filters, compressor, noise gate, envelope follower
- **Neural Network**: Lightweight inference engine
- **SIMD**: Platform-specific optimizations
- **Fixed-Point**: Integer math for embedded systems

## Platform Support

- Linux (x86_64, ARM64)
- macOS (x86_64, ARM64)
- Windows (x86_64)
- WebAssembly
- Embedded (ARM Cortex-M)

## License

MIT OR Apache-2.0
