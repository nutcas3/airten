# Real-Time Audio Processing Guide

## Overview

AirTen is designed for real-time audio processing with ultra-low latency. This guide covers best practices for integrating AirTen into real-time audio applications.

## Latency Considerations

### Target Latency

AirTen targets **<10ms** total processing latency, which includes:

- **Buffer latency**: Time to fill input buffer
- **Processing latency**: Time to process samples
- **Output latency**: Time to output processed samples

### Frame Size Selection

| Frame Size | Buffer Latency (48kHz) | Use Case |
|------------|------------------------|----------|
| 64 | 1.3ms | Ultra-low latency |
| 128 | 2.7ms | Low latency (recommended) |
| 256 | 5.3ms | Balanced |
| 512 | 10.7ms | High quality |

## Integration Patterns

### Audio Callback Pattern

```rust
use airten_core::{AudioProcessor, ProcessorConfig};

struct AudioCallback {
    processor: AudioProcessor,
}

impl AudioCallback {
    fn new(sample_rate: u32) -> Self {
        let config = ProcessorConfig {
            sample_rate,
            frame_size: 128,
            ..Default::default()
        };
        Self {
            processor: AudioProcessor::new(config),
        }
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        output.copy_from_slice(input);
        self.processor.process(output).unwrap();
    }
}
```

### Ring Buffer Pattern

For producer/consumer scenarios:

```rust
use airten_core::audio::RingBuffer;

// Producer thread
fn producer(ring: &mut RingBuffer<4096>, input: &[f32]) {
    ring.write(input);
}

// Consumer thread (audio callback)
fn consumer(ring: &mut RingBuffer<4096>, output: &mut [f32]) {
    ring.read(output);
}
```

## Platform-Specific Integration

### CPAL (Cross-Platform Audio)

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use airten_core::{AudioProcessor, ProcessorConfig};

fn setup_audio() {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();
    
    let mut processor = AudioProcessor::new(ProcessorConfig {
        sample_rate: config.sample_rate().0,
        frame_size: 128,
        ..Default::default()
    });

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            processor.process(data).unwrap();
        },
        |err| eprintln!("Error: {}", err),
        None,
    ).unwrap();

    stream.play().unwrap();
}
```

### Web Audio API (WASM)

```javascript
class AirtenNode extends AudioWorkletNode {
    constructor(context) {
        super(context, 'airten-processor');
    }
}

// Register worklet
await context.audioWorklet.addModule('worklet-processor.js');
const airtenNode = new AirtenNode(context);

// Connect to audio graph
source.connect(airtenNode);
airtenNode.connect(context.destination);
```

## Performance Optimization

### Avoid Allocations

```rust
// Bad: Allocates on every call
fn process_bad(input: &[f32]) -> Vec<f32> {
    let mut output = input.to_vec();
    // process...
    output
}

// Good: Reuse buffers
fn process_good(input: &[f32], output: &mut [f32]) {
    output.copy_from_slice(input);
    // process...
}
```

### Use SIMD

Enable SIMD for vectorized processing:

```toml
[dependencies]
airten-core = { version = "1.0", features = ["simd"] }
```

### Batch Processing

Process multiple frames together when possible:

```rust
// Process 4 frames at once
let mut buffer = [0.0f32; 512]; // 4 x 128
processor.process(&mut buffer)?;
```

## Debugging Latency Issues

### Measure Processing Time

```rust
use std::time::Instant;

let start = Instant::now();
processor.process(&mut samples)?;
let duration = start.elapsed();

if duration.as_micros() > 1000 {
    eprintln!("Warning: Processing took {}µs", duration.as_micros());
}
```

### Use CLI Benchmark

```bash
airten benchmark --iterations 10000 --frame-size 128
```
