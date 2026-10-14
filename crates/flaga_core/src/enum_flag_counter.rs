use crate::flag::Flag;

/// A concrete implementation of the [`Flag`] trait using an 8-bit unsigned integer.
/// 
/// This container uses a specialized Enum ([`EnumFlag`]) to enforce type-safety, 
/// ensuring that only valid, pre-defined bits can be manipulated.
pub struct EnumFlagContainer {
    /// The raw 8-bit storage for the bitmask.
    value: u8,
}

impl Flag for EnumFlagContainer {
    /// We bind the trait's `Value` to our specific `EnumFlag` type.
    type Value = EnumFlag;

    /// Activates the bit corresponding to the enum variant using bitwise OR.
    fn set_flag(&mut self, flag: Self::Value) {
        self.value |= flag as u8;
    }

    /// Checks if the specific bit for the variant is set using bitwise AND.
    fn check_flag(&self, flag: Self::Value) -> bool {
        (self.value & (flag as u8)) != 0
    }

    /// Deactivates the bit using bitwise AND with a bitwise NOT mask.
    fn clear_flag(&mut self, flag: Self::Value) {
        self.value &= !(flag as u8);
    }

    /// Flips the state of the bit using bitwise XOR.
    fn toggle_flag(&mut self, flag: Self::Value) {
        self.value ^= flag as u8;
    }

    /// Returns the raw internal state.
    /// 
    /// Note: Since `EnumFlag` is an enum, returning a single variant here 
    /// is technically incorrect if multiple bits are set. 
    /// We return the raw `u8` casted to the Value type if applicable, 
    /// or provide a dedicated method for the raw value.
    fn get_flags(&self) -> Self::Value {
        // In a real-world scenario, you might change the trait to return the raw u8 
        // or use a 'bitflags' crate style approach.
        panic!("Ambiguous return: multiple flags may be set in the underlying u8.");
    }

    /// Overwrites the entire 8-bit mask with the value of a specific enum variant.
    fn set_flags(&mut self, value: Self::Value) {
        self.value = value as u8;
    }
}

/// A bit-mapped representation of allowed flags.
///
/// Using `#[repr(u8)]` ensures the compiler treats these variants as 
/// specific numerical values during casting.

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum EnumFlag {
    /// 0000 0001
    Flag1 = 0b0001,
    /// 0000 0010
    Flag2 = 0b0010,
    /// 0000 0100
    Flag3 = 0b0100,
    /// 0000 1000
    Flag4 = 0b1000,
}