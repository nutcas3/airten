use crate::{MAX_CHANNELS, MAX_FRAME_SIZE, Sample};

/// Multi-channel audio frame for sample storage and processing
/// 
/// This provides a fixed-size audio frame suitable for real-time audio processing.
/// The large stack arrays are intentional for performance in audio processing.
#[repr(C)]
#[derive(Clone)]
pub struct AudioFrame {
    samples: [[Sample; MAX_FRAME_SIZE]; MAX_CHANNELS],
    num_samples: usize,
    num_channels: usize,
    sample_rate: u32,
}

impl AudioFrame {
    /// Creates a new audio frame with default settings
    #[must_use]
    pub const fn new() -> Self {
        Self {
            samples: [[0.0; MAX_FRAME_SIZE]; MAX_CHANNELS],
            num_samples: 0,
            num_channels: 1,
            sample_rate: 48000,
        }
    }

    /// Creates a new audio frame with the specified configuration
    #[must_use]
    pub fn with_config(num_samples: usize, num_channels: usize, sample_rate: u32) -> Self {
        debug_assert!(num_samples <= MAX_FRAME_SIZE);
        debug_assert!(num_channels <= MAX_CHANNELS);

        Self {
            samples: [[0.0; MAX_FRAME_SIZE]; MAX_CHANNELS],
            num_samples: num_samples.min(MAX_FRAME_SIZE),
            num_channels: num_channels.min(MAX_CHANNELS),
            sample_rate,
        }
    }

    /// Returns the number of samples per channel
    #[inline]
    #[must_use]
    pub fn num_samples(&self) -> usize {
        self.num_samples
    }

    /// Returns the number of audio channels
    #[inline]
    #[must_use]
    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    /// Returns the sample rate in Hz
    #[inline]
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Gets an immutable slice to the specified channel's samples
    #[inline]
    #[must_use]
    pub fn channel(&self, index: usize) -> Option<&[Sample]> {
        if index < self.num_channels {
            Some(&self.samples[index][..self.num_samples])
        } else {
            None
        }
    }

    /// Gets a mutable slice to the specified channel's samples
    #[inline]
    pub fn channel_mut(&mut self, index: usize) -> Option<&mut [Sample]> {
        if index < self.num_channels {
            Some(&mut self.samples[index][..self.num_samples])
        } else {
            None
        }
    }

    /// Copies samples from a slice to the specified channel
    pub fn copy_from_slice(&mut self, channel: usize, src: &[Sample]) {
        if channel < self.num_channels {
            let len = src.len().min(MAX_FRAME_SIZE);
            self.samples[channel][..len].copy_from_slice(&src[..len]);
            self.num_samples = self.num_samples.max(len);
        }
    }

    /// Copies samples from the specified channel to a slice
    pub fn copy_to_slice(&self, channel: usize, dst: &mut [Sample]) {
        if channel < self.num_channels {
            let len = dst.len().min(self.num_samples);
            dst[..len].copy_from_slice(&self.samples[channel][..len]);
        }
    }

    /// Copies samples from interleaved data into this frame
    pub fn from_interleaved(&mut self, interleaved: &[Sample]) {
        let total_samples = interleaved.len() / self.num_channels;
        self.num_samples = total_samples.min(MAX_FRAME_SIZE);

        for (i, sample) in interleaved.iter().enumerate() {
            let channel = i % self.num_channels;
            let frame_idx = i / self.num_channels;
            if frame_idx < self.num_samples {
                self.samples[channel][frame_idx] = *sample;
            }
        }
    }

    /// Copies samples from this frame to interleaved data
    pub fn to_interleaved(&self, interleaved: &mut [Sample]) {
        for i in 0..self.num_samples {
            for ch in 0..self.num_channels {
                let idx = i * self.num_channels + ch;
                if idx < interleaved.len() {
                    interleaved[idx] = self.samples[ch][i];
                }
            }
        }
    }

    /// Clears all samples in the frame to zero
    pub fn clear(&mut self) {
        for ch in 0..self.num_channels {
            for sample in &mut self.samples[ch][..self.num_samples] {
                *sample = 0.0;
            }
        }
    }

    pub fn apply_gain(&mut self, gain: Sample) {
        for ch in 0..self.num_channels {
            for sample in &mut self.samples[ch][..self.num_samples] {
                *sample *= gain;
            }
        }
    }

    pub fn peak(&self) -> Sample {
        let mut peak = 0.0f32;
        for ch in 0..self.num_channels {
            for &sample in &self.samples[ch][..self.num_samples] {
                let abs = sample.abs();
                if abs > peak {
                    peak = abs;
                }
            }
        }
        peak
    }

    pub fn rms(&self) -> Sample {
        if self.num_samples == 0 {
            return 0.0;
        }

        let mut sum_sq = 0.0f32;
        let mut count = 0usize;

        for ch in 0..self.num_channels {
            for &sample in &self.samples[ch][..self.num_samples] {
                sum_sq += sample * sample;
                count += 1;
            }
        }

        if count > 0 {
            (sum_sq as f64 / count as f64).sqrt() as Sample
        } else {
            0.0
        }
    }
}

impl Default for AudioFrame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        let frame = AudioFrame::new();
        assert_eq!(frame.num_samples(), 0);
        assert_eq!(frame.num_channels(), 1);
        assert_eq!(frame.sample_rate(), 48000);
    }

    #[test]
    fn test_frame_with_config() {
        let frame = AudioFrame::with_config(256, 2, 44100);
        assert_eq!(frame.num_samples(), 256);
        assert_eq!(frame.num_channels(), 2);
        assert_eq!(frame.sample_rate(), 44100);
    }

    #[test]
    fn test_channel_access() {
        let mut frame = AudioFrame::with_config(128, 2, 48000);

        if let Some(ch) = frame.channel_mut(0) {
            ch[0] = 0.5;
            ch[1] = -0.5;
        }

        assert_eq!(frame.channel(0).unwrap()[0], 0.5);
        assert_eq!(frame.channel(0).unwrap()[1], -0.5);
        assert!(frame.channel(2).is_none());
    }

    #[test]
    fn test_interleaved_conversion() {
        let mut frame = AudioFrame::with_config(4, 2, 48000);
        let interleaved = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];

        frame.from_interleaved(&interleaved);

        assert_eq!(frame.channel(0).unwrap()[0], 0.1);
        assert_eq!(frame.channel(1).unwrap()[0], 0.2);
        assert_eq!(frame.channel(0).unwrap()[1], 0.3);
        assert_eq!(frame.channel(1).unwrap()[1], 0.4);
    }

    #[test]
    fn test_peak_and_rms() {
        let mut frame = AudioFrame::with_config(4, 1, 48000);
        frame.copy_from_slice(0, &[0.5, -1.0, 0.25, 0.75]);

        assert!((frame.peak() - 1.0).abs() < 0.001);

        let expected_rms = ((0.25 + 1.0 + 0.0625 + 0.5625) / 4.0f32).sqrt();
        assert!((frame.rms() - expected_rms).abs() < 0.001);
    }
}
