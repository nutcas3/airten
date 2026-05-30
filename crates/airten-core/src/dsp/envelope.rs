use crate::Sample;
use crate::dsp::time_constant;

/// Envelope follower for tracking signal amplitude over time
pub struct EnvelopeFollower {
    sample_rate: Sample,
    attack_coeff: Sample,
    release_coeff: Sample,
    envelope: Sample,
}

impl EnvelopeFollower {
    /// Creates a new envelope follower
    #[must_use]
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

    /// Sets attack time in milliseconds
    pub fn set_attack(&mut self, attack_ms: Sample) {
        self.attack_coeff = time_constant(attack_ms, self.sample_rate);
    }

    /// Sets release time in milliseconds
    pub fn set_release(&mut self, release_ms: Sample) {
        self.release_coeff = time_constant(release_ms, self.sample_rate);
    }

    /// Processes a single sample
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

    /// Processes a block of samples and returns envelope level
    pub fn process_block(&mut self, samples: &[Sample]) -> Sample {
        for &sample in samples {
            self.process(sample);
        }
        self.envelope
    }

    /// Gets current envelope level
    #[must_use]
    pub fn level(&self) -> Sample {
        self.envelope
    }

    /// Resets the envelope follower state
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }
}

/// Peak hold envelope for peak detection with hold time
#[allow(dead_code)]
pub struct PeakHoldEnvelope {
    sample_rate: Sample,
    hold_samples: usize,
    release_coeff: Sample,
    peak: Sample,
    hold_counter: usize,
}

#[allow(dead_code)]
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
        follower.set_release(100.0);
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
            self.peak *= self.release_coeff;
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
