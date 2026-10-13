#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// `FlagType` defines the mathematical and logical category of a flag.
/// 
/// This discriminator tells the `FlagManager` whether to treat a value 
/// as a simple boolean toggle, a complex 64-bit hex mask, or a 
/// restricted set of enumerable states.

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FlagType {
    /// **Binary Flags (u8):** /// Used for simple "On/Off" toggles. 
    /// Ideal for features that are either enabled or disabled (e.g., `is_active`).
    Binary,

    /// **Hex Flags (u64):** /// Used for high-density bitmasks or complex configurations. 
    /// Provides 64 individual slots for flags, typically represented in hexadecimal 
    /// format (e.g., `0x0000FFFF`) for better readability in config files.
    Hex,

    /// **Enum Flags (u8):** /// Used for mutually exclusive states or specific categories. 
    /// While stored as a bitmask, these are logically grouped to represent 
    /// distinct modes (e.g., `Mode::Read`, `Mode::Write`, `Mode::Admin`).
    Enum,
}

/*
/// FUTURE ENHANCEMENT: FlagTypeDynamic
/// 
/// If you ever need to support metadata-heavy flags (like a UI that needs to 
/// know the names of every possible state in an Enum), you would use this.
/// 
/// The `Vec<&'static str>` would store the labels for each bit index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlagTypeDynamic {
    Binary,
    Hex,
    Enum(Vec<&'static str>),
}
*/