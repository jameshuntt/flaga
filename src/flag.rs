/// `Flag` defines the standard interface for bitwise or state-based flag manipulation.
///
/// This trait abstracts the underlying data storage, allowing the same logic to work 
/// across different numeric widths (u8, u32, u64) or even specialized bitsets.
pub trait Flag {
    /// The associated type representing the numeric or bitwise container.
    /// It must implement `Copy` (for easy passing) and `Debug` (for logging).
    type Value: Copy + std::fmt::Debug;

    /// Activates a specific bit or state within the flag container.
    /// Usually maps to a bitwise OR (`|=`) operation.
    fn set_flag(&mut self, flag: Self::Value);

    /// Validates whether a specific bit or state is currently active.
    /// Usually maps to a bitwise AND (`&`) check against the bitmask.
    fn check_flag(&self, flag: Self::Value) -> bool;

    /// Deactivates a specific bit or state.
    /// Usually maps to a bitwise AND-NOT (`&= !`) operation.
    fn clear_flag(&mut self, flag: Self::Value);

    /// Flips the state of a specific bit: if it was active, it becomes inactive, and vice versa.
    /// Usually maps to a bitwise XOR (`^=`) operation.
    fn toggle_flag(&mut self, flag: Self::Value);

    /// Retrieves the raw representation of all currently active flags.
    /// Useful for serialization or low-level comparisons.
    fn get_flags(&self) -> Self::Value;

    /// Overwrites the entire internal state with a provided raw value.
    /// Useful for restoring state from a database or file.
    fn set_flags(&mut self, value: Self::Value);
}