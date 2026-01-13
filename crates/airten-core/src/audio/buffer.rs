#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{Sample, MAX_FRAME_SIZE, MAX_CHANNELS, error::{Error, Result}};

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

    #[cfg(feature = "alloc")]
    pub fn new(num_samples: usize, num_channels: usize, sample_rate: u32) -> Result<Self> {
        if num_channels == 0 {
            return Err(Error::InvalidChannelCount);
        }
        
        let size = num_samples * num_channels;
        let mut data = Vec::new();
        data.try_reserve(size).map_err(|_| Error::AllocationFailed)?;
        data.resize(size, 0.0);
        
        Ok(Self {
            data,
            num_samples,
            num_channels,
            sample_rate,
        })
    }

    #[inline]
    pub fn num_samples(&self) -> usize {
        self.num_samples
    }

    #[inline]
    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    #[inline]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.num_samples * self.num_channels
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.num_samples == 0
    }

    #[inline]
    pub fn get(&self, channel: usize, frame: usize) -> Option<Sample> {
        if channel < self.num_channels && frame < self.num_samples {
            Some(self.data[channel * self.num_samples + frame])
        } else {
            None
        }
    }

    #[inline]
    pub fn set(&mut self, channel: usize, frame: usize, value: Sample) {
        if channel < self.num_channels && frame < self.num_samples {
            self.data[channel * self.num_samples + frame] = value;
        }
    }

    pub fn channel(&self, index: usize) -> Option<&[Sample]> {
        if index < self.num_channels {
            let start = index * self.num_samples;
            let end = start + self.num_samples;
            Some(&self.data[start..end])
        } else {
            None
        }
    }

    pub fn channel_mut(&mut self, index: usize) -> Option<&mut [Sample]> {
        if index < self.num_channels {
            let start = index * self.num_samples;
            let end = start + self.num_samples;
            Some(&mut self.data[start..end])
        } else {
            None
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[Sample] {
        &self.data[..self.len()]
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Sample] {
        let len = self.len();
        &mut self.data[..len]
    }

    pub fn clear(&mut self) {
        for sample in self.data[..self.len()].iter_mut() {
            *sample = 0.0;
        }
    }

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
