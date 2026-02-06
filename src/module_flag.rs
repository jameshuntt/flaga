use crate::flag::Flag;

pub struct ModuleFlags {
    pub flags: u64,
}

impl ModuleFlags {
    pub fn new() -> Self {
        Self { flags: 0 }
    }
}

/// `ModuleFlags` implementation of the [`Flag`] trait.
/// 
/// This implementation treats the `Value` as a **positional bit index**. 
/// By utilizing a `u64` as the underlying storage, it provides 64 distinct 
/// addressable slots for boolean states, manipulated through low-level 
/// bitwise arithmetic.
impl Flag for ModuleFlags {
    /// The positional index of the bit to be manipulated (valid range: 0-63).
    type Value = u64;

    /// Activates the bit at the specified index.
    ///
    /// # Logic
    /// Computes an addressable mask using `1 << flag_index` and performs a 
    /// **Bitwise OR** (`|=`). This ensures the target bit becomes `1` while 
    /// all other bits remain in their current state.
    fn set_flag(&mut self, flag_index: Self::Value) {
        self.flags |= 1 << flag_index;
    }

    /// Evaluates whether the bit at the specified index is currently set.
    ///
    /// # Logic
    /// Isolates the target bit by performing a **Bitwise AND** (`&`) between 
    /// the internal state and a computed bit-mask.
    ///
    /// # Returns
    /// `true` if the resulting operation is non-zero (bit is `1`), `false` otherwise.
    fn check_flag(&self, flag_index: Self::Value) -> bool {
        (self.flags & (1 << flag_index)) != 0
    }

    /// Force-deactivates the bit at the specified index.
    ///
    /// # Logic
    /// 1. Creates a mask where only the target bit is `1` (`1 << flag_index`).
    /// 2. Inverts that mask using **Bitwise NOT** (`!`), resulting in a mask of 
    ///    all `1`s with a `0` at the target index.
    /// 3. Performs a **Bitwise AND** (`&=`), effectively "masking out" the target bit.
    
    fn clear_flag(&mut self, flag_index: Self::Value) {
        self.flags &= !(1 << flag_index);
    }

    /// Inverts the current state of the bit at the specified index.
    ///
    /// # Logic
    /// Performs a **Bitwise XOR** (`^=`) against a computed mask. If the bit 
    /// was `0`, it becomes `1`; if it was `1`, it becomes `0`. This is an 
    /// atomic-style toggle that requires no prior knowledge of the state.
    
    fn toggle_flag(&mut self, flag_index: Self::Value) {
        self.flags ^= 1 << flag_index;
    }

    /// Extracts the complete 64-bit state from the container.
    ///
    /// # Returns
    /// The raw `u64` value, representing the composite state of all 64 flags. 
    /// This is the primary format for binary serialization.
    fn get_flags(&self) -> Self::Value {
        self.flags
    }

    /// Overwrites the entire 64-bit internal state with a new bitfield.
    ///
    /// # Arguments
    /// * `value` - A `u64` bitmask representing the new global state of the module.
    fn set_flags(&mut self, value: Self::Value) {
        self.flags = value;
    }
}