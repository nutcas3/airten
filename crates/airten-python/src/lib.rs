#[cfg(feature = "python-bindings")]
use numpy::{PyArray1, PyArrayMethods, PyReadonlyArray1};
#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;
#[cfg(feature = "python-bindings")]
use pyo3::exceptions::{PyValueError, PyRuntimeError};
#[cfg(feature = "python-bindings")]
use pyo3::wrap_pyfunction;
#[cfg(feature = "python-bindings")]
use pyo3::types::PyModule;

#[cfg(feature = "python-bindings")]
use airten_core::{
    AudioProcessor as CoreProcessor,
    ProcessorConfig,
};
#[cfg(feature = "python-bindings")]
use airten_core::dsp::{BiquadFilter, Compressor, NoiseGate};
#[cfg(feature = "python-bindings")]
use airten_core::audio::Resampler as CoreResampler;

#[cfg(feature = "python-bindings")]
#[pyclass]
pub struct AudioProcessor {
    inner: CoreProcessor,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl AudioProcessor {
    /// Create a new audio processor
    ///
    /// Args:
    ///     sample_rate: Sample rate in Hz (default: 48000)
    ///     frame_size: Frame size in samples (default: 512)
    ///     channels: Number of channels (default: 1)
    ///     noise_suppression: Enable noise suppression (default: True)
    ///     compression: Enable dynamic range compression (default: True)
    #[new]
    #[pyo3(signature = (sample_rate=48000, frame_size=512, channels=1, noise_suppression=true, compression=true))]
    fn new(
        sample_rate: u32,
        frame_size: usize,
        channels: usize,
        noise_suppression: bool,
        compression: bool,
    ) -> PyResult<Self> {
        let config = ProcessorConfig {
            sample_rate,
            frame_size,
            num_channels: channels,
            noise_suppression,
            compression,
            target_latency_ms: 10.0,
        };
        Ok(Self {
            inner: CoreProcessor::new(config),
        })
    }

    /// Process audio samples in-place
    ///
    /// Args:
    ///     samples: NumPy array of float32 samples (modified in-place)
    fn process_inplace<'py>(&mut self, samples: &Bound<'py, PyArray1<f32>>) -> PyResult<()> {
        let mut samples_rw = unsafe { samples.as_array_mut() };
        let slice = samples_rw.as_slice_mut()
            .ok_or_else(|| PyValueError::new_err("Array must be contiguous"))?;
        
        self.inner.process(slice)
            .map_err(|e| PyRuntimeError::new_err(format!("Processing error: {}", e)))
    }

    /// Process audio samples and return a new array
    ///
    /// Args:
    ///     samples: NumPy array of float32 samples
    ///
    /// Returns:
    ///     Processed audio as a new NumPy array
    fn process<'py>(
        &mut self,
        py: Python<'py>,
        samples: PyReadonlyArray1<'py, f32>,
    ) -> PyResult<Bound<'py, PyArray1<f32>>> {
        let input = samples.as_slice()
            .map_err(|_| PyValueError::new_err("Array must be contiguous"))?;
        
        let mut output: Vec<f32> = input.to_vec();
        
        self.inner.process(&mut output)
            .map_err(|e| PyRuntimeError::new_err(format!("Processing error: {}", e)))?;
        
        Ok(PyArray1::from_vec(py, output))
    }

    /// Reset processor state
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Get the current envelope level
    #[getter]
    fn envelope_level(&self) -> f32 {
        self.inner.envelope_level()
    }

    /// Get the sample rate
    #[getter]
    fn sample_rate(&self) -> u32 {
        self.inner.config().sample_rate
    }

    /// Get the frame size
    #[getter]
    fn frame_size(&self) -> usize {
        self.inner.config().frame_size
    }
}

/// Biquad filter for audio processing
#[cfg(feature = "python-bindings")]
#[pyclass]
pub struct Filter {
    inner: BiquadFilter,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Filter {
    /// Create a lowpass filter
    ///
    /// Args:
    ///     sample_rate: Sample rate in Hz
    ///     cutoff: Cutoff frequency in Hz
    ///     q: Q factor (default: 0.707 for Butterworth)
    #[staticmethod]
    #[pyo3(signature = (sample_rate, cutoff, q=0.707))]
    fn lowpass(sample_rate: f32, cutoff: f32, q: f32) -> Self {
        Self {
            inner: BiquadFilter::lowpass(sample_rate, cutoff, q),
        }
    }

    /// Create a highpass filter
    #[staticmethod]
    #[pyo3(signature = (sample_rate, cutoff, q=0.707))]
    fn highpass(sample_rate: f32, cutoff: f32, q: f32) -> Self {
        Self {
            inner: BiquadFilter::highpass(sample_rate, cutoff, q),
        }
    }

    /// Create a bandpass filter
    #[staticmethod]
    #[pyo3(signature = (sample_rate, center, q=1.0))]
    fn bandpass(sample_rate: f32, center: f32, q: f32) -> Self {
        Self {
            inner: BiquadFilter::bandpass(sample_rate, center, q),
        }
    }

    /// Create a notch filter
    #[staticmethod]
    #[pyo3(signature = (sample_rate, center, q=1.0))]
    fn notch(sample_rate: f32, center: f32, q: f32) -> Self {
        Self {
            inner: BiquadFilter::notch(sample_rate, center, q),
        }
    }

    /// Create a peaking EQ filter
    #[staticmethod]
    fn peaking(sample_rate: f32, center: f32, q: f32, gain_db: f32) -> Self {
        Self {
            inner: BiquadFilter::peaking(sample_rate, center, q, gain_db),
        }
    }

    /// Process samples in-place
    fn process_inplace<'py>(&mut self, samples: &Bound<'py, PyArray1<f32>>) -> PyResult<()> {
        let mut samples_rw = unsafe { samples.as_array_mut() };
        let slice = samples_rw.as_slice_mut()
            .ok_or_else(|| PyValueError::new_err("Array must be contiguous"))?;
        
        self.inner.process_block(slice);
        Ok(())
    }

    /// Process samples and return new array
    fn process<'py>(
        &mut self,
        py: Python<'py>,
        samples: PyReadonlyArray1<'py, f32>,
    ) -> PyResult<Bound<'py, PyArray1<f32>>> {
        let input = samples.as_slice()
            .map_err(|_| PyValueError::new_err("Array must be contiguous"))?;
        
        let mut output: Vec<f32> = input.to_vec();
        self.inner.process_block(&mut output);
        
        Ok(PyArray1::from_vec(py, output))
    }

    /// Reset filter state
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Dynamic range compressor
#[cfg(feature = "python-bindings")]
#[pyclass]
pub struct DynamicCompressor {
    inner: Compressor,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl DynamicCompressor {
    /// Create a new compressor
    ///
    /// Args:
    ///     sample_rate: Sample rate in Hz
    #[new]
    fn new(sample_rate: f32) -> Self {
        Self {
            inner: Compressor::new(sample_rate),
        }
    }

    /// Set threshold in dB
    fn set_threshold(&mut self, threshold_db: f32) {
        self.inner.set_threshold(threshold_db);
    }

    /// Set compression ratio
    fn set_ratio(&mut self, ratio: f32) {
        self.inner.set_ratio(ratio);
    }

    /// Set attack time in milliseconds
    fn set_attack(&mut self, attack_ms: f32) {
        self.inner.set_attack(attack_ms);
    }

    /// Set release time in milliseconds
    fn set_release(&mut self, release_ms: f32) {
        self.inner.set_release(release_ms);
    }

    /// Set knee width in dB
    fn set_knee(&mut self, knee_db: f32) {
        self.inner.set_knee(knee_db);
    }

    /// Set makeup gain in dB
    fn set_makeup_gain(&mut self, gain_db: f32) {
        self.inner.set_makeup_gain(gain_db);
    }

    /// Process samples in-place
    fn process_inplace<'py>(&mut self, samples: &Bound<'py, PyArray1<f32>>) -> PyResult<()> {
        let mut samples_rw = unsafe { samples.as_array_mut() };
        let slice = samples_rw.as_slice_mut()
            .ok_or_else(|| PyValueError::new_err("Array must be contiguous"))?;
        
        self.inner.process_block(slice);
        Ok(())
    }

    /// Get current gain reduction in dB
    #[getter]
    fn gain_reduction_db(&self) -> f32 {
        self.inner.gain_reduction_db()
    }

    /// Reset compressor state
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Noise gate
#[cfg(feature = "python-bindings")]
#[pyclass]
pub struct Gate {
    inner: NoiseGate,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Gate {
    /// Create a new noise gate
    #[new]
    fn new(sample_rate: f32) -> Self {
        Self {
            inner: NoiseGate::new(sample_rate),
        }
    }

    /// Set threshold in dB
    fn set_threshold(&mut self, threshold_db: f32) {
        self.inner.set_threshold(threshold_db);
    }

    /// Set attack time in milliseconds
    fn set_attack(&mut self, attack_ms: f32) {
        self.inner.set_attack(attack_ms);
    }

    /// Set hold time in milliseconds
    fn set_hold(&mut self, hold_ms: f32) {
        self.inner.set_hold(hold_ms);
    }

    /// Set release time in milliseconds
    fn set_release(&mut self, release_ms: f32) {
        self.inner.set_release(release_ms);
    }

    /// Set range (maximum attenuation) in dB
    fn set_range(&mut self, range_db: f32) {
        self.inner.set_range(range_db);
    }

    /// Process samples in-place
    fn process_inplace<'py>(&mut self, samples: &Bound<'py, PyArray1<f32>>) -> PyResult<()> {
        let mut samples_rw = unsafe { samples.as_array_mut() };
        let slice = samples_rw.as_slice_mut()
            .ok_or_else(|| PyValueError::new_err("Array must be contiguous"))?;
        
        self.inner.process_block(slice);
        Ok(())
    }

    /// Check if gate is currently open
    #[getter]
    fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    /// Reset gate state
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Audio resampler
#[cfg(feature = "python-bindings")]
#[pyclass]
pub struct Resampler {
    inner: CoreResampler,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Resampler {
    /// Create a new resampler
    ///
    /// Args:
    ///     input_rate: Input sample rate in Hz
    ///     output_rate: Output sample rate in Hz
    #[new]
    fn new(input_rate: u32, output_rate: u32) -> Self {
        Self {
            inner: CoreResampler::new(input_rate, output_rate),
        }
    }

    /// Process samples
    ///
    /// Args:
    ///     samples: Input samples
    ///
    /// Returns:
    ///     Resampled output
    fn process<'py>(
        &mut self,
        py: Python<'py>,
        samples: PyReadonlyArray1<'py, f32>,
    ) -> PyResult<Bound<'py, PyArray1<f32>>> {
        let input = samples.as_slice()
            .map_err(|_| PyValueError::new_err("Array must be contiguous"))?;
        
        let output_size = self.inner.output_size(input.len());
        let mut output = vec![0.0f32; output_size];
        
        let written = self.inner.process(input, &mut output);
        output.truncate(written);
        
        Ok(PyArray1::from_vec(py, output))
    }

    /// Reset resampler state
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Get the resampling ratio
    #[getter]
    fn ratio(&self) -> f32 {
        self.inner.ratio()
    }
}

/// Calculate RMS of audio samples
#[cfg(feature = "python-bindings")]
#[pyfunction]
fn calculate_rms(samples: PyReadonlyArray1<f32>) -> PyResult<f32> {
    let slice = samples.as_slice()
        .map_err(|_| PyValueError::new_err("Array must be contiguous"))?;
    
    if slice.is_empty() {
        return Ok(0.0);
    }
    
    let sum_sq: f32 = slice.iter().map(|&x| x * x).sum();
    Ok((sum_sq / slice.len() as f32).sqrt())
}

/// Find peak amplitude of audio samples
#[cfg(feature = "python-bindings")]
#[pyfunction]
fn find_peak(samples: PyReadonlyArray1<f32>) -> PyResult<f32> {
    let slice = samples.as_slice()
        .map_err(|_| PyValueError::new_err("Array must be contiguous"))?;
    
    Ok(slice.iter().map(|&x| x.abs()).fold(0.0f32, f32::max))
}

/// Convert decibels to linear amplitude
#[cfg(feature = "python-bindings")]
#[pyfunction]
fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Convert linear amplitude to decibels
#[cfg(feature = "python-bindings")]
#[pyfunction]
fn linear_to_db(linear: f32) -> f32 {
    if linear <= 1e-6 {
        -120.0
    } else {
        20.0 * linear.log10()
    }
}

/// AirTen Python module
#[cfg(feature = "python-bindings")]
#[pymodule]
fn airten(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<AudioProcessor>()?;
    m.add_class::<Filter>()?;
    m.add_class::<DynamicCompressor>()?;
    m.add_class::<Gate>()?;
    m.add_class::<Resampler>()?;
    m.add_function(wrap_pyfunction!(calculate_rms, m)?)?;
    m.add_function(wrap_pyfunction!(find_peak, m)?)?;
    m.add_function(wrap_pyfunction!(db_to_linear, m)?)?;
    m.add_function(wrap_pyfunction!(linear_to_db, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

// Stub implementation when Python bindings are disabled
#[cfg(not(feature = "python-bindings"))]
pub fn calculate_rms(_samples: &[f32]) -> Result<f32, &'static str> {
    Err("Python bindings not enabled. Build with --features python-bindings")
}

#[cfg(not(feature = "python-bindings"))]
pub fn find_peak(_samples: &[f32]) -> Result<f32, &'static str> {
    Err("Python bindings not enabled. Build with --features python-bindings")
}

#[cfg(not(feature = "python-bindings"))]
pub fn db_to_linear(_db: f32) -> Result<f32, &'static str> {
    Err("Python bindings not enabled. Build with --features python-bindings")
}

#[cfg(not(feature = "python-bindings"))]
pub fn linear_to_db(_linear: f32) -> Result<f32, &'static str> {
    Err("Python bindings not enabled. Build with --features python-bindings")
}
