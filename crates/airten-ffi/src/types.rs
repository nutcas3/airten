use std::os::raw::c_char;

#[repr(C)]
pub enum AirtenAudioFormat {
    Float32 = 0,
    Int16 = 1,
    Int32 = 2,
}

#[repr(C)]
pub enum AirtenChannelLayout {
    Mono = 1,
    Stereo = 2,
}

#[repr(C)]
pub struct AirtenAudioBuffer {
    pub data: *mut f32,
    pub num_samples: u32,
    pub num_channels: u32,
    pub sample_rate: u32,
    pub interleaved: i32,
}

impl Default for AirtenAudioBuffer {
    fn default() -> Self {
        Self {
            data: std::ptr::null_mut(),
            num_samples: 0,
            num_channels: 1,
            sample_rate: 48000,
            interleaved: 1,
        }
    }
}

#[repr(C)]
pub struct AirtenStats {
    pub latency_ms: f32,
    pub input_level_db: f32,
    pub output_level_db: f32,
    pub gain_reduction_db: f32,
    pub frames_processed: u64,
}

pub type AirtenProcessCallback = Option<
    unsafe extern "C" fn(
        user_data: *mut std::ffi::c_void,
        input: *const f32,
        output: *mut f32,
        num_samples: u32,
    ),
>;

pub type AirtenLogCallback = Option<
    unsafe extern "C" fn(
        level: i32,
        message: *const c_char,
    ),
>;
