use crate::Sample;
use crate::dsp::constants::TWO_PI;

/// Types of biquad filters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    /// Low-pass filter (attenuates frequencies above cutoff)
    LowPass,
    /// High-pass filter (attenuates frequencies below cutoff)
    HighPass,
    /// Band-pass filter (passes frequencies within a band)
    BandPass,
    /// Notch filter (attenuates frequencies within a narrow band)
    Notch,
    /// All-pass filter (passes all frequencies with phase shift)
    AllPass,
    /// Peaking filter (boosts or cuts frequencies within a band)
    Peaking,
    /// Low-shelf filter (boosts or cuts low frequencies)
    LowShelf,
    /// High-shelf filter (boosts or cuts high frequencies)
    HighShelf,
}

/// Biquad filter coefficients in direct form I
#[derive(Debug, Clone, Copy)]
pub struct FilterCoefficients {
    /// Feed-forward coefficient for current input
    pub b0: Sample,
    /// Feed-forward coefficient for previous input
    pub b1: Sample,
    /// Feed-forward coefficient for second previous input
    pub b2: Sample,
    /// Feed-back coefficient for previous output
    pub a1: Sample,
    /// Feed-back coefficient for second previous output
    pub a2: Sample,
}

impl Default for FilterCoefficients {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }
}

/// Biquad filter implementation with direct form I processing
#[derive(Clone, Copy)]
pub struct BiquadFilter {
    coeffs: FilterCoefficients,
    z1: Sample,
    z2: Sample,
}

impl BiquadFilter {
    /// Creates a new biquad filter with the given coefficients
    #[must_use]
    pub fn new(coeffs: FilterCoefficients) -> Self {
        Self {
            coeffs,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// Creates a bypass filter (no processing)
    #[must_use]
    pub fn bypass() -> Self {
        Self::new(FilterCoefficients::default())
    }

    /// Creates a low-pass filter
    #[must_use]
    pub fn lowpass(sample_rate: Sample, cutoff: Sample, q: Sample) -> Self {
        let omega = TWO_PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = (1.0 - cos_omega) / 2.0;
        let b1 = 1.0 - cos_omega;
        let b2 = (1.0 - cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self::new(FilterCoefficients {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        })
    }

    /// Creates a high-pass filter
    #[must_use]
    pub fn highpass(sample_rate: Sample, cutoff: Sample, q: Sample) -> Self {
        let omega = TWO_PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = (1.0 + cos_omega) / 2.0;
        let b1 = -(1.0 + cos_omega);
        let b2 = (1.0 + cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self::new(FilterCoefficients {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        })
    }

    /// Creates a band-pass filter
    #[must_use]
    pub fn bandpass(sample_rate: Sample, center: Sample, q: Sample) -> Self {
        let omega = TWO_PI * center / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self::new(FilterCoefficients {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        })
    }

    /// Creates a notch filter
    #[must_use]
    pub fn notch(sample_rate: Sample, center: Sample, q: Sample) -> Self {
        let omega = TWO_PI * center / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self::new(FilterCoefficients {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        })
    }

    /// Creates a peaking filter
    #[must_use]
    pub fn peaking(sample_rate: Sample, center: Sample, q: Sample, gain_db: Sample) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = TWO_PI * center / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / a;

        Self::new(FilterCoefficients {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        })
    }

    /// Creates a low-shelf filter
    #[must_use]
    pub fn low_shelf(sample_rate: Sample, cutoff: Sample, gain_db: Sample) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = TWO_PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / 2.0 * ((a + 1.0 / a) * (1.0 / 0.707 - 1.0) + 2.0).sqrt();
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_omega + two_sqrt_a_alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_omega - two_sqrt_a_alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_omega + two_sqrt_a_alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega);
        let a2 = (a + 1.0) + (a - 1.0) * cos_omega - two_sqrt_a_alpha;

        Self::new(FilterCoefficients {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        })
    }

    /// Creates a high-shelf filter
    #[must_use]
    pub fn high_shelf(sample_rate: Sample, cutoff: Sample, gain_db: Sample) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = TWO_PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / 2.0 * ((a + 1.0 / a) * (1.0 / 0.707 - 1.0) + 2.0).sqrt();
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_omega + two_sqrt_a_alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_omega);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_omega - two_sqrt_a_alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_omega + two_sqrt_a_alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_omega);
        let a2 = (a + 1.0) - (a - 1.0) * cos_omega - two_sqrt_a_alpha;

        Self::new(FilterCoefficients {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        })
    }

    /// Processes a single sample through the filter
    #[inline]
    pub fn process(&mut self, input: Sample) -> Sample {
        let output = self.coeffs.b0 * input + self.z1;
        self.z1 = self.coeffs.b1 * input - self.coeffs.a1 * output + self.z2;
        self.z2 = self.coeffs.b2 * input - self.coeffs.a2 * output;
        output
    }

    /// Processes a block of samples in-place
    pub fn process_block(&mut self, samples: &mut [Sample]) {
        for sample in samples.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    /// Resets filter state (clears delay lines)
    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    /// Gets the current filter coefficients
    #[must_use]
    pub fn coefficients(&self) -> &FilterCoefficients {
        &self.coeffs
    }

    /// Sets new filter coefficients
    pub fn set_coefficients(&mut self, coeffs: FilterCoefficients) {
        self.coeffs = coeffs;
    }
}

impl Default for BiquadFilter {
    fn default() -> Self {
        Self::bypass()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowpass_filter() {
        let mut filter = BiquadFilter::lowpass(48000.0, 1000.0, 0.707);

        let mut output = 0.0;
        for _ in 0..100 {
            output = filter.process(1.0);
        }

        assert!((output - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_highpass_filter() {
        let mut filter = BiquadFilter::highpass(48000.0, 1000.0, 0.707);

        let mut output = 0.0;
        for _ in 0..1000 {
            output = filter.process(1.0);
        }

        assert!(output.abs() < 0.01);
    }

    #[test]
    fn test_filter_reset() {
        let mut filter = BiquadFilter::lowpass(48000.0, 1000.0, 0.707);

        for _ in 0..100 {
            filter.process(1.0);
        }

        filter.reset();

        let output = filter.process(1.0);
        assert!((output - filter.coefficients().b0).abs() < 0.001);
    }
}
