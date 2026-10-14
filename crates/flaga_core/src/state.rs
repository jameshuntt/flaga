use crate::flag_descriptor::FlagDescriptor;
use std::collections::HashMap;
use alloc::string::String;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Pure persisted state.
/// No locks, no Arc, no file path, no backend concerns.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct Inner {
    /// 8-bit mask for binary/boolean logic.
    pub binary_flags: u8,

    /// 64-bit mask for complex or large hex-based flags.
    pub hex_flags: u64,

    /// 8-bit mask reserved for enumerable state flags.
    pub enum_flags: u8,

    /// Registry of flag names and descriptors.
    pub flags: HashMap<String, FlagDescriptor<u64>>,
}
