# AirTen Architecture Overview

## System Design

AirTen is designed as a modular, high-performance audio processing library with the following key architectural principles:

### Core Principles

1. **Zero-Allocation Hot Path**: All real-time audio processing operates without heap allocations
2. **`no_std` Compatibility**: Core functionality works on embedded systems
3. **SIMD Acceleration**: Automatic vectorization for supported platforms
4. **Lock-Free Concurrency**: Ring buffers and atomic operations for thread safety

## Crate Structure

```
airten/
├── airten-core       # Core DSP engine (no_std)
├── airten-ffi        # C FFI bindings
├── airten-python     # Python bindings (PyO3)
├── airten-wasm       # WebAssembly bindings
├── airten-cli        # Command-line interface
├── airten-bench      # Benchmarking utilities
└── airten-test-utils # Testing utilities
```

## Data Flow

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Input     │────▶│  Processor   │────▶│   Output    │
│   Audio     │     │   Pipeline   │     │   Audio     │
└─────────────┘     └──────────────┘     └─────────────┘
                           │
                    ┌──────┴──────┐
                    ▼             ▼
              ┌──────────┐  ┌──────────┐
              │  Filter  │  │ Dynamics │
              │  Chain   │  │  Chain   │
              └──────────┘  └──────────┘
```

## Processing Pipeline

1. **Input Stage**
   - Sample format conversion
   - Resampling (if needed)
   - Channel routing

2. **Filter Stage**
   - High-pass filter (DC removal)
   - Low-pass filter (anti-aliasing)
   - Custom EQ

3. **Dynamics Stage**
   - Noise gate
   - Compressor
   - Limiter

4. **Output Stage**
   - Gain adjustment
   - Soft clipping
   - Format conversion

## Memory Management

### Static Allocation (no_std)
- Fixed-size buffers for audio frames
- Pool allocator for temporary buffers
- No runtime memory allocation

### Dynamic Allocation (std)
- Vec-based buffers for flexibility
- Automatic capacity management
- Optional pre-allocation

## Thread Safety

- **Single-threaded**: Direct processing calls
- **Multi-threaded**: Lock-free ring buffers for producer/consumer
- **Real-time safe**: No locks in audio callback

## Platform Support

| Platform | Architecture | Features |
|----------|-------------|----------|
| Linux | x86_64, ARM64 | Full |
| macOS | x86_64, ARM64 | Full |
| Windows | x86_64 | Full |
| iOS | ARM64 | Core only |
| Android | ARM64 | Core only |
| WebAssembly | wasm32 | Core + WASM bindings |
| Embedded | ARM Cortex-M | Core (no_std) |
