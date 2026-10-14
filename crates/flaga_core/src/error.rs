use liaise::{Liaise, LiaiseCodes};
use alloc::string::String;
#[derive(LiaiseCodes, Debug)]
#[liaise(prefix = "FLAG")]
pub enum FlagError {
    // --- 1xxx: Discovery Errors ---
    #[liaise(code = 1001, msg = "Flag '{name}' not found in the current manager state")]
    NotFound { name: String },

    #[liaise(code = 1002, msg = "The requested flag definition could not be located")]
    FlagNotFound,

    #[liaise(code = 1003, msg = "Cannot register flag: a definition with this name already exists")]
    FlagAlreadyExists,

    // --- 2xxx: Type & Logic Errors ---
    #[liaise(code = 2001, msg = "The operation is incompatible with the provided flag's Type")]
    InvalidFlagType,

    #[liaise(code = 2002, msg = "No persistence store configured")]
    NoStoreConfigured,

    #[liaise(code = 2003, msg = "Lock poisoned")]
    LockPoisoned,

    // --- 3xxx: System & I/O Errors ---
    #[cfg(feature = "std")]
    #[liaise(code = 3001, msg = "Filesystem I/O failure", source)]
    IOError(std::io::Error),

    // #[liaise(code = 3002, msg = "Extended Attribute (XAttr) operation failed: {0}")]
    // XAttrError(String),

    // #[cfg(feature = "std")]
    // #[liaise(code = 3003, msg = "Permission/ACL error")]
    // ACLError(std::io::Error),

    // --- 4xxx: Serialization Errors ---
    #[cfg(feature = "json")]
    #[liaise(code = 4001, msg = "JSON Serialization failure", source)]
    SerdeError(serde_json::Error),

    #[cfg(feature = "bincode")]
    #[liaise(code = 4002, msg = "Binary (Bincode) Serialization failure", source)]
    BincodeError(bincode::Error),

    #[liaise(code = 4003, msg = "Encode error: {item}")]
    Encode {item: String},

    #[liaise(code = 4004, msg = "Decode error: {item}")]
    Decode { item: String },
}




// use std::io;
// use thiserror::Error;

// /// `FlagError` defines the exhaustive list of failure states within the Flagger system.
// ///
// /// It utilizes the `thiserror` crate to automatically implement `std::fmt::Display`
// /// and `std::error::Error`, allowing for elegant error propagation using the `?` operator.
// #[derive(Debug, Error)]
// pub enum FlagError {
//     // --- Discovery Errors ---

//     /// Triggered when a specific flag name is requested but does not exist in the registry.
//     #[error("Flag '{0}' not found in the current manager state")]
//     NotFound(String),

//     /// A generic version of the not-found error for cases where the specific name isn't available.
//     #[error("The requested flag definition could not be located")]
//     FlagNotFound,

//     /// Triggered during registration if a flag name is already taken.
//     #[error("Cannot register flag: a definition with this name already exists")]
//     FlagAlreadyExists,

//     // --- Type & Logic Errors ---

//     /// Occurs if a bitmask operation is attempted on a flag with a mismatched category (e.g., Hex vs Binary).
//     #[error("The operation is incompatible with the provided flag's Type (Binary/Hex/Enum)")]
//     InvalidFlagType,

//     // --- System & I/O Errors ---

//     /// Wraps standard filesystem errors (permissions, missing files, etc.).
//     /// The `#[from]` attribute allows `std::io::Error` to be automatically converted into this variant.
//     #[error("Filesystem I/O failure: {0}")]
//     IOError(#[from] std::io::Error),

//     /// Represents failures related to Extended File Attributes (metadata) often used in specialized filesystems.
//     #[error("Extended Attribute (XAttr) operation failed: {0}")]
//     XAttrError(String),

//     /// Specifically handles Access Control List failures, which manage fine-grained file permissions.
//     #[error("Permission/ACL error: {0}")]
//     ACLError(io::Error),






// //     // --- Serialization Errors ---
// // 
// //     /// Handles failures during JSON encoding/decoding. 
// //     /// Useful for web-based configs or human-readable exports.
// //     #[error("JSON Serialization failure: {0}")]
// //     SerdeError(#[from] serde_json::Error),
// // 
// //     /// Handles failures during Bincode encoding/decoding.
// //     /// This is the primary error for high-performance binary persistence.
// //     #[error("Binary (Bincode) Serialization failure: {0}")]
// //     BincodeError(#[from] bincode::Error),
//     // --- Serialization Errors ---

//     /// Handles failures during JSON encoding/decoding.
//     /// Useful for web-based configs or human-readable exports.
//     #[cfg(feature = "json")]
//     #[error("JSON Serialization failure: {0}")]
//     SerdeError(#[from] serde_json::Error),

//     /// Handles failures during Bincode encoding/decoding.
//     /// This is the primary error for high-performance binary persistence.
//     #[cfg(feature = "bincode")]
//     #[error("Binary (Bincode) Serialization failure: {0}")]
//     BincodeError(#[from] bincode::Error),
    



    
// //     #[error("flag already exists")]
// //     FlagAlreadyExists,
// // 
// //     #[error("flag not found: {0}")]
// //     NotFound(String),

//     #[error("no persistence store configured")]
//     NoStoreConfigured,

//     #[error("lock poisoned")]
//     LockPoisoned,

// //     #[error("io error: {0}")]
// //     Io(#[from] std::io::Error),

//     #[error("encode error: {0}")]
//     Encode(String),

//     #[error("decode error: {0}")]
//     Decode(String),
// }

