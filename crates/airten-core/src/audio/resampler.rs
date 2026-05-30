use crate::Sample;

pub struct Resampler {
    ratio: Sample,
    phase: Sample,
    last_sample: Sample,
}

impl Resampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Self {
        Self {
            ratio: input_rate as Sample / output_rate as Sample,
            phase: 0.0,
            last_sample: 0.0,
        }
    }

    #[inline]
    pub fn ratio(&self) -> Sample {
        self.ratio
    }

    pub fn output_size(&self, input_size: usize) -> usize {
        ((input_size as Sample) / self.ratio).ceil() as usize
    }

    pub fn input_size(&self, output_size: usize) -> usize {
        ((output_size as Sample) * self.ratio).ceil() as usize
    }

    pub fn process(&mut self, input: &[Sample], output: &mut [Sample]) -> usize {
        if input.is_empty() {
            return 0;
        }

        let mut out_idx = 0;
        
        while out_idx < output.len() {
            let idx = self.phase as usize;
            
            if idx >= input.len() {
                break;
            }
            
            let frac = self.phase - idx as Sample;
   
            let current = input[idx];
            let next = if idx + 1 < input.len() {
                input[idx + 1]
            } else {
                current
            };
            
            output[out_idx] = current + frac * (next - current);
            
            self.phase += self.ratio;
            out_idx += 1;
        }
        
        // Wrap phase and save last sample for continuity
        if !input.is_empty() {
            self.phase -= input.len() as Sample;
            self.last_sample = *input.last().unwrap_or(&0.0);
        }
        
        out_idx
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.last_sample = 0.0;
    }

    pub fn set_ratio(&mut self, input_rate: u32, output_rate: u32) {
        self.ratio = input_rate as Sample / output_rate as Sample;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resampler_creation() {
        let resampler = Resampler::new(48000, 44100);
        assert!((resampler.ratio() - 48000.0 / 44100.0).abs() < 0.001);
    }

    #[test]
    fn test_resampler_downsample() {
        let mut resampler = Resampler::new(48000, 24000);
        
        let input: [Sample; 10] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
        let mut output = [0.0; 5];
        
        let written = resampler.process(&input, &mut output);
        assert_eq!(written, 5);
    }

    #[test]
    fn test_resampler_upsample() {
        let mut resampler = Resampler::new(24000, 48000);
        
        let input: [Sample; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
        let mut output = [0.0; 10];
        
        let written = resampler.process(&input, &mut output);
        assert!(written > 5);
    }

    #[test]
    fn test_output_size_calculation() {
        let resampler = Resampler::new(48000, 16000);
        assert_eq!(resampler.output_size(300), 100);
    }
}
