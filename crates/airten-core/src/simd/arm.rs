//! ARM NEON SIMD optimizations (requires nightly Rust with portable_simd feature)
//!
//! This module is disabled on stable Rust. The scalar fallback in mod.rs is used instead.

#![cfg(all(feature = "simd", target_arch = "aarch64"))]

use crate::Sample;

/// Process gain using scalar operations (SIMD disabled on stable)
#[inline]
pub fn process_gain_simd(input: &[Sample], output: &mut [Sample], gain: Sample) {
    let len = input.len().min(output.len());
    for i in 0..len {
        output[i] = input[i] * gain;
    }
}

/// Mix signals using scalar operations (SIMD disabled on stable)
#[inline]
pub fn mix_signals_simd(a: &[Sample], b: &[Sample], output: &mut [Sample], mix: Sample) {
    let len = a.len().min(b.len()).min(output.len());
    let inv_mix = 1.0 - mix;
    for i in 0..len {
        output[i] = a[i] * inv_mix + b[i] * mix;
    }
}

/// Calculate RMS using scalar operations (SIMD disabled on stable)
#[inline]
pub fn calculate_rms_simd(samples: &[Sample]) -> Sample {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: Sample = samples.iter().map(|&x| x * x).sum();
    (sum / samples.len() as Sample).sqrt()
}

/// Find peak using scalar operations (SIMD disabled on stable)
#[inline]
pub fn find_peak_simd(samples: &[Sample]) -> Sample {
    samples.iter().map(|&x| x.abs()).fold(0.0f32, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_gain() {
        let input: [Sample; 16] = [1.0; 16];
        let mut output = [0.0; 16];

        process_gain_simd(&input, &mut output, 0.5);

        for &o in &output {
            assert!((o - 0.5).abs() < 0.001);
        }
    }

    #[test]
    fn test_simd_rms() {
        let samples = [1.0f32; 16];
        let rms = calculate_rms_simd(&samples);
        assert!((rms - 1.0).abs() < 0.001);
    }
}
