use airten_core::Sample;

pub mod generators;
pub mod regression;
pub mod fixtures;

/// Generate a sine wave
pub fn generate_sine(frequency: f32, sample_rate: u32, num_samples: usize) -> Vec<Sample> {
    let omega = 2.0 * std::f32::consts::PI * frequency / sample_rate as f32;
    (0..num_samples).map(|i| (omega * i as f32).sin()).collect()
}

/// Generate white noise
pub fn generate_white_noise(num_samples: usize, amplitude: f32) -> Vec<Sample> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    (0..num_samples)
        .map(|i| {
            i.hash(&mut hasher);
            let hash = hasher.finish();
            ((hash as f32 / u64::MAX as f32) * 2.0 - 1.0) * amplitude
        })
        .collect()
}

/// Generate a chirp (frequency sweep)
pub fn generate_chirp(
    start_freq: f32,
    end_freq: f32,
    sample_rate: u32,
    duration_secs: f32,
) -> Vec<Sample> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let k = (end_freq - start_freq) / duration_secs;

    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let phase = 2.0 * std::f32::consts::PI * (start_freq * t + 0.5 * k * t * t);
            phase.sin()
        })
        .collect()
}

/// Generate an impulse
pub fn generate_impulse(num_samples: usize, position: usize) -> Vec<Sample> {
    let mut samples = vec![0.0; num_samples];
    if position < num_samples {
        samples[position] = 1.0;
    }
    samples
}

/// Generate silence
pub fn generate_silence(num_samples: usize) -> Vec<Sample> {
    vec![0.0; num_samples]
}

/// Calculate RMS (Root Mean Square) of audio samples
pub fn calculate_rms(samples: &[Sample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Find peak amplitude in audio samples
pub fn find_peak(samples: &[Sample]) -> f32 {
    samples.iter().map(|&x| x.abs()).fold(0.0f32, f32::max)
}

/// Calculate Signal-to-Noise Ratio in dB
pub fn calculate_snr(signal: &[Sample], noise: &[Sample]) -> f32 {
    let signal_power: f32 = signal.iter().map(|&x| x * x).sum::<f32>() / signal.len() as f32;
    let noise_power: f32 = noise.iter().map(|&x| x * x).sum::<f32>() / noise.len() as f32;

    if noise_power < 1e-10 {
        return 120.0; // Effectively infinite SNR
    }

    10.0 * (signal_power / noise_power).log10()
}

/// Calculate Total Harmonic Distortion
pub fn calculate_thd(samples: &[Sample], fundamental_freq: f32, sample_rate: u32) -> f32 {
    // Simple THD estimation using DFT at harmonic frequencies
    let n = samples.len();
    let fundamental_bin = (fundamental_freq * n as f32 / sample_rate as f32).round() as usize;

    let mut fundamental_power = 0.0f32;
    let mut harmonic_power = 0.0f32;

    // Calculate power at fundamental
    let omega = 2.0 * std::f32::consts::PI * fundamental_bin as f32 / n as f32;
    let mut real = 0.0f32;
    let mut imag = 0.0f32;
    for (i, &sample) in samples.iter().enumerate() {
        real += sample * (omega * i as f32).cos();
        imag += sample * (omega * i as f32).sin();
    }
    fundamental_power = (real * real + imag * imag) / (n * n) as f32;

    // Calculate power at harmonics (2nd through 5th)
    for harmonic in 2..=5 {
        let bin = fundamental_bin * harmonic;
        if bin >= n / 2 {
            break;
        }

        let omega = 2.0 * std::f32::consts::PI * bin as f32 / n as f32;
        let mut real = 0.0f32;
        let mut imag = 0.0f32;
        for (i, &sample) in samples.iter().enumerate() {
            real += sample * (omega * i as f32).cos();
            imag += sample * (omega * i as f32).sin();
        }
        harmonic_power += (real * real + imag * imag) / (n * n) as f32;
    }

    if fundamental_power < 1e-10 {
        return 0.0;
    }

    (harmonic_power / fundamental_power).sqrt() * 100.0
}

/// Check if two audio buffers are approximately equal
pub fn buffers_approx_equal(a: &[Sample], b: &[Sample], epsilon: f32) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.iter()
        .zip(b.iter())
        .all(|(&x, &y)| (x - y).abs() < epsilon)
}

/// Calculate the maximum absolute difference between two buffers
pub fn max_difference(a: &[Sample], b: &[Sample]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x - y).abs())
        .fold(0.0f32, f32::max)
}

/// Detect clipping in audio samples
pub fn detect_clipping(samples: &[Sample], threshold: f32) -> Vec<usize> {
    samples
        .iter()
        .enumerate()
        .filter(|&(_, s)| s.abs() >= threshold)
        .map(|(i, _)| i)
        .collect()
}

/// Calculate crest factor (peak to RMS ratio) in dB
pub fn crest_factor(samples: &[Sample]) -> f32 {
    let peak = samples.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
    let rms = (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt();
    
    if rms < 1e-10 {
        return 0.0;
    }
    
    20.0 * (peak / rms).log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sine() {
        let samples = generate_sine(440.0, 48000, 480);
        assert_eq!(samples.len(), 480);
        
        // Check that values are in valid range
        for &s in &samples {
            assert!(s >= -1.0 && s <= 1.0);
        }
    }

    #[test]
    fn test_generate_white_noise() {
        let samples = generate_white_noise(1000, 0.5);
        assert_eq!(samples.len(), 1000);
        
        // Check amplitude bounds
        for &s in &samples {
            assert!(s >= -0.5 && s <= 0.5);
        }
    }

    #[test]
    fn test_generate_impulse() {
        let samples = generate_impulse(100, 50);
        assert_eq!(samples.len(), 100);
        assert_eq!(samples[50], 1.0);
        assert_eq!(samples[0], 0.0);
        assert_eq!(samples[99], 0.0);
    }

    #[test]
    fn test_snr_calculation() {
        let signal = generate_sine(1000.0, 48000, 4800);
        let noise = generate_white_noise(4800, 0.01);
        
        let snr = calculate_snr(&signal, &noise);
        assert!(snr > 20.0); // Should have decent SNR
    }

    #[test]
    fn test_buffers_approx_equal() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.001, 2.001, 3.001];
        
        assert!(buffers_approx_equal(&a, &b, 0.01));
        assert!(!buffers_approx_equal(&a, &b, 0.0001));
    }

    #[test]
    fn test_detect_clipping() {
        let samples = vec![0.5, 0.9, 1.0, 0.8, -1.0, 0.3];
        let clipped = detect_clipping(&samples, 0.99);
        
        assert_eq!(clipped, vec![2, 4]);
    }

    #[test]
    fn test_crest_factor() {
        // Sine wave has crest factor of ~3dB
        let sine = generate_sine(1000.0, 48000, 4800);
        let cf = crest_factor(&sine);
        
        assert!((cf - 3.0).abs() < 0.5);
    }
}
