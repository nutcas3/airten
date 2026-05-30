#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{
    Sample,
    error::{Error, Result},
};

#[cfg(not(feature = "alloc"))]
use crate::{MAX_CHANNELS, MAX_FRAME_SIZE};

/// Multi-channel audio buffer for sample storage and processing
///
/// This provides a flexible audio buffer that can work with both heap-allocated
/// and stack-allocated memory depending on the `alloc` feature.
pub struct AudioBuffer {
    #[cfg(feature = "alloc")]
    data: Vec<Sample>,
    #[cfg(not(feature = "alloc"))]
    data: [Sample; MAX_FRAME_SIZE * MAX_CHANNELS],
    num_samples: usize,
    num_channels: usize,
    sample_rate: u32,
}

impl AudioBuffer {
    /// Creates a new audio buffer with the specified parameters (no_std version)
    ///
    /// # Arguments
    /// * `num_samples` - Number of samples per channel
    /// * `num_channels` - Number of audio channels  
    /// * `sample_rate` - Sample rate in Hz
    ///
    /// # Errors
    ///
    /// Returns `Error::BufferTooLarge` if num_samples exceeds MAX_FRAME_SIZE
    /// Returns `Error::InvalidChannelCount` if num_channels is 0 or exceeds MAX_CHANNELS
    #[cfg(not(feature = "alloc"))]
    pub fn new(num_samples: usize, num_channels: usize, sample_rate: u32) -> Result<Self> {
        if num_samples > MAX_FRAME_SIZE {
            return Err(Error::BufferTooLarge);
        }
        if num_channels > MAX_CHANNELS || num_channels == 0 {
            return Err(Error::InvalidChannelCount);
        }

        Ok(Self {
            data: [0.0; MAX_FRAME_SIZE * MAX_CHANNELS],
            num_samples,
            num_channels,
            sample_rate,
        })
    }

    /// Creates a new audio buffer with the specified parameters (alloc version)
    ///
    /// # Arguments
    /// * `num_samples` - Number of samples per channel
    /// * `num_channels` - Number of audio channels  
    /// * `sample_rate` - Sample rate in Hz
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidChannelCount` if `num_channels` is 0
    /// Returns `Error::AllocationFailed` if memory allocation fails
    #[cfg(feature = "alloc")]
    pub fn new(num_samples: usize, num_channels: usize, sample_rate: u32) -> Result<Self> {
        if num_channels == 0 {
            return Err(Error::InvalidChannelCount);
        }

        let size = num_samples * num_channels;
        let mut data = Vec::new();
        data.try_reserve(size)
            .map_err(|_| Error::AllocationFailed)?;
        data.resize(size, 0.0);

        Ok(Self {
            data,
            num_samples,
            num_channels,
            sample_rate,
        })
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

    /// Returns the total number of samples across all channels
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.num_samples * self.num_channels
    }

    /// Returns true if the buffer contains no samples
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.num_samples == 0
    }

    /// Gets a sample from the specified channel and frame
    ///
    /// Returns None if channel or frame indices are out of bounds
    #[inline]
    #[must_use]
    pub fn get(&self, channel: usize, frame: usize) -> Option<Sample> {
        if channel < self.num_channels && frame < self.num_samples {
            Some(self.data[channel * self.num_samples + frame])
        } else {
            None
        }
    }

    /// Sets a sample at the specified channel and frame
    ///
    /// Does nothing if channel or frame indices are out of bounds
    #[inline]
    pub fn set(&mut self, channel: usize, frame: usize, value: Sample) {
        if channel < self.num_channels && frame < self.num_samples {
            self.data[channel * self.num_samples + frame] = value;
        }
    }

    /// Gets an immutable slice to the specified channel's samples
    ///
    /// Returns None if channel index is out of bounds
    #[must_use]
    pub fn channel(&self, index: usize) -> Option<&[Sample]> {
        if index < self.num_channels {
            let start = index * self.num_samples;
            let end = start + self.num_samples;
            Some(&self.data[start..end])
        } else {
            None
        }
    }

    /// Gets a mutable slice to the specified channel's samples
    ///
    /// Returns None if channel index is out of bounds
    pub fn channel_mut(&mut self, index: usize) -> Option<&mut [Sample]> {
        if index < self.num_channels {
            let start = index * self.num_samples;
            let end = start + self.num_samples;
            Some(&mut self.data[start..end])
        } else {
            None
        }
    }

    /// Gets an immutable slice to all samples in interleaved format
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[Sample] {
        &self.data[..self.len()]
    }

    /// Gets a mutable slice to all samples in interleaved format
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Sample] {
        let len = self.len();
        &mut self.data[..len]
    }

    /// Clears all samples in the buffer to zero
    pub fn clear(&mut self) {
        let len = self.len();
        for sample in &mut self.data[..len] {
            *sample = 0.0;
        }
    }

    /// Copies samples from interleaved data into this buffer
    ///
    /// The interleaved data should be in channel-major order: \[ch0\[0\], ch1\[0\], ch0\[1\], ch1\[1\], ...\]
    pub fn from_interleaved(&mut self, interleaved: &[Sample]) {
        let frames = interleaved.len() / self.num_channels;
        let frames = frames.min(self.num_samples);

        for frame in 0..frames {
            for ch in 0..self.num_channels {
                let src_idx = frame * self.num_channels + ch;
                let dst_idx = ch * self.num_samples + frame;
                if src_idx < interleaved.len() {
                    self.data[dst_idx] = interleaved[src_idx];
                }
            }
        }
    }

    /// Copies samples from this buffer to interleaved data
    ///
    /// The interleaved output will be in channel-major order: \[ch0\[0\], ch1\[0\], ch0\[1\], ch1\[1\], ...\]
    pub fn to_interleaved(&self, interleaved: &mut [Sample]) {
        let frames = interleaved.len() / self.num_channels;
        let frames = frames.min(self.num_samples);

        for frame in 0..frames {
            for ch in 0..self.num_channels {
                let src_idx = ch * self.num_samples + frame;
                let dst_idx = frame * self.num_channels + ch;
                if dst_idx < interleaved.len() {
                    interleaved[dst_idx] = self.data[src_idx];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = AudioBuffer::new(256, 2, 48000).unwrap();
        assert_eq!(buffer.num_samples(), 256);
        assert_eq!(buffer.num_channels(), 2);
        assert_eq!(buffer.sample_rate(), 48000);
        assert_eq!(buffer.len(), 512);
    }

    #[test]
    fn test_channel_access() {
        let mut buffer = AudioBuffer::new(128, 2, 48000).unwrap();

        buffer.set(0, 0, 0.5);
        buffer.set(1, 0, -0.5);

        assert_eq!(buffer.get(0, 0), Some(0.5));
        assert_eq!(buffer.get(1, 0), Some(-0.5));
        assert_eq!(buffer.get(2, 0), None);
    }

    #[test]
    fn test_interleaved() {
        let mut buffer = AudioBuffer::new(4, 2, 48000).unwrap();
        let interleaved = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];

        buffer.from_interleaved(&interleaved);

        assert!((buffer.get(0, 0).unwrap() - 0.1).abs() < 0.001);
        assert!((buffer.get(1, 0).unwrap() - 0.2).abs() < 0.001);
        assert!((buffer.get(0, 1).unwrap() - 0.3).abs() < 0.001);
    }
}
