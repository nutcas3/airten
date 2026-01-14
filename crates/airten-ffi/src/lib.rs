use std::ffi::c_void;
use std::ptr;
use std::slice;

use airten_core::{AudioProcessor, ProcessorConfig, Sample};
use airten_core::dsp::{BiquadFilter, Compressor, NoiseGate};

mod types;
mod processor;
mod utils;

pub use types::*;
pub use processor::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AirtenError {
    Ok = 0,
    NullPointer = 1,
    InvalidParameter = 2,
    InvalidBufferSize = 3,
    AllocationFailed = 4,
    NotSupported = 5,
    InternalError = 6,
}

#[repr(C)]
pub struct AirtenProcessor {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AirtenFilter {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AirtenCompressor {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AirtenGate {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AirtenConfig {
    pub sample_rate: u32,
    pub frame_size: u32,
    pub num_channels: u32,
    pub noise_suppression: i32,
    pub compression: i32,
}

impl Default for AirtenConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            frame_size: 512,
            num_channels: 1,
            noise_suppression: 1,
            compression: 1,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn airten_processor_new(config: *const AirtenConfig) -> *mut AirtenProcessor {
    let config = if config.is_null() {
        AirtenConfig::default()
    } else {
        *config
    };

    let proc_config = ProcessorConfig {
        sample_rate: config.sample_rate,
        frame_size: config.frame_size as usize,
        num_channels: config.num_channels as usize,
        noise_suppression: config.noise_suppression != 0,
        compression: config.compression != 0,
        target_latency_ms: 10.0,
    };

    let processor = Box::new(AudioProcessor::new(proc_config));
    Box::into_raw(processor) as *mut AirtenProcessor
}

#[no_mangle]
pub unsafe extern "C" fn airten_processor_free(processor: *mut AirtenProcessor) {
    if !processor.is_null() {
        drop(Box::from_raw(processor as *mut AudioProcessor));
    }
}

#[no_mangle]
pub unsafe extern "C" fn airten_process(
    processor: *mut AirtenProcessor,
    samples: *mut f32,
    num_samples: u32,
) -> AirtenError {
    if processor.is_null() || samples.is_null() {
        return AirtenError::NullPointer;
    }

    let processor = &mut *(processor as *mut AudioProcessor);
    let samples = slice::from_raw_parts_mut(samples, num_samples as usize);

    match processor.process(samples) {
        Ok(()) => AirtenError::Ok,
        Err(_) => AirtenError::InvalidBufferSize,
    }
}

#[no_mangle]
pub unsafe extern "C" fn airten_processor_reset(processor: *mut AirtenProcessor) -> AirtenError {
    if processor.is_null() {
        return AirtenError::NullPointer;
    }

    let processor = &mut *(processor as *mut AudioProcessor);
    processor.reset();
    AirtenError::Ok
}

#[repr(C)]
pub enum AirtenFilterType {
    LowPass = 0,
    HighPass = 1,
    BandPass = 2,
    Notch = 3,
    Peaking = 4,
    LowShelf = 5,
    HighShelf = 6,
}

#[no_mangle]
pub unsafe extern "C" fn airten_filter_new(
    filter_type: AirtenFilterType,
    sample_rate: f32,
    frequency: f32,
    q: f32,
    gain_db: f32,
) -> *mut AirtenFilter {
    let filter = match filter_type {
        AirtenFilterType::LowPass => BiquadFilter::lowpass(sample_rate, frequency, q),
        AirtenFilterType::HighPass => BiquadFilter::highpass(sample_rate, frequency, q),
        AirtenFilterType::BandPass => BiquadFilter::bandpass(sample_rate, frequency, q),
        AirtenFilterType::Notch => BiquadFilter::notch(sample_rate, frequency, q),
        AirtenFilterType::Peaking => BiquadFilter::peaking(sample_rate, frequency, q, gain_db),
        AirtenFilterType::LowShelf => BiquadFilter::low_shelf(sample_rate, frequency, gain_db),
        AirtenFilterType::HighShelf => BiquadFilter::high_shelf(sample_rate, frequency, gain_db),
    };

    Box::into_raw(Box::new(filter)) as *mut AirtenFilter
}

#[no_mangle]
pub unsafe extern "C" fn airten_filter_free(filter: *mut AirtenFilter) {
    if !filter.is_null() {
        drop(Box::from_raw(filter as *mut BiquadFilter));
    }
}

#[no_mangle]
pub unsafe extern "C" fn airten_filter_process(
    filter: *mut AirtenFilter,
    samples: *mut f32,
    num_samples: u32,
) -> AirtenError {
    if filter.is_null() || samples.is_null() {
        return AirtenError::NullPointer;
    }

    let filter = &mut *(filter as *mut BiquadFilter);
    let samples = slice::from_raw_parts_mut(samples, num_samples as usize);
    filter.process_block(samples);
    AirtenError::Ok
}

#[no_mangle]
pub unsafe extern "C" fn airten_filter_reset(filter: *mut AirtenFilter) -> AirtenError {
    if filter.is_null() {
        return AirtenError::NullPointer;
    }

    let filter = &mut *(filter as *mut BiquadFilter);
    filter.reset();
    AirtenError::Ok
}

#[no_mangle]
pub unsafe extern "C" fn airten_compressor_new(sample_rate: f32) -> *mut AirtenCompressor {
    let compressor = Box::new(Compressor::new(sample_rate));
    Box::into_raw(compressor) as *mut AirtenCompressor
}

#[no_mangle]
pub unsafe extern "C" fn airten_compressor_free(compressor: *mut AirtenCompressor) {
    if !compressor.is_null() {
        drop(Box::from_raw(compressor as *mut Compressor));
    }
}

#[no_mangle]
pub unsafe extern "C" fn airten_compressor_set_threshold(
    compressor: *mut AirtenCompressor,
    threshold_db: f32,
) -> AirtenError {
    if compressor.is_null() {
        return AirtenError::NullPointer;
    }

    let compressor = &mut *(compressor as *mut Compressor);
    compressor.set_threshold(threshold_db);
    AirtenError::Ok
}

#[no_mangle]
pub unsafe extern "C" fn airten_compressor_set_ratio(
    compressor: *mut AirtenCompressor,
    ratio: f32,
) -> AirtenError {
    if compressor.is_null() {
        return AirtenError::NullPointer;
    }

    let compressor = &mut *(compressor as *mut Compressor);
    compressor.set_ratio(ratio);
    AirtenError::Ok
}

#[no_mangle]
pub unsafe extern "C" fn airten_compressor_set_attack(
    compressor: *mut AirtenCompressor,
    attack_ms: f32,
) -> AirtenError {
    if compressor.is_null() {
        return AirtenError::NullPointer;
    }

    let compressor = &mut *(compressor as *mut Compressor);
    compressor.set_attack(attack_ms);
    AirtenError::Ok
}

#[no_mangle]
pub unsafe extern "C" fn airten_compressor_set_release(
    compressor: *mut AirtenCompressor,
    release_ms: f32,
) -> AirtenError {
    if compressor.is_null() {
        return AirtenError::NullPointer;
    }

    let compressor = &mut *(compressor as *mut Compressor);
    compressor.set_release(release_ms);
    AirtenError::Ok
}

#[no_mangle]
pub unsafe extern "C" fn airten_compressor_process(
    compressor: *mut AirtenCompressor,
    samples: *mut f32,
    num_samples: u32,
) -> AirtenError {
    if compressor.is_null() || samples.is_null() {
        return AirtenError::NullPointer;
    }

    let compressor = &mut *(compressor as *mut Compressor);
    let samples = slice::from_raw_parts_mut(samples, num_samples as usize);
    compressor.process_block(samples);
    AirtenError::Ok
}

#[no_mangle]
pub unsafe extern "C" fn airten_gate_new(sample_rate: f32) -> *mut AirtenGate {
    let gate = Box::new(NoiseGate::new(sample_rate));
    Box::into_raw(gate) as *mut AirtenGate
}

#[no_mangle]
pub unsafe extern "C" fn airten_gate_free(gate: *mut AirtenGate) {
    if !gate.is_null() {
        drop(Box::from_raw(gate as *mut NoiseGate));
    }
}

#[no_mangle]
pub unsafe extern "C" fn airten_gate_set_threshold(
    gate: *mut AirtenGate,
    threshold_db: f32,
) -> AirtenError {
    if gate.is_null() {
        return AirtenError::NullPointer;
    }

    let gate = &mut *(gate as *mut NoiseGate);
    gate.set_threshold(threshold_db);
    AirtenError::Ok
}

#[no_mangle]
pub unsafe extern "C" fn airten_gate_process(
    gate: *mut AirtenGate,
    samples: *mut f32,
    num_samples: u32,
) -> AirtenError {
    if gate.is_null() || samples.is_null() {
        return AirtenError::NullPointer;
    }

    let gate = &mut *(gate as *mut NoiseGate);
    let samples = slice::from_raw_parts_mut(samples, num_samples as usize);
    gate.process_block(samples);
    AirtenError::Ok
}

#[no_mangle]
pub extern "C" fn airten_version() -> *const i8 {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const i8
}

#[no_mangle]
pub extern "C" fn airten_version_info(major: *mut u32, minor: *mut u32, patch: *mut u32) {
    unsafe {
        if !major.is_null() {
            *major = env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or(0);
        }
        if !minor.is_null() {
            *minor = env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or(0);
        }
        if !patch.is_null() {
            *patch = env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_lifecycle() {
        unsafe {
            let config = AirtenConfig::default();
            let processor = airten_processor_new(&config);
            assert!(!processor.is_null());
            
            let mut samples = [0.5f32; 512];
            let result = airten_process(processor, samples.as_mut_ptr(), 512);
            assert_eq!(result, AirtenError::Ok);
            
            airten_processor_free(processor);
        }
    }

    #[test]
    fn test_filter_lifecycle() {
        unsafe {
            let filter = airten_filter_new(
                AirtenFilterType::LowPass,
                48000.0,
                1000.0,
                0.707,
                0.0,
            );
            assert!(!filter.is_null());
            
            let mut samples = [0.5f32; 256];
            let result = airten_filter_process(filter, samples.as_mut_ptr(), 256);
            assert_eq!(result, AirtenError::Ok);
            
            airten_filter_free(filter);
        }
    }

    #[test]
    fn test_null_pointer_handling() {
        unsafe {
            let result = airten_process(ptr::null_mut(), ptr::null_mut(), 0);
            assert_eq!(result, AirtenError::NullPointer);
        }
    }
}
