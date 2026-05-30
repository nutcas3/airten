use crate::Sample;

// Generate a sine wave with specified parameters
pub fn sine_wave(
    frequency: f32,
    sample_rate: u32,
    num_samples: usize,
    amplitude: f32,
) -> Vec<Sample> {
    let omega = 2.0 * std::f32::consts::PI * frequency / sample_rate as f32;
    (0..num_samples)
        .map(|i| (omega * i as f32).sin() * amplitude)
        .collect()
}

/// Generate a square wave
pub fn square_wave(
    frequency: f32,
    sample_rate: u32,
    num_samples: usize,
    amplitude: f32,
) -> Vec<Sample> {
    let period = sample_rate as f32 / frequency;
    (0..num_samples)
        .map(|i| {
            if (i as f32 % period) < (period / 2.0) {
                amplitude
            } else {
                -amplitude
            }
        })
        .collect()
}

/// Generate a sawtooth wave
pub fn sawtooth_wave(
    frequency: f32,
    sample_rate: u32,
    num_samples: usize,
    amplitude: f32,
) -> Vec<Sample> {
    let period = sample_rate as f32 / frequency;
    (0..num_samples)
        .map(|i| {
            let phase = (i as f32 % period) / period;
            (2.0 * phase - 1.0) * amplitude
        })
        .collect()
}

/// Generate a triangle wave
pub fn triangle_wave(
    frequency: f32,
    sample_rate: u32,
    num_samples: usize,
    amplitude: f32,
) -> Vec<Sample> {
    let period = sample_rate as f32 / frequency;
    (0..num_samples)
        .map(|i| {
            let phase = (i as f32 % period) / period;
            let value = if phase < 0.5 {
                4.0 * phase - 1.0
            } else {
                3.0 - 4.0 * phase
            };
            value * amplitude
        })
        .collect()
}

/// Generate pink noise (1/f noise)
pub fn pink_noise(num_samples: usize, amplitude: f32) -> Vec<Sample> {
    // Simple pink noise approximation using multiple octaves
    let mut samples = vec![0.0; num_samples];
    let mut seed = 12345u32;

    for octave in 0..5 {
        let freq = 1 << octave;
        for i in 0..num_samples {
            if i % freq == 0 {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let rand = ((seed >> 16) as f32 / 32768.0) - 1.0;
                samples[i] += rand * amplitude / (octave + 1) as f32;
            }
        }
    }

    samples
}
