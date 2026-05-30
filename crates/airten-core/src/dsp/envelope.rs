use crate::Sample;
use crate::dsp::time_constant;

pub struct EnvelopeFollower {
    sample_rate: Sample,
    attack_coeff: Sample,
    release_coeff: Sample,
    envelope: Sample,
}

impl EnvelopeFollower {
    pub fn new(sample_rate: Sample) -> Self {
        let mut follower = Self {
            sample_rate,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            envelope: 0.0,
        };
        follower.set_attack(10.0);
        follower.set_release(100.0);
        follower
    }

    pub fn set_attack(&mut self, attack_ms: Sample) {
        self.attack_coeff = time_constant(attack_ms, self.sample_rate);
    }

    pub fn set_release(&mut self, release_ms: Sample) {
        self.release_coeff = time_constant(release_ms, self.sample_rate);
    }

    #[inline]
    pub fn process(&mut self, input: Sample) -> Sample {
        let input_abs = input.abs();

        let coeff = if input_abs > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };

        self.envelope = coeff * self.envelope + (1.0 - coeff) * input_abs;
        self.envelope
    }

    pub fn process_block(&mut self, samples: &[Sample]) -> Sample {
        for &sample in samples {
            self.process(sample);
        }
        self.envelope
    }

    #[inline]
    pub fn level(&self) -> Sample {
        self.envelope
    }

    /// Resets the envelope follower state
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }
}

pub struct RmsEnvelopeFollower {
    sample_rate: Sample,
    window_size: usize,
    buffer: [Sample; 1024],
    write_pos: usize,
    sum_squares: Sample,
    attack_coeff: Sample,
    release_coeff: Sample,
    envelope: Sample,
}

impl RmsEnvelopeFollower {
    pub fn new(sample_rate: Sample, window_ms: Sample) -> Self {
        let window_size = ((window_ms * 0.001 * sample_rate) as usize).min(1024);

        let mut follower = Self {
            sample_rate,
            window_size,
            buffer: [0.0; 1024],
            write_pos: 0,
            sum_squares: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            envelope: 0.0,
        };
        follower.set_attack(5.0);
        follower.set_release(50.0);
        follower
    }

    pub fn set_attack(&mut self, attack_ms: Sample) {
        self.attack_coeff = time_constant(attack_ms, self.sample_rate);
    }

    pub fn set_release(&mut self, release_ms: Sample) {
        self.release_coeff = time_constant(release_ms, self.sample_rate);
    }

    #[inline]
    pub fn process(&mut self, input: Sample) -> Sample {
        // Remove old sample from sum
        let old_sample = self.buffer[self.write_pos];
        self.sum_squares -= old_sample * old_sample;

        // Add new sample
        self.buffer[self.write_pos] = input;
        self.sum_squares += input * input;

        // Advance write position
        self.write_pos = (self.write_pos + 1) % self.window_size;

        // Calculate RMS
        let rms = (self.sum_squares / self.window_size as Sample).sqrt();

        // Smooth with attack/release
        let coeff = if rms > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };

        self.envelope = coeff * self.envelope + (1.0 - coeff) * rms;
        self.envelope
    }

    #[inline]
    pub fn level(&self) -> Sample {
        self.envelope
    }

    /// Resets the RMS envelope follower state
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.buffer = [0.0; 1024];
        self.write_pos = 0;
        self.sum_squares = 0.0;
        self.envelope = 0.0;
    }
}

pub struct PeakHoldEnvelope {
    sample_rate: Sample,
    hold_samples: usize,
    release_coeff: Sample,
    peak: Sample,
    hold_counter: usize,
}

impl PeakHoldEnvelope {
    pub fn new(sample_rate: Sample) -> Self {
        let mut follower = Self {
            sample_rate,
            hold_samples: 0,
            release_coeff: 0.0,
            peak: 0.0,
            hold_counter: 0,
        };
        follower.set_hold(100.0);
        follower.set_release(500.0);
        follower
    }

    pub fn set_hold(&mut self, hold_ms: Sample) {
        self.hold_samples = (hold_ms * 0.001 * self.sample_rate) as usize;
    }

    pub fn set_release(&mut self, release_ms: Sample) {
        self.release_coeff = time_constant(release_ms, self.sample_rate);
    }

    #[inline]
    pub fn process(&mut self, input: Sample) -> Sample {
        let input_abs = input.abs();

        if input_abs >= self.peak {
            self.peak = input_abs;
            self.hold_counter = self.hold_samples;
        } else if self.hold_counter > 0 {
            self.hold_counter -= 1;
        } else {
            self.peak = self.release_coeff * self.peak;
        }

        self.peak
    }

    #[inline]
    pub fn level(&self) -> Sample {
        self.peak
    }

    /// Resets the peak hold envelope state
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.peak = 0.0;
        self.hold_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_follower() {
        let mut env = EnvelopeFollower::new(48000.0);

        for i in 0..1000 {
            let input = (i as Sample) / 1000.0;
            env.process(input);
        }

        assert!(env.level() > 0.5);
    }

    #[test]
    fn test_envelope_attack_release() {
        let mut env = EnvelopeFollower::new(48000.0);
        env.set_attack(1.0);
        env.set_release(10.0);

        for _ in 0..500 {
            env.process(1.0);
        }
        let attack_level = env.level();

        for _ in 0..5000 {
            env.process(0.0);
        }
        let release_level = env.level();

        assert!(attack_level > release_level);
    }

    #[test]
    fn test_rms_envelope() {
        let mut env = RmsEnvelopeFollower::new(48000.0, 10.0);

        for _ in 0..10000 {
            env.process(0.5);
        }

        assert!((env.level() - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_peak_hold() {
        let mut env = PeakHoldEnvelope::new(48000.0);
        env.set_hold(10.0);

        env.process(1.0);

        for _ in 0..100 {
            env.process(0.0);
        }

        assert!((env.level() - 1.0).abs() < 0.01);
    }
}
