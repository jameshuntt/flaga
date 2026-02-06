use serde_json;
use std::io;
use thiserror::Error;

/// `FlagError` defines the exhaustive list of failure states within the Flagger system.
///
/// It utilizes the `thiserror` crate to automatically implement `std::fmt::Display`
/// and `std::error::Error`, allowing for elegant error propagation using the `?` operator.
#[derive(Debug, Error)]
pub enum FlagError {
    // --- Discovery Errors ---

    /// Triggered when a specific flag name is requested but does not exist in the registry.
    #[error("Flag '{0}' not found in the current manager state")]
    NotFound(String),

    /// A generic version of the not-found error for cases where the specific name isn't available.
    #[error("The requested flag definition could not be located")]
    FlagNotFound,

    /// Triggered during registration if a flag name is already taken.
    #[error("Cannot register flag: a definition with this name already exists")]
    FlagAlreadyExists,

    // --- Type & Logic Errors ---

    /// Occurs if a bitmask operation is attempted on a flag with a mismatched category (e.g., Hex vs Binary).
    #[error("The operation is incompatible with the provided flag's Type (Binary/Hex/Enum)")]
    InvalidFlagType,

    // --- System & I/O Errors ---

    /// Wraps standard filesystem errors (permissions, missing files, etc.).
    /// The `#[from]` attribute allows `std::io::Error` to be automatically converted into this variant.
    #[error("Filesystem I/O failure: {0}")]
    IOError(#[from] std::io::Error),

    /// Represents failures related to Extended File Attributes (metadata) often used in specialized filesystems.
    #[error("Extended Attribute (XAttr) operation failed: {0}")]
    XAttrError(String),

    /// Specifically handles Access Control List failures, which manage fine-grained file permissions.
    #[error("Permission/ACL error: {0}")]
    ACLError(io::Error),

    // --- Serialization Errors ---

    /// Handles failures during JSON encoding/decoding. 
    /// Useful for web-based configs or human-readable exports.
    #[error("JSON Serialization failure: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// Handles failures during Bincode encoding/decoding.
    /// This is the primary error for high-performance binary persistence.
    #[error("Binary (Bincode) Serialization failure: {0}")]
    BincodeError(#[from] bincode::Error),
}