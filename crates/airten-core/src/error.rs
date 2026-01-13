use core::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    BufferTooLarge,
    InvalidBufferSize,
    InvalidSampleRate,
    InvalidChannelCount,
    ChannelOutOfBounds,
    BufferFull,
    BufferEmpty,
    InvalidFilterParams,
    InvalidCompressorParams,
    InferenceError,
    ModelNotLoaded,
    InvalidModelFormat,
    AllocationFailed,
    NotSupported,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferTooLarge => write!(f, "buffer size exceeds maximum allowed"),
            Self::InvalidBufferSize => write!(f, "invalid buffer size"),
            Self::InvalidSampleRate => write!(f, "invalid sample rate"),
            Self::InvalidChannelCount => write!(f, "invalid channel count"),
            Self::ChannelOutOfBounds => write!(f, "channel index out of bounds"),
            Self::BufferFull => write!(f, "ring buffer is full"),
            Self::BufferEmpty => write!(f, "ring buffer is empty"),
            Self::InvalidFilterParams => write!(f, "invalid filter parameters"),
            Self::InvalidCompressorParams => write!(f, "invalid compressor parameters"),
            Self::InferenceError => write!(f, "neural network inference error"),
            Self::ModelNotLoaded => write!(f, "model not loaded"),
            Self::InvalidModelFormat => write!(f, "invalid model format"),
            Self::AllocationFailed => write!(f, "memory allocation failed"),
            Self::NotSupported => write!(f, "operation not supported"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
