#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_attr_outside_unsafe)]

use std::ffi::CStr;
use std::os::raw::c_char;

pub unsafe fn c_str_to_str<'a>(s: *const c_char) -> Option<&'a str> {
    if s.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(s).to_str().ok() }
}

#[unsafe(no_mangle)]
pub extern "C" fn airten_db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn airten_linear_to_db(linear: f32) -> f32 {
    if linear <= 1e-6 {
        -120.0
    } else {
        20.0 * linear.log10()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn airten_calculate_rms(samples: *const f32, num_samples: u32) -> f32 {
    if samples.is_null() || num_samples == 0 {
        return 0.0;
    }

    let samples = unsafe { std::slice::from_raw_parts(samples, num_samples as usize) };
    let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
    (sum_sq / num_samples as f32).sqrt()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn airten_find_peak(samples: *const f32, num_samples: u32) -> f32 {
    if samples.is_null() || num_samples == 0 {
        return 0.0;
    }

    let samples = unsafe { std::slice::from_raw_parts(samples, num_samples as usize) };
    samples.iter().map(|&x| x.abs()).fold(0.0f32, f32::max)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn airten_apply_gain(samples: *mut f32, num_samples: u32, gain: f32) {
    if samples.is_null() || num_samples == 0 {
        return;
    }

    let samples = unsafe { std::slice::from_raw_parts_mut(samples, num_samples as usize) };
    for sample in samples {
        *sample *= gain;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn airten_mix_buffers(
    a: *const f32,
    b: *const f32,
    output: *mut f32,
    num_samples: u32,
    mix: f32,
) {
    if a.is_null() || b.is_null() || output.is_null() || num_samples == 0 {
        return;
    }

    let a = unsafe { std::slice::from_raw_parts(a, num_samples as usize) };
    let b = unsafe { std::slice::from_raw_parts(b, num_samples as usize) };
    let output = unsafe { std::slice::from_raw_parts_mut(output, num_samples as usize) };

    let inv_mix = 1.0 - mix;
    for i in 0..num_samples as usize {
        output[i] = a[i] * inv_mix + b[i] * mix;
    }
}
