use js_sys::Float32Array;
use wasm_bindgen::prelude::*;

use airten_core::audio::Resampler as CoreResampler;
use airten_core::dsp::{BiquadFilter, Compressor, NoiseGate};
use airten_core::{AudioProcessor as CoreProcessor, ProcessorConfig};

#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Audio processor for real-time voice enhancement
#[wasm_bindgen]
pub struct AudioProcessor {
    inner: CoreProcessor,
}

#[wasm_bindgen]
impl AudioProcessor {
    /// Create a new audio processor
    #[wasm_bindgen(constructor)]
    pub fn new(config: Option<ProcessorOptions>) -> AudioProcessor {
        let config = config.unwrap_or_default();

        let proc_config = ProcessorConfig {
            sample_rate: config.sample_rate(),
            frame_size: config.frame_size() as usize,
            num_channels: config.channels() as usize,
            noise_suppression: config.noise_suppression(),
            compression: config.compression(),
            target_latency_ms: 10.0,
        };

        AudioProcessor {
            inner: CoreProcessor::new(proc_config),
        }
    }

    /// Process audio samples in-place
    #[wasm_bindgen]
    pub fn process(&mut self, samples: &mut [f32]) -> Result<(), JsValue> {
        self.inner
            .process(samples)
            .map_err(|e| JsValue::from_str(&format!("Processing error: {}", e)))
    }

    /// Process a Float32Array and return processed samples
    #[wasm_bindgen(js_name = processArray)]
    pub fn process_array(&mut self, input: Float32Array) -> Result<Float32Array, JsValue> {
        let mut samples: Vec<f32> = input.to_vec();

        self.inner
            .process(&mut samples)
            .map_err(|e| JsValue::from_str(&format!("Processing error: {}", e)))?;

        Ok(Float32Array::from(&samples[..]))
    }

    /// Reset processor state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Get the current envelope level
    #[wasm_bindgen(getter, js_name = envelopeLevel)]
    pub fn envelope_level(&self) -> f32 {
        self.inner.envelope_level()
    }

    /// Get the sample rate
    #[wasm_bindgen(getter, js_name = sampleRate)]
    pub fn sample_rate(&self) -> u32 {
        self.inner.config().sample_rate
    }

    /// Get the frame size
    #[wasm_bindgen(getter, js_name = frameSize)]
    pub fn frame_size(&self) -> usize {
        self.inner.config().frame_size
    }
}

/// Configuration options for the processor
#[wasm_bindgen]
#[derive(Default)]
pub struct ProcessorOptions {
    sample_rate: u32,
    frame_size: u32,
    channels: u32,
    noise_suppression: bool,
    compression: bool,
}

#[wasm_bindgen]
impl ProcessorOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ProcessorOptions {
        ProcessorOptions {
            sample_rate: 48000,
            frame_size: 128,
            channels: 1,
            noise_suppression: true,
            compression: true,
        }
    }

    #[wasm_bindgen(getter, js_name = sampleRate)]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[wasm_bindgen(setter, js_name = sampleRate)]
    pub fn set_sample_rate(&mut self, value: u32) {
        self.sample_rate = value;
    }

    #[wasm_bindgen(getter, js_name = frameSize)]
    pub fn frame_size(&self) -> u32 {
        self.frame_size
    }

    #[wasm_bindgen(setter, js_name = frameSize)]
    pub fn set_frame_size(&mut self, value: u32) {
        self.frame_size = value;
    }

    #[wasm_bindgen(getter)]
    pub fn channels(&self) -> u32 {
        self.channels
    }

    #[wasm_bindgen(setter)]
    pub fn set_channels(&mut self, value: u32) {
        self.channels = value;
    }

    #[wasm_bindgen(getter, js_name = noiseSuppression)]
    pub fn noise_suppression(&self) -> bool {
        self.noise_suppression
    }

    #[wasm_bindgen(setter, js_name = noiseSuppression)]
    pub fn set_noise_suppression(&mut self, value: bool) {
        self.noise_suppression = value;
    }

    #[wasm_bindgen(getter)]
    pub fn compression(&self) -> bool {
        self.compression
    }

    #[wasm_bindgen(setter)]
    pub fn set_compression(&mut self, value: bool) {
        self.compression = value;
    }
}

/// Biquad filter for audio processing
#[wasm_bindgen]
pub struct Filter {
    inner: BiquadFilter,
}

#[wasm_bindgen]
impl Filter {
    /// Create a lowpass filter
    #[wasm_bindgen]
    pub fn lowpass(sample_rate: f32, cutoff: f32, q: f32) -> Filter {
        Filter {
            inner: BiquadFilter::lowpass(sample_rate, cutoff, q),
        }
    }

    /// Create a highpass filter
    #[wasm_bindgen]
    pub fn highpass(sample_rate: f32, cutoff: f32, q: f32) -> Filter {
        Filter {
            inner: BiquadFilter::highpass(sample_rate, cutoff, q),
        }
    }

    /// Create a bandpass filter
    #[wasm_bindgen]
    pub fn bandpass(sample_rate: f32, center: f32, q: f32) -> Filter {
        Filter {
            inner: BiquadFilter::bandpass(sample_rate, center, q),
        }
    }

    /// Create a notch filter
    #[wasm_bindgen]
    pub fn notch(sample_rate: f32, center: f32, q: f32) -> Filter {
        Filter {
            inner: BiquadFilter::notch(sample_rate, center, q),
        }
    }

    /// Create a peaking EQ filter
    #[wasm_bindgen]
    pub fn peaking(sample_rate: f32, center: f32, q: f32, gain_db: f32) -> Filter {
        Filter {
            inner: BiquadFilter::peaking(sample_rate, center, q, gain_db),
        }
    }

    /// Process samples in-place
    #[wasm_bindgen]
    pub fn process(&mut self, samples: &mut [f32]) {
        self.inner.process_block(samples);
    }

    /// Process a Float32Array
    #[wasm_bindgen(js_name = processArray)]
    pub fn process_array(&mut self, input: Float32Array) -> Float32Array {
        let mut samples: Vec<f32> = input.to_vec();
        self.inner.process_block(&mut samples);
        Float32Array::from(&samples[..])
    }

    /// Reset filter state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Dynamic range compressor
#[wasm_bindgen]
pub struct DynamicCompressor {
    inner: Compressor,
}

#[wasm_bindgen]
impl DynamicCompressor {
    /// Create a new compressor
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32) -> DynamicCompressor {
        DynamicCompressor {
            inner: Compressor::new(sample_rate),
        }
    }

    /// Set threshold in dB
    #[wasm_bindgen(js_name = setThreshold)]
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.inner.set_threshold(threshold_db);
    }

    /// Set compression ratio
    #[wasm_bindgen(js_name = setRatio)]
    pub fn set_ratio(&mut self, ratio: f32) {
        self.inner.set_ratio(ratio);
    }

    /// Set attack time in milliseconds
    #[wasm_bindgen(js_name = setAttack)]
    pub fn set_attack(&mut self, attack_ms: f32) {
        self.inner.set_attack(attack_ms);
    }

    /// Set release time in milliseconds
    #[wasm_bindgen(js_name = setRelease)]
    pub fn set_release(&mut self, release_ms: f32) {
        self.inner.set_release(release_ms);
    }

    /// Process samples in-place
    #[wasm_bindgen]
    pub fn process(&mut self, samples: &mut [f32]) {
        self.inner.process_block(samples);
    }

    /// Get current gain reduction in dB
    #[wasm_bindgen(getter, js_name = gainReductionDb)]
    pub fn gain_reduction_db(&self) -> f32 {
        self.inner.gain_reduction_db()
    }

    /// Reset compressor state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Noise gate
#[wasm_bindgen]
pub struct Gate {
    inner: NoiseGate,
}

#[wasm_bindgen]
impl Gate {
    /// Create a new noise gate
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32) -> Gate {
        Gate {
            inner: NoiseGate::new(sample_rate),
        }
    }

    /// Set threshold in dB
    #[wasm_bindgen(js_name = setThreshold)]
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.inner.set_threshold(threshold_db);
    }

    /// Set attack time in milliseconds
    #[wasm_bindgen(js_name = setAttack)]
    pub fn set_attack(&mut self, attack_ms: f32) {
        self.inner.set_attack(attack_ms);
    }

    /// Set hold time in milliseconds
    #[wasm_bindgen(js_name = setHold)]
    pub fn set_hold(&mut self, hold_ms: f32) {
        self.inner.set_hold(hold_ms);
    }

    /// Set release time in milliseconds
    #[wasm_bindgen(js_name = setRelease)]
    pub fn set_release(&mut self, release_ms: f32) {
        self.inner.set_release(release_ms);
    }

    /// Process samples in-place
    #[wasm_bindgen]
    pub fn process(&mut self, samples: &mut [f32]) {
        self.inner.process_block(samples);
    }

    /// Check if gate is currently open
    #[wasm_bindgen(getter, js_name = isOpen)]
    pub fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    /// Reset gate state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Audio resampler
#[wasm_bindgen]
pub struct Resampler {
    inner: CoreResampler,
}

#[wasm_bindgen]
impl Resampler {
    /// Create a new resampler
    #[wasm_bindgen(constructor)]
    pub fn new(input_rate: u32, output_rate: u32) -> Resampler {
        Resampler {
            inner: CoreResampler::new(input_rate, output_rate),
        }
    }

    /// Process samples
    #[wasm_bindgen]
    pub fn process(&mut self, input: Float32Array) -> Float32Array {
        let input_vec: Vec<f32> = input.to_vec();
        let output_size = self.inner.output_size(input_vec.len());
        let mut output = vec![0.0f32; output_size];

        let written = self.inner.process(&input_vec, &mut output);
        output.truncate(written);

        Float32Array::from(&output[..])
    }

    /// Reset resampler state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Get the resampling ratio
    #[wasm_bindgen(getter)]
    pub fn ratio(&self) -> f32 {
        self.inner.ratio()
    }
}

// Utility functions

/// Calculate RMS of audio samples
#[wasm_bindgen(js_name = calculateRms)]
pub fn calculate_rms(samples: Float32Array) -> f32 {
    let samples: Vec<f32> = samples.to_vec();
    if samples.is_empty() {
        return 0.0;
    }

    let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Find peak amplitude
#[wasm_bindgen(js_name = findPeak)]
pub fn find_peak(samples: Float32Array) -> f32 {
    let samples: Vec<f32> = samples.to_vec();
    samples.iter().map(|&x| x.abs()).fold(0.0f32, f32::max)
}

/// Convert decibels to linear amplitude
#[wasm_bindgen(js_name = dbToLinear)]
pub fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Convert linear amplitude to decibels
#[wasm_bindgen(js_name = linearToDb)]
pub fn linear_to_db(linear: f32) -> f32 {
    if linear <= 1e-6 {
        -120.0
    } else {
        20.0 * linear.log10()
    }
}

/// Get library version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_processor_creation() {
        let proc = AudioProcessor::new(None);
        assert_eq!(proc.sample_rate(), 48000);
    }

    #[wasm_bindgen_test]
    fn test_db_conversion() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 0.001);
        assert!((linear_to_db(1.0) - 0.0).abs() < 0.001);
    }
}
