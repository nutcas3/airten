use crate::Sample;
use crate::dsp::{db_to_linear, linear_to_db, time_constant};

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

    pub fn set_threshold(&mut self, threshold_db: Sample) {
        self.threshold_db = threshold_db;
    }

    pub fn set_ratio(&mut self, ratio: Sample) {
        self.ratio = ratio.max(1.0);
    }

    pub fn set_attack(&mut self, attack_ms: Sample) {
        self.attack_coeff = time_constant(attack_ms, self.sample_rate);
    }

    pub fn set_release(&mut self, release_ms: Sample) {
        self.release_coeff = time_constant(release_ms, self.sample_rate);
    }

    pub fn set_knee(&mut self, knee_db: Sample) {
        self.knee_width_db = knee_db.max(0.0);
    }

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

    pub fn process_block(&mut self, samples: &mut [Sample]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }

    pub fn gain_reduction_db(&self) -> Sample {
        let input_db = linear_to_db(self.envelope);
        -self.compute_gain(input_db)
    }

    pub fn envelope(&self) -> Sample {
        self.envelope
    }
}

/// Limiter - compressor with infinite ratio
pub struct Limiter {
    compressor: Compressor,
    lookahead_buffer: [Sample; 256],
    lookahead_pos: usize,
    lookahead_samples: usize,
}

impl Limiter {
    pub fn new(sample_rate: Sample) -> Self {
        let mut compressor = Compressor::new(sample_rate);
        compressor.set_ratio(100.0); // Near-infinite ratio
        compressor.set_attack(0.1);
        compressor.set_release(50.0);
        compressor.set_knee(0.0); // Hard knee for limiting
        
        Self {
            compressor,
            lookahead_buffer: [0.0; 256],
            lookahead_pos: 0,
            lookahead_samples: 0,
        }
    }

    pub fn set_ceiling(&mut self, ceiling_db: Sample) {
        self.compressor.set_threshold(ceiling_db);
    }

    pub fn process(&mut self, input: Sample) -> Sample {
        if self.lookahead_samples == 0 {
            return self.compressor.process(input);
        }

        // Store input in lookahead buffer
        let output_pos = (self.lookahead_pos + 256 - self.lookahead_samples) % 256;
        let delayed = self.lookahead_buffer[output_pos];
        self.lookahead_buffer[self.lookahead_pos] = input;
        self.lookahead_pos = (self.lookahead_pos + 1) % 256;

        // Process with lookahead
        self.compressor.process(delayed)
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
