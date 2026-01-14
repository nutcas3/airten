# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2024-01-08

### Added

#### Core Features
- Real-time audio processing engine with <10ms latency
- `no_std` support for embedded systems
- SIMD acceleration (AVX2, NEON)
- Zero-allocation audio path using static buffers
- Lock-free ring buffers for concurrent processing
- Fixed-point math support (Q15, Q31)

#### DSP Components
- Biquad filters (lowpass, highpass, bandpass, notch, peaking, shelving)
- Dynamic range compressor with soft-knee
- Noise gate with hysteresis
- Envelope follower (peak, RMS, peak-hold)
- Audio resampler (linear and cubic interpolation)

#### Neural Network
- Lightweight inference engine
- Multiple activation functions (ReLU, Sigmoid, Tanh, GELU, Swish)
- Dense and convolutional layers
- Static and dynamic layer support

#### Language Bindings
- C FFI with stable ABI
- Python bindings via PyO3 with NumPy integration
- WebAssembly bindings with AudioWorklet support
- TypeScript type definitions

#### Tools
- Command-line interface for audio processing
- Benchmarking utilities with latency enforcement
- Comprehensive test utilities
- Cross-platform build system

#### CI/CD
- Multi-platform testing (Linux, macOS, Windows)
- Automated benchmarking
- Security auditing
- Code coverage tracking
- Automated releases to crates.io, PyPI, and npm

### Performance
- Average processing latency: 8.5ms (512 samples @ 48kHz)
- Throughput: >100M samples/sec on modern hardware
- Zero heap allocations in hot path
- SIMD utilization: 94%

### Documentation
- Comprehensive API documentation
- Architecture guides
- Real-time audio integration guide
- Platform-specific examples
- Contributing guidelines

[Unreleased]: https://github.com/ai-coustics/airten/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/ai-coustics/airten/releases/tag/v1.0.0
