//! Error handling for the AMX runtime

use crate::types::AmxError;
use thiserror::Error;

/// AMX runtime errors
#[derive(Error, Debug)]
pub enum AmxRuntimeError {
    #[error("AMX error: {0}")]
    AmxError(#[from] crate::types::AmxError),

    #[error("Invalid file format")]
    InvalidFormat,

    #[error("File version not supported: {0}")]
    UnsupportedVersion(u8),

    #[error("Out of memory")]
    OutOfMemory,

    #[error("Invalid instruction at offset {0}")]
    InvalidInstruction(usize),

    #[error("Stack overflow")]
    StackOverflow,

    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Heap overflow")]
    HeapOverflow,

    #[error("Heap underflow")]
    HeapUnderflow,

    #[error("Array bounds error")]
    ArrayBounds,

    #[error("Invalid memory access at address 0x{0:08x}")]
    InvalidMemoryAccess(usize),

    #[error("Native function not found: {0}")]
    NativeNotFound(String),

    #[error("Public function not found: {0}")]
    PublicNotFound(String),

    #[error("Public variable not found: {0}")]
    PubVarNotFound(String),

    #[error("Tag not found: {0}")]
    TagNotFound(String),

    #[error("Callback error: {0}")]
    CallbackError(String),

    #[error("Debug error: {0}")]
    DebugError(String),

    #[error("JIT initialization failed: {0}")]
    JitInitFailed(String),

    #[error("Parameter error: {0}")]
    ParameterError(String),

    #[error("Domain error: {0}")]
    DomainError(String),

    #[error("General error: {0}")]
    GeneralError(String),
}

impl From<AmxRuntimeError> for crate::types::AmxError {
    fn from(error: AmxRuntimeError) -> Self {
        match error {
            AmxRuntimeError::AmxError(e) => e,
            AmxRuntimeError::InvalidFormat => AmxError::Format,
            AmxRuntimeError::UnsupportedVersion(_) => AmxError::Version,
            AmxRuntimeError::OutOfMemory => AmxError::Memory,
            AmxRuntimeError::InvalidInstruction(_) => AmxError::InvInstr,
            AmxRuntimeError::StackOverflow => AmxError::StackErr,
            AmxRuntimeError::StackUnderflow => AmxError::StackLow,
            AmxRuntimeError::HeapOverflow => AmxError::StackErr,
            AmxRuntimeError::HeapUnderflow => AmxError::HeapLow,
            AmxRuntimeError::ArrayBounds => AmxError::Bounds,
            AmxRuntimeError::InvalidMemoryAccess(_) => AmxError::MemAccess,
            AmxRuntimeError::NativeNotFound(_) => AmxError::NotFound,
            AmxRuntimeError::PublicNotFound(_) => AmxError::NotFound,
            AmxRuntimeError::PubVarNotFound(_) => AmxError::NotFound,
            AmxRuntimeError::TagNotFound(_) => AmxError::NotFound,
            AmxRuntimeError::CallbackError(_) => AmxError::Callback,
            AmxRuntimeError::DebugError(_) => AmxError::Debug,
            AmxRuntimeError::JitInitFailed(_) => AmxError::InitJit,
            AmxRuntimeError::ParameterError(_) => AmxError::Params,
            AmxRuntimeError::DomainError(_) => AmxError::Domain,
            AmxRuntimeError::GeneralError(_) => AmxError::General,
        }
    }
}

/// Result type for AMX operations
pub type AmxResult<T> = Result<T, AmxRuntimeError>;
