use core::fmt;

/// Result type for `AirTen` operations
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur during audio processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Buffer size exceeds maximum allowed
    BufferTooLarge,
    /// Invalid buffer size provided
    InvalidBufferSize,
    /// Invalid sample rate provided
    InvalidSampleRate,
    /// Invalid channel count provided
    InvalidChannelCount,
    /// Channel index is out of bounds
    ChannelOutOfBounds,
    /// Buffer is full and cannot accept more data
    BufferFull,
    /// Buffer is empty and cannot provide data
    BufferEmpty,
    /// Invalid filter parameters provided
    InvalidFilterParams,
    /// Invalid compressor parameters provided
    InvalidCompressorParams,
    /// Neural network inference error
    InferenceError,
    /// Neural network model not loaded
    ModelNotLoaded,
    /// Invalid model format provided
    InvalidModelFormat,
    /// Memory allocation failed
    AllocationFailed,
    /// Operation not supported
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
