use crate::Sample;
use crate::dsp::{db_to_linear, linear_to_db, time_constant};

pub struct NoiseGate {
    sample_rate: Sample,
    threshold_db: Sample,
    hysteresis_db: Sample,
    attack_coeff: Sample,
    hold_samples: usize,
    release_coeff: Sample,
    range_db: Sample,
    envelope: Sample,
    gain: Sample,
    hold_counter: usize,
    is_open: bool,
}

impl NoiseGate {
    pub fn new(sample_rate: Sample) -> Self {
        let mut gate = Self {
            sample_rate,
            threshold_db: -40.0,
            hysteresis_db: 3.0,
            attack_coeff: 0.0,
            hold_samples: 0,
            release_coeff: 0.0,
            range_db: -80.0,
            envelope: 0.0,
            gain: 0.0,
            hold_counter: 0,
            is_open: false,
        };
        gate.set_attack(1.0);
        gate.set_hold(50.0);
        gate.set_release(100.0);
        gate
    }

    pub fn set_threshold(&mut self, threshold_db: Sample) {
        self.threshold_db = threshold_db;
    }

    pub fn set_hysteresis(&mut self, hysteresis_db: Sample) {
        self.hysteresis_db = hysteresis_db.max(0.0);
    }

    pub fn set_attack(&mut self, attack_ms: Sample) {
        self.attack_coeff = time_constant(attack_ms, self.sample_rate);
    }

    pub fn set_hold(&mut self, hold_ms: Sample) {
        self.hold_samples = (hold_ms * 0.001 * self.sample_rate) as usize;
    }

    pub fn set_release(&mut self, release_ms: Sample) {
        self.release_coeff = time_constant(release_ms, self.sample_rate);
    }

    pub fn set_range(&mut self, range_db: Sample) {
        self.range_db = range_db.min(0.0);
    }

    #[inline]
    pub fn process(&mut self, input: Sample) -> Sample {
        let input_abs = input.abs();

        let env_coeff = if input_abs > self.envelope {
            0.0
        } else {
            0.9995
        };
        self.envelope = env_coeff * self.envelope + (1.0 - env_coeff) * input_abs;

        let input_db = linear_to_db(self.envelope);

        let open_threshold = self.threshold_db;
        let close_threshold = self.threshold_db - self.hysteresis_db;

        if !self.is_open && input_db > open_threshold {
            self.is_open = true;
            self.hold_counter = self.hold_samples;
        } else if self.is_open && input_db < close_threshold {
            if self.hold_counter > 0 {
                self.hold_counter -= 1;
            } else {
                self.is_open = false;
            }
        } else if self.is_open {
            self.hold_counter = self.hold_samples;
        }

        let target_gain = if self.is_open {
            1.0
        } else {
            db_to_linear(self.range_db)
        };

        let coeff = if target_gain > self.gain {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        self.gain = coeff * self.gain + (1.0 - coeff) * target_gain;

        input * self.gain
    }

    pub fn process_block(&mut self, samples: &mut [Sample]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.gain = 0.0;
        self.hold_counter = 0;
        self.is_open = false;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn gain(&self) -> Sample {
        self.gain
    }

    pub fn gain_reduction_db(&self) -> Sample {
        linear_to_db(self.gain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_creation() {
        let gate = NoiseGate::new(48000.0);
        assert!(!gate.is_open());
    }

    #[test]
    fn test_gate_opens_on_loud_signal() {
        let mut gate = NoiseGate::new(48000.0);
        gate.set_threshold(-30.0);

        for _ in 0..1000 {
            gate.process(0.5);
        }

        assert!(gate.is_open());
    }

    #[test]
    fn test_gate_closes_on_quiet_signal() {
        let mut gate = NoiseGate::new(48000.0);
        gate.set_threshold(-30.0);
        gate.set_hold(0.0);

        for _ in 0..1000 {
            gate.process(0.5);
        }
        assert!(gate.is_open());

        for _ in 0..10000 {
            gate.process(0.001);
        }
        assert!(!gate.is_open());
    }

    #[test]
    fn test_gate_attenuates_quiet_signal() {
        let mut gate = NoiseGate::new(48000.0);
        gate.set_threshold(-20.0);
        gate.set_range(-60.0);

        let input = 0.01;
        let mut output = input;
        for _ in 0..10000 {
            output = gate.process(input);
        }

        assert!(output < input * 0.1);
    }
}
