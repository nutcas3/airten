#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_attr_outside_unsafe)]

use std::slice;
use crate::{AirtenError, AirtenAudioBuffer, AirtenStats};
use airten_core::{AudioProcessor, AudioFrame};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn airten_process_buffer(
    processor: *mut crate::AirtenProcessor,
    buffer: *mut AirtenAudioBuffer,
) -> AirtenError {
    if processor.is_null() || buffer.is_null() {
        return AirtenError::NullPointer;
    }

    let processor = unsafe { &mut *(processor as *mut AudioProcessor) };
    let buffer = unsafe { &mut *buffer };

    if buffer.data.is_null() {
        return AirtenError::NullPointer;
    }

    let total_samples = (buffer.num_samples * buffer.num_channels) as usize;
    let samples = unsafe { slice::from_raw_parts_mut(buffer.data, total_samples) };

    if buffer.interleaved != 0 {
        let mut frame = AudioFrame::with_config(
            buffer.num_samples as usize,
            buffer.num_channels as usize,
            buffer.sample_rate,
        );
        frame.from_interleaved(samples);
        
        if let Err(_) = processor.process_frame(&mut frame) {
            return AirtenError::InternalError;
        }
        
        frame.to_interleaved(samples);
    } else {
        for ch in 0..buffer.num_channels as usize {
            let offset = ch * buffer.num_samples as usize;
            let channel_samples = &mut samples[offset..offset + buffer.num_samples as usize];
            
            if let Err(_) = processor.process(channel_samples) {
                return AirtenError::InternalError;
            }
        }
    }

    AirtenError::Ok
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn airten_get_stats(
    processor: *mut crate::AirtenProcessor,
    stats: *mut AirtenStats,
) -> AirtenError {
    if processor.is_null() || stats.is_null() {
        return AirtenError::NullPointer;
    }

    let processor = unsafe { &*(processor as *mut AudioProcessor) };
    let stats = unsafe { &mut *stats };

    let config = processor.config();
    stats.latency_ms = (config.frame_size as f32 / config.sample_rate as f32) * 1000.0;
    stats.input_level_db = 0.0;
    stats.output_level_db = 0.0;
    stats.gain_reduction_db = 0.0;

    AirtenError::Ok
}
