use crate::Sample;

pub const SAMPLE_RATES: &[u32] = &[8000, 16000, 22050, 44100, 48000, 96000];
pub const FRAME_SIZES: &[usize] = &[64, 128, 256, 512, 1024, 2048];

pub struct AudioFixture {
    pub sample_rate: u32,
    pub samples: Vec<Sample>,
    pub description: String,
}

impl AudioFixture {
    pub fn new(sample_rate: u32, samples: Vec<Sample>, description: String) -> Self {
        Self {
            sample_rate,
            samples,
            description,
        }
    }

    pub fn sine(frequency: f32, sample_rate: u32, duration_secs: f32) -> Self {
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let samples = crate::generate_sine(frequency, sample_rate, num_samples);
        
        Self {
            sample_rate,
            samples,
            description: format!("Sine wave: {} Hz, {} seconds", frequency, duration_secs),
        }
    }

    pub fn white_noise(sample_rate: u32, duration_secs: f32, amplitude: f32) -> Self {
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        let samples = crate::generate_white_noise(num_samples, amplitude);
        
        Self {
            sample_rate,
            samples,
            description: format!("White noise: {} seconds, amplitude {}", duration_secs, amplitude),
        }
    }

    pub fn chirp(start_freq: f32, end_freq: f32, sample_rate: u32, duration_secs: f32) -> Self {
        let samples = crate::generate_chirp(start_freq, end_freq, sample_rate, duration_secs);
        
        Self {
            sample_rate,
            samples,
            description: format!("Chirp: {}-{} Hz, {} seconds", start_freq, end_freq, duration_secs),
        }
    }

    pub fn duration(&self) -> f32 {
        self.samples.len() as f32 / self.sample_rate as f32
    }

    pub fn rms(&self) -> f32 {
        crate::calculate_rms(&self.samples)
    }

    pub fn peak(&self) -> f32 {
        crate::find_peak(&self.samples)
    }
}

pub struct TestFixtures {
    fixtures: Vec<AudioFixture>,
}

impl TestFixtures {
    pub fn standard() -> Self {
        let mut fixtures = Vec::new();

        for &freq in &[100.0, 440.0, 1000.0, 5000.0] {
            fixtures.push(AudioFixture::sine(freq, 48000, 1.0));
        }

        fixtures.push(AudioFixture::white_noise(48000, 1.0, 0.5));

        fixtures.push(AudioFixture::chirp(20.0, 20000.0, 48000, 2.0));

        fixtures.push(AudioFixture::new(
            48000,
            crate::generate_silence(48000),
            "Silence: 1 second".to_string(),
        ));

        fixtures.push(AudioFixture::new(
            48000,
            crate::generate_impulse(48000, 24000),
            "Impulse at center".to_string(),
        ));

        Self { fixtures }
    }

    pub fn all(&self) -> &[AudioFixture] {
        &self.fixtures
    }

    pub fn get(&self, index: usize) -> Option<&AudioFixture> {
        self.fixtures.get(index)
    }

    pub fn len(&self) -> usize {
        self.fixtures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fixtures.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_fixture() {
        let fixture = AudioFixture::sine(440.0, 48000, 1.0);
        assert_eq!(fixture.sample_rate, 48000);
        assert_eq!(fixture.samples.len(), 48000);
        assert!((fixture.duration() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_standard_fixtures() {
        let fixtures = TestFixtures::standard();
        assert!(fixtures.len() > 0);
        
        for fixture in fixtures.all() {
            assert!(!fixture.samples.is_empty());
            assert!(fixture.sample_rate > 0);
        }
    }
}
