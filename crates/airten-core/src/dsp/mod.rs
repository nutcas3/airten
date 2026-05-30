/// Digital Signal Processing module for audio effects and processing
mod compressor;
mod envelope;
mod filter;
mod gate;

pub use compressor::Compressor;
pub use envelope::EnvelopeFollower;
pub use filter::{BiquadFilter, FilterCoefficients, FilterType};
pub use gate::NoiseGate;

/// Mathematical constants for DSP operations
pub mod constants {
    use crate::Sample;

    /// Pi constant (π)
    pub const PI: Sample = core::f32::consts::PI;

    /// 2 * Pi constant (2π)
    pub const TWO_PI: Sample = 2.0 * PI;

    /// Natural logarithm of 2
    pub const LN_2: Sample = core::f32::consts::LN_2;

    /// Conversion factor from decibels to linear amplitude (ln(10) / 20)
    pub const DB_TO_LINEAR: Sample = 0.115_129_25;

    /// Conversion factor from linear amplitude to decibels (20 / ln(10))
    pub const LINEAR_TO_DB: Sample = 8.685_889_6;

    /// Minimum decibel level (floor for calculations)
    pub const MIN_DB: Sample = -120.0;

    /// Minimum linear amplitude (floor for calculations)
    pub const MIN_LINEAR: Sample = 1e-6;
}

/// Convert decibels to linear amplitude
#[inline]
pub fn db_to_linear(db: crate::Sample) -> crate::Sample {
    10.0_f32.powf(db / 20.0)
}

/// Convert linear amplitude to decibels
#[inline]
pub fn linear_to_db(linear: crate::Sample) -> crate::Sample {
    if linear <= constants::MIN_LINEAR {
        constants::MIN_DB
    } else {
        20.0 * linear.log10()
    }
}

/// Apply soft clipping to prevent harsh distortion
#[inline]
pub fn soft_clip(x: crate::Sample) -> crate::Sample {
    x.tanh()
}

/// Apply hard clipping with a specified threshold
#[inline]
pub fn hard_clip(x: crate::Sample, threshold: crate::Sample) -> crate::Sample {
    x.clamp(-threshold, threshold)
}

/// Convert time in milliseconds to filter time constant
#[inline]
pub fn time_constant(time_ms: crate::Sample, sample_rate: crate::Sample) -> crate::Sample {
    if time_ms <= 0.0 {
        0.0
    } else {
        (-1.0 / (time_ms * 0.001 * sample_rate)).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_conversion() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 0.001);
        assert!((db_to_linear(-6.0) - 0.501).abs() < 0.01);
        assert!((db_to_linear(6.0) - 1.995).abs() < 0.01);
    }

    #[test]
    fn test_linear_to_db() {
        assert!((linear_to_db(1.0) - 0.0).abs() < 0.001);
        assert!((linear_to_db(0.5) - (-6.02)).abs() < 0.1);
    }

    #[test]
    fn test_soft_clip() {
        assert!((soft_clip(0.0) - 0.0).abs() < 0.001);
        assert!(soft_clip(10.0) <= 1.0);
        assert!(soft_clip(-10.0) >= -1.0);
    }
}
