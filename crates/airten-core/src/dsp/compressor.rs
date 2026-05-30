use crate::Sample;
use crate::dsp::{db_to_linear, linear_to_db, time_constant};

/// Dynamic range compressor for audio processing
pub struct Compressor {
    sample_rate: Sample,
    threshold_db: Sample,
    ratio: Sample,
    attack_coeff: Sample,
    release_coeff: Sample,
    knee_width_db: Sample,
    makeup_gain: Sample,
    envelope: Sample,
}

impl Compressor {
    /// Creates a new compressor with default settings
    #[must_use]
    pub fn new(sample_rate: Sample) -> Self {
        let mut comp = Self {
            sample_rate,
            threshold_db: -20.0,
            ratio: 4.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            knee_width_db: 6.0,
            makeup_gain: 1.0,
            envelope: 0.0,
        };
        comp.set_attack(10.0);
        comp.set_release(100.0);
        comp
    }

    /// Sets the compression threshold in dB
    pub fn set_threshold(&mut self, threshold_db: Sample) {
        self.threshold_db = threshold_db;
    }

    /// Sets the compression ratio
    pub fn set_ratio(&mut self, ratio: Sample) {
        self.ratio = ratio.max(1.0);
    }

    /// Sets the attack time in milliseconds
    pub fn set_attack(&mut self, attack_ms: Sample) {
        self.attack_coeff = time_constant(attack_ms, self.sample_rate);
    }

    /// Sets the release time in milliseconds
    pub fn set_release(&mut self, release_ms: Sample) {
        self.release_coeff = time_constant(release_ms, self.sample_rate);
    }

    /// Sets the knee width in dB
    pub fn set_knee(&mut self, knee_db: Sample) {
        self.knee_width_db = knee_db.max(0.0);
    }

    /// Sets the makeup gain in dB
    pub fn set_makeup_gain(&mut self, gain_db: Sample) {
        self.makeup_gain = db_to_linear(gain_db);
    }

    #[inline]
    fn compute_gain(&self, input_db: Sample) -> Sample {
        let half_knee = self.knee_width_db / 2.0;
        let knee_start = self.threshold_db - half_knee;
        let knee_end = self.threshold_db + half_knee;

        let output_db = if input_db < knee_start {
            // Below knee - no compression
            input_db
        } else if input_db > knee_end {
            // Above knee - full compression
            self.threshold_db + (input_db - self.threshold_db) / self.ratio
        } else {
            // In knee - soft transition
            let x = input_db - knee_start;
            let knee_factor = x * x / (2.0 * self.knee_width_db);
            input_db - knee_factor * (1.0 - 1.0 / self.ratio)
        };

        output_db - input_db
    }

    /// Processes a single sample through the compressor
    #[inline]
    pub fn process(&mut self, input: Sample) -> Sample {
        let input_abs = input.abs();

        let coeff = if input_abs > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        self.envelope = coeff * self.envelope + (1.0 - coeff) * input_abs;

        // Convert to dB and compute gain reduction
        let input_db = linear_to_db(self.envelope);
        let gain_db = self.compute_gain(input_db);
        let gain = db_to_linear(gain_db);

        // Apply gain and makeup
        input * gain * self.makeup_gain
    }

    /// Processes a block of samples in-place
    pub fn process_block(&mut self, samples: &mut [Sample]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    /// Resets compressor state
    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }

    /// Gets current gain reduction in dB
    #[must_use]
    pub fn gain_reduction_db(&self) -> Sample {
        let input_db = linear_to_db(self.envelope);
        -self.compute_gain(input_db)
    }

    /// Gets current envelope level
    #[must_use]
    pub fn envelope(&self) -> Sample {
        self.envelope
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressor_creation() {
        let comp = Compressor::new(48000.0);
        assert!((comp.envelope() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compressor_below_threshold() {
        let mut comp = Compressor::new(48000.0);
        comp.set_threshold(-10.0);
        comp.set_ratio(4.0);
        comp.set_knee(0.0);

        // Process quiet signal (below threshold)
        let input = 0.1; // About -20 dB
        let mut output = input;
        for _ in 0..1000 {
            output = comp.process(input);
        }

        // Should pass through with minimal change
        assert!((output - input).abs() < 0.05);
    }

    #[test]
    fn test_compressor_above_threshold() {
        let mut comp = Compressor::new(48000.0);
        comp.set_threshold(-20.0);
        comp.set_ratio(4.0);
        comp.set_knee(0.0);
        comp.set_makeup_gain(0.0);

        // Process loud signal (above threshold)
        let input = 1.0; // 0 dB
        let mut output = input;
        for _ in 0..10000 {
            output = comp.process(input);
        }

        // Should be compressed
        assert!(output < input);
    }

    #[test]
    fn test_limiter() {
        let mut limiter = Limiter::new(48000.0);
        limiter.set_ceiling(-3.0);

        // Process signal that exceeds ceiling
        let input = 1.5;
        let mut output = input;
        for _ in 0..10000 {
            output = limiter.process(input);
        }

        // Should be limited
        assert!(output < input);
    }
}
