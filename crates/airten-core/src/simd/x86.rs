#![cfg(all(feature = "simd", target_arch = "x86_64"))]

use crate::Sample;
use core::simd::{Simd, SimdFloat, StdFloat, f32x8};

const SIMD_WIDTH: usize = 8;

#[inline]
pub fn process_gain_simd(input: &[Sample], output: &mut [Sample], gain: Sample) {
    let len = input.len().min(output.len());
    let simd_len = len - (len % SIMD_WIDTH);
    let gain_vec = f32x8::splat(gain);

    for i in (0..simd_len).step_by(SIMD_WIDTH) {
        let chunk = f32x8::from_slice(&input[i..]);
        let result = chunk * gain_vec;
        result.copy_to_slice(&mut output[i..]);
    }

    for i in simd_len..len {
        output[i] = input[i] * gain;
    }
}

#[inline]
pub fn mix_signals_simd(a: &[Sample], b: &[Sample], output: &mut [Sample], mix: Sample) {
    let len = a.len().min(b.len()).min(output.len());
    let simd_len = len - (len % SIMD_WIDTH);

    let mix_vec = f32x8::splat(mix);
    let inv_mix_vec = f32x8::splat(1.0 - mix);

    for i in (0..simd_len).step_by(SIMD_WIDTH) {
        let a_chunk = f32x8::from_slice(&a[i..]);
        let b_chunk = f32x8::from_slice(&b[i..]);
        let result = a_chunk * inv_mix_vec + b_chunk * mix_vec;
        result.copy_to_slice(&mut output[i..]);
    }

    let inv_mix = 1.0 - mix;
    for i in simd_len..len {
        output[i] = a[i] * inv_mix + b[i] * mix;
    }
}

#[inline]
pub fn calculate_rms_simd(samples: &[Sample]) -> Sample {
    if samples.is_empty() {
        return 0.0;
    }

    let len = samples.len();
    let simd_len = len - (len % SIMD_WIDTH);
    let mut sum_vec = f32x8::splat(0.0);

    for i in (0..simd_len).step_by(SIMD_WIDTH) {
        let chunk = f32x8::from_slice(&samples[i..]);
        sum_vec += chunk * chunk;
    }

    let mut sum: Sample = sum_vec.reduce_sum();

    for i in simd_len..len {
        sum += samples[i] * samples[i];
    }

    libm::sqrtf(sum / len as Sample)
}

#[inline]
pub fn find_peak_simd(samples: &[Sample]) -> Sample {
    if samples.is_empty() {
        return 0.0;
    }

    let len = samples.len();
    let simd_len = len - (len % SIMD_WIDTH);
    let mut max_vec = f32x8::splat(0.0);

    for i in (0..simd_len).step_by(SIMD_WIDTH) {
        let chunk = f32x8::from_slice(&samples[i..]);
        let abs_chunk = chunk.abs();
        max_vec = max_vec.simd_max(abs_chunk);
    }

    let mut peak = max_vec.reduce_max();

    for i in simd_len..len {
        let abs = samples[i].abs();
        if abs > peak {
            peak = abs;
        }
    }

    peak
}

#[inline]
pub fn soft_clip_simd(samples: &mut [Sample]) {
    let len = samples.len();
    let simd_len = len - (len % SIMD_WIDTH);

    for i in (0..simd_len).step_by(SIMD_WIDTH) {
        let chunk = f32x8::from_slice(&samples[i..]);
        // Approximate tanh using rational approximation
        let x2 = chunk * chunk;
        let result =
            chunk * (f32x8::splat(27.0) + x2) / (f32x8::splat(27.0) + x2 * f32x8::splat(9.0));
        result.copy_to_slice(&mut samples[i..]);
    }

    for i in simd_len..len {
        samples[i] = libm::tanhf(samples[i]);
    }
}

pub struct SimdBiquadFilter {
    b0: f32x8,
    b1: f32x8,
    b2: f32x8,
    a1: f32x8,
    a2: f32x8,
    z1: f32x8,
    z2: f32x8,
}

impl SimdBiquadFilter {
    pub fn new(b0: Sample, b1: Sample, b2: Sample, a1: Sample, a2: Sample) -> Self {
        Self {
            b0: f32x8::splat(b0),
            b1: f32x8::splat(b1),
            b2: f32x8::splat(b2),
            a1: f32x8::splat(a1),
            a2: f32x8::splat(a2),
            z1: f32x8::splat(0.0),
            z2: f32x8::splat(0.0),
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32x8) -> f32x8 {
        let output = self.b0 * input + self.z1;
        self.z1 = self.b1 * input - self.a1 * output + self.z2;
        self.z2 = self.b2 * input - self.a2 * output;
        output
    }

    pub fn reset(&mut self) {
        self.z1 = f32x8::splat(0.0);
        self.z2 = f32x8::splat(0.0);
    }
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

    #[test]
    fn test_simd_peak() {
        let mut samples = [0.5f32; 16];
        samples[7] = -0.9;
        let peak = find_peak_simd(&samples);
        assert!((peak - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_simd_mix() {
        let a = [1.0f32; 16];
        let b = [0.0f32; 16];
        let mut output = [0.0; 16];

        mix_signals_simd(&a, &b, &mut output, 0.5);

        for &o in &output {
            assert!((o - 0.5).abs() < 0.001);
        }
    }
}
