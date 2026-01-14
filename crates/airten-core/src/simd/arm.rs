#![cfg(all(feature = "simd", target_arch = "aarch64"))]

use core::simd::{f32x4, SimdFloat, StdFloat};
use crate::Sample;

const SIMD_WIDTH: usize = 4;

#[inline]
pub fn process_gain_simd(input: &[Sample], output: &mut [Sample], gain: Sample) {
    let len = input.len().min(output.len());
    let simd_len = len - (len % SIMD_WIDTH);
    let gain_vec = f32x4::splat(gain);

    for i in (0..simd_len).step_by(SIMD_WIDTH) {
        let chunk = f32x4::from_slice(&input[i..]);
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
    
    let mix_vec = f32x4::splat(mix);
    let inv_mix_vec = f32x4::splat(1.0 - mix);

    for i in (0..simd_len).step_by(SIMD_WIDTH) {
        let a_chunk = f32x4::from_slice(&a[i..]);
        let b_chunk = f32x4::from_slice(&b[i..]);
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
    let mut sum_vec = f32x4::splat(0.0);

    for i in (0..simd_len).step_by(SIMD_WIDTH) {
        let chunk = f32x4::from_slice(&samples[i..]);
        sum_vec += chunk * chunk;
    }

    let mut sum: Sample = sum_vec.reduce_sum();

    for i in simd_len..len {
        sum += samples[i] * samples[i];
    }

    (sum / len as Sample).sqrt()
}

#[inline]
pub fn find_peak_simd(samples: &[Sample]) -> Sample {
    if samples.is_empty() {
        return 0.0;
    }

    let len = samples.len();
    let simd_len = len - (len % SIMD_WIDTH);
    let mut max_vec = f32x4::splat(0.0);

    for i in (0..simd_len).step_by(SIMD_WIDTH) {
        let chunk = f32x4::from_slice(&samples[i..]);
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
