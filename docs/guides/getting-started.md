# Getting Started with AirTen

## Installation

### Rust

Add AirTen to your `Cargo.toml`:

```toml
[dependencies]
airten-core = "1.0"
```

### Python

```bash
pip install airten
```

### JavaScript (WebAssembly)

```bash
npm install @airten/wasm
```

### C/C++

Download the pre-built library from releases or build from source:

```bash
cargo build --package airten-ffi --release
```

## Quick Start

### Rust

```rust
use airten_core::{AudioProcessor, ProcessorConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create processor with default settings
    let config = ProcessorConfig::default();
    let mut processor = AudioProcessor::new(config);

    // Process audio samples
    let mut samples = vec![0.5f32; 512];
    processor.process(&mut samples)?;

    Ok(())
}
```

### Python

```python
import airten
import numpy as np

# Create processor
processor = airten.AudioProcessor(sample_rate=48000)

# Process audio
audio = np.random.randn(512).astype(np.float32) * 0.5
processed = processor.process(audio)
```

### JavaScript

```javascript
import init, { AudioProcessor, ProcessorOptions } from '@airten/wasm';

async function main() {
    await init();
    
    const options = new ProcessorOptions();
    options.sampleRate = 48000;
    
    const processor = new AudioProcessor(options);
    
    const samples = new Float32Array(512);
    processor.process(samples);
}
```

### C

```c
#include <airten.h>

int main() {
    AirtenConfig config = {
        .sample_rate = 48000,
        .frame_size = 512,
        .num_channels = 1,
        .noise_suppression = 1,
        .compression = 1
    };
    
    AirtenProcessor* proc = airten_processor_new(&config);
    
    float samples[512];
    airten_process(proc, samples, 512);
    
    airten_processor_free(proc);
    return 0;
}
```

## Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `sample_rate` | 48000 | Sample rate in Hz |
| `frame_size` | 512 | Samples per frame |
| `num_channels` | 1 | Number of audio channels |
| `noise_suppression` | true | Enable noise gate |
| `compression` | true | Enable dynamic compression |

## Next Steps

- [Real-Time Audio Guide](real-time-audio.md)
- [DSP Pipeline Documentation](../architecture/dsp-pipeline.md)
- [API Reference](../api/rust.md)
