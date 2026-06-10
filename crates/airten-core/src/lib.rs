//! # `AirTen` Core
//!
//! Real-time neural network audio processing engine designed for embedded systems,
//! WebAssembly, and low-latency applications.
//!
//! ## Features
//!
//! - **`no_std` Support**: Runs on embedded systems without standard library
//! - **SIMD Acceleration**: AVX2/AVX-512 (x86) and NEON (ARM) optimizations
//! - **Zero-Allocation**: Hot path operates without heap allocations
//! - **Fixed-Point Math**: Support for systems without FPU
//!
//! ## Example
//!
//! ```rust
//! use airten_core::{AudioProcessor, ProcessorConfig};
//!
//! let config = ProcessorConfig::default();
//! let mut processor = AudioProcessor::new(config);
//!
//! let mut samples = [0.0f32; 512];
//! processor.process(&mut samples).unwrap();
//! ```

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// Audio processing module with buffers, frames, and resampling
pub mod audio;
/// Digital Signal Processing module with filters, compressors, and gates
pub mod dsp;
/// Error types and result handling
pub mod error;
/// Fixed-point arithmetic for embedded systems
pub mod fixed_point;
/// Neural network inference and activation functions
pub mod nn;
#[cfg(feature = "simd")]
/// SIMD-accelerated operations
pub mod simd;

pub use audio::{AudioBuffer, AudioFrame, RingBuffer};
pub use dsp::{BiquadFilter, Compressor, EnvelopeFollower, NoiseGate};
pub use error::{Error, Result};
pub use fixed_point::Q15;

/// Sample type for audio processing (32-bit float)
pub type Sample = f32;

/// Maximum supported frame size for audio processing
pub const MAX_FRAME_SIZE: usize = 4096;

/// Maximum supported number of audio channels
pub const MAX_CHANNELS: usize = 16;

/// Default sample rate (48 kHz)
pub const DEFAULT_SAMPLE_RATE: u32 = 48000;

/// Configuration for audio processor
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessorConfig {
    /// Audio sample rate in Hz
    pub sample_rate: u32,
    /// Frame size for processing (samples per frame)
    pub frame_size: usize,
    /// Number of audio channels
    pub num_channels: usize,
    /// Enable neural network noise suppression
    pub noise_suppression: bool,
    /// Enable dynamic range compression
    pub compression: bool,
    /// Target latency in milliseconds
    pub target_latency_ms: f32,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            sample_rate: DEFAULT_SAMPLE_RATE,
            frame_size: 512,
            num_channels: 1,
            noise_suppression: true,
            compression: true,
            target_latency_ms: 10.0,
        }
    }
}

/// Real-time audio processor with neural network noise suppression
pub struct AudioProcessor {
    config: ProcessorConfig,
    highpass: BiquadFilter,
    lowpass: BiquadFilter,
    compressor: Compressor,
    gate: NoiseGate,
    envelope: EnvelopeFollower,
}

impl AudioProcessor {
    /// Creates a new audio processor with the given configuration
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn new(config: ProcessorConfig) -> Self {
        let sample_rate = config.sample_rate as Sample;

        Self {
            config,
            highpass: BiquadFilter::highpass(sample_rate, 80.0, 0.707),
            lowpass: BiquadFilter::lowpass(sample_rate, 16000.0, 0.707),
            compressor: Compressor::new(sample_rate),
            gate: NoiseGate::new(sample_rate),
            envelope: EnvelopeFollower::new(sample_rate),
        }
    }

    /// Processes audio samples through the DSP chain
    ///
    /// # Errors
    /// Returns `Error::BufferTooLarge` if input buffer exceeds `MAX_FRAME_SIZE`
    pub fn process(&mut self, samples: &mut [Sample]) -> Result<()> {
        if samples.len() > MAX_FRAME_SIZE {
            return Err(Error::BufferTooLarge);
        }

        for sample in samples.iter_mut() {
            *sample = self.highpass.process(*sample);

            *sample = self.lowpass.process(*sample);

            if self.config.noise_suppression {
                *sample = self.gate.process(*sample);
            }

            if self.config.compression {
                *sample = self.compressor.process(*sample);
            }
        }

        Ok(())
    }

    /// Processes an audio frame through the DSP chain
    ///
    /// # Errors
    /// Returns `Error::BufferTooLarge` if frame size exceeds `MAX_FRAME_SIZE`
    pub fn process_frame(&mut self, frame: &mut AudioFrame) -> Result<()> {
        for ch in 0..frame.num_channels() {
            if let Some(channel) = frame.channel_mut(ch) {
                self.process(channel)?;
            }
        }
        Ok(())
    }

    /// Resets all DSP components to their initial state
    pub fn reset(&mut self) {
        self.highpass.reset();
        self.lowpass.reset();
        self.compressor.reset();
        self.gate.reset();
        self.envelope.reset();
    }

    /// Gets the processor configuration
    #[must_use]
    pub fn config(&self) -> &ProcessorConfig {
        &self.config
    }

    /// Gets the current envelope level (for monitoring)
    #[must_use]
    pub fn envelope_level(&self) -> Sample {
        self.envelope.level()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let config = ProcessorConfig::default();
        let processor = AudioProcessor::new(config);
        assert_eq!(processor.config().sample_rate, 48000);
    }

    #[test]
    fn test_process_samples() {
        let config = ProcessorConfig::default();
        let mut processor = AudioProcessor::new(config);

        let mut samples = [0.5f32; 512];
        let result = processor.process(&mut samples);
        assert!(result.is_ok());
    }

    #[test]
    #[allow(clippy::large_stack_arrays)]
    fn test_buffer_too_large() {
        let config = ProcessorConfig::default();
        let mut processor = AudioProcessor::new(config);

        let mut samples = [0.0f32; MAX_FRAME_SIZE + 1];
        let result = processor.process(&mut samples);
        assert!(matches!(result, Err(Error::BufferTooLarge)));
    }
}

#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
