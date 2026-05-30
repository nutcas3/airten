use crate::Sample;
use core::ops::{Add, Mul, Neg, Sub};

/// Q15 fixed-point number format (16-bit signed integer with 15 fractional bits)
///
/// Q15 format represents numbers in the range [-1.0, 1.0) using 16-bit signed integers.
/// The value 32767 (0x7FFF) represents approximately 1.0, and -32768 (0x8000) represents -1.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct Q15(pub i16);

impl Q15 {
    /// Scale factor for Q15 format (2^15 = 32768)
    pub const SCALE: i32 = 32768;
    /// Maximum Q15 value (1.0)
    #[allow(dead_code)]
    pub const MAX: Q15 = Q15(i16::MAX);
    /// Minimum Q15 value (-1.0)
    #[allow(dead_code)]
    pub const MIN: Q15 = Q15(i16::MIN);
    /// Zero Q15 value (0.0)
    #[allow(dead_code)]
    pub const ZERO: Q15 = Q15(0);
    /// One Q15 value (1.0)
    #[allow(dead_code)]
    pub const ONE: Q15 = Q15(i16::MAX);

    /// Creates Q15 from raw integer value
    #[inline]
    #[allow(dead_code)]
    pub const fn from_raw(raw: i16) -> Self {
        Self(raw)
    }

    /// Returns raw integer value
    #[allow(dead_code)]
    pub const fn to_raw(self) -> i16 {
        self.0
    }

    /// Creates Q15 from f32 value, clamping to valid range
    #[inline]
    pub fn from_f32(x: Sample) -> Self {
        let scaled = (x * Self::SCALE as Sample).round();
        let clamped = scaled.clamp(i16::MIN as Sample, i16::MAX as Sample);
        Self(clamped as i16)
    }

    /// Converts Q15 to f32 value
    #[inline]
    pub fn to_f32(self) -> Sample {
        (self.0 as Sample) / Self::SCALE as Sample
    }

    /// Saturating addition that clamps to valid Q15 range
    #[inline]
    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    #[inline]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    #[inline]
    pub fn mul_q15(self, rhs: Self) -> Self {
        let product = (self.0 as i32) * (rhs.0 as i32);
        let shifted = (product + (1 << 14)) >> 15;
        let saturated = shifted.clamp(i16::MIN as i32, i16::MAX as i32);
        Self(saturated as i16)
    }

    /// Returns the absolute value
    #[inline]
    #[allow(dead_code)]
    pub fn abs(self) -> Self {
        Self(self.0.saturating_abs())
    }

    /// Returns the saturated negation
    #[inline]
    #[allow(dead_code)]
    pub fn saturating_neg(self) -> Self {
        Self(self.0.saturating_neg())
    }

    /// Linear interpolation between self and other
    ///
    /// # Arguments
    /// * `other` - End value
    /// * `t` - Interpolation parameter (0.0 = self, 1.0 = other)
    #[inline]
    #[allow(dead_code)]
    pub fn lerp(self, other: Self, t: Self) -> Self {
        let diff = other.saturating_sub(self);
        let scaled = t.mul_q15(diff);
        self.saturating_add(scaled)
    }

    /// Left shift by n bits (equivalent to multiplication by 2^n)
    ///
    /// # Arguments
    /// * `n` - Number of bits to shift
    #[inline]
    #[allow(dead_code)]
    pub fn shl(self, n: u32) -> Self {
        Self(self.0.saturating_mul(1i16.wrapping_shl(n)))
    }

    /// Right shift by n bits (equivalent to division by 2^n)
    ///
    /// # Arguments
    /// * `n` - Number of bits to shift
    #[inline]
    #[allow(dead_code)]
    pub fn shr(self, n: u32) -> Self {
        Self(self.0 >> n)
    }
}

impl Add for Q15 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }
}

impl Sub for Q15 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
    }
}

impl Mul for Q15 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        self.mul_q15(rhs)
    }
}

impl Neg for Q15 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        self.saturating_neg()
    }
}

impl From<Sample> for Q15 {
    fn from(x: Sample) -> Self {
        Self::from_f32(x)
    }
}

impl From<Q15> for Sample {
    fn from(x: Q15) -> Self {
        x.to_f32()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct Q31(pub i32);

impl Q31 {
    pub const SCALE: i64 = 2147483648;

    /// Maximum Q31 value (1.0)
    #[allow(dead_code)]
    pub const MAX: Q31 = Q31(i32::MAX);

    /// Minimum Q31 value (-1.0)
    #[allow(dead_code)]
    pub const MIN: Q31 = Q31(i32::MIN);

    /// Zero Q31 value (0.0)
    #[allow(dead_code)]
    pub const ZERO: Q31 = Q31(0);

    #[inline]
    /// Creates Q31 from raw integer value
    #[allow(dead_code)]
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    /// Returns raw integer value
    #[allow(dead_code)]
    pub const fn to_raw(self) -> i32 {
        self.0
    }

    #[inline]
    pub fn from_f32(x: Sample) -> Self {
        let scaled = (x as f64 * Self::SCALE as f64).round();
        let clamped = scaled.clamp(i32::MIN as f64, i32::MAX as f64);
        Self(clamped as i32)
    }

    #[inline]
    pub fn to_f32(self) -> Sample {
        (self.0 as f64 / Self::SCALE as f64) as Sample
    }

    #[inline]
    pub fn to_q15(self) -> Q15 {
        Q15((self.0 >> 16) as i16)
    }

    pub fn from_q15(q15: Q15) -> Self {
        Self((q15.0 as i32) << 16)
    }

    #[inline]
    /// Multiplies two Q31 numbers with proper rounding
    #[allow(dead_code)]
    pub fn mul_q31(self, rhs: Self) -> Self {
        let product = (self.0 as i64) * (rhs.0 as i64);
        let shifted = (product + (1i64 << 30)) >> 31;
        let saturated = shifted.clamp(i32::MIN as i64, i32::MAX as i64);
        Self(saturated as i32)
    }
}

pub fn f32_to_q15_slice(input: &[Sample], output: &mut [Q15]) {
    let len = input.len().min(output.len());
    for i in 0..len {
        output[i] = Q15::from_f32(input[i]);
    }
}

pub fn q15_to_f32_slice(input: &[Q15], output: &mut [Sample]) {
    let len = input.len().min(output.len());
    for i in 0..len {
        output[i] = input[i].to_f32();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_q15_conversion() {
        let f = 0.5f32;
        let q = Q15::from_f32(f);
        let back = q.to_f32();
        assert!((f - back).abs() < 0.0001);
    }

    #[test]
    fn test_q15_saturation() {
        let q = Q15::from_f32(2.0);
        assert_eq!(q, Q15::MAX);
        let q = Q15::from_f32(-2.0);
        assert_eq!(q, Q15::MIN);
    }

    #[test]
    fn test_q15_multiplication() {
        let a = Q15::from_f32(0.5);
        let b = Q15::from_f32(0.5);
        let c = a * b;
        let result = c.to_f32();
        assert!((result - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_q15_addition() {
        let a = Q15::from_f32(0.25);
        let b = Q15::from_f32(0.25);
        let c = a + b;
        let result = c.to_f32();
        assert!((result - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_q15_saturation_add() {
        let a = Q15::from_f32(0.9);
        let b = Q15::from_f32(0.9);
        let c = a + b;

        assert!(c.to_f32() <= 1.0);
    }

    #[test]
    fn test_q15_lerp() {
        let a = Q15::from_f32(0.0);
        let b = Q15::from_f32(1.0);
        let t = Q15::from_f32(0.5);
        let result = a.lerp(b, t);
        assert!((result.to_f32() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_q31_conversion() {
        let f = 0.5f32;
        let q = Q31::from_f32(f);
        let back = q.to_f32();
        assert!((f - back).abs() < 0.0000001);
    }

    #[test]
    fn test_q15_q31_conversion() {
        let q15 = Q15::from_f32(0.5);
        let q31 = Q31::from_q15(q15);
        let back = q31.to_q15();
        assert_eq!(q15, back);
    }
}
