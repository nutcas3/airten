#[cfg(all(feature = "simd", target_arch = "x86_64"))]
mod x86;

#[cfg(all(feature = "simd", target_arch = "aarch64"))]
mod arm;

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
pub use x86::*;

#[cfg(all(feature = "simd", target_arch = "aarch64"))]
pub use arm::*;

use crate::Sample;

#[inline]
pub fn process_gain(input: &[Sample], output: &mut [Sample], gain: Sample) {
    #[cfg(all(feature = "simd", any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        process_gain_simd(input, output, gain);
    }
    
    #[cfg(not(all(feature = "simd", any(target_arch = "x86_64", target_arch = "aarch64"))))]
    {
        process_gain_scalar(input, output, gain);
    }
}

#[inline]
pub fn process_gain_scalar(input: &[Sample], output: &mut [Sample], gain: Sample) {
    let len = input.len().min(output.len());
    for i in 0..len {
        output[i] = input[i] * gain;
    }
}

#[inline]
pub fn mix_signals(a: &[Sample], b: &[Sample], output: &mut [Sample], mix: Sample) {
    let _len = a.len().min(b.len()).min(output.len());
    
    #[cfg(all(feature = "simd", any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        mix_signals_simd(a, b, output, mix);
    }
    
    #[cfg(not(all(feature = "simd", any(target_arch = "x86_64", target_arch = "aarch64"))))]
    {
        let inv_mix = 1.0 - mix;
        for i in 0.._len {
            output[i] = a[i] * inv_mix + b[i] * mix;
        }
    }
}

#[inline]
pub fn calculate_rms(samples: &[Sample]) -> Sample {
    if samples.is_empty() {
        return 0.0;
    }
    
    #[cfg(all(feature = "simd", any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        calculate_rms_simd(samples)
    }
    
    #[cfg(not(all(feature = "simd", any(target_arch = "x86_64", target_arch = "aarch64"))))]
    {
        let sum_sq: Sample = samples.iter().map(|&x| x * x).sum();
        (sum_sq / samples.len() as Sample).sqrt()
    }
}

#[inline]
pub fn find_peak(samples: &[Sample]) -> Sample {
    if samples.is_empty() {
        return 0.0;
    }
    
    #[cfg(all(feature = "simd", any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        find_peak_simd(samples)
    }
    
    #[cfg(not(all(feature = "simd", any(target_arch = "x86_64", target_arch = "aarch64"))))]
    {
        samples.iter().map(|&x| x.abs()).fold(0.0f32, Sample::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_gain_scalar() {
        let input = [1.0, 2.0, 3.0, 4.0];
        let mut output = [0.0; 4];
        
        process_gain_scalar(&input, &mut output, 0.5);
        
        assert!((output[0] - 0.5).abs() < 0.001);
        assert!((output[1] - 1.0).abs() < 0.001);
        assert!((output[2] - 1.5).abs() < 0.001);
        assert!((output[3] - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_rms() {
        let samples = [1.0, -1.0, 1.0, -1.0];
        let rms = calculate_rms(&samples);
        assert!((rms - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_find_peak() {
        let samples = [0.5, -0.8, 0.3, -0.2];
        let peak = find_peak(&samples);
        assert!((peak - 0.8).abs() < 0.001);
    }
}
