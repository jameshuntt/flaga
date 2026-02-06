use crate::flag::Flag;

/// A collection of items that implement the [`Flag`] trait.
///
/// `FlagGroup` acts as a container for managing multiple bitwise or logical flags
/// indexed within a vector. It provides a safe interface for manipulating 
/// individual flags while handling bounds checking.
///
/// # Type Parameters
/// * `F`: A type that implements the [`Flag`] trait, defining how values are set/checked.
pub struct FlagGroup<F: Flag> {
    flags: Vec<F>,
}

impl<F: Flag> FlagGroup<F> {
    /// Creates a new, empty `FlagGroup`.
    pub fn new() -> Self {
        Self { flags: Vec::new() }
    }

    /// Appends a new flag to the end of the group.
    ///
    /// # Arguments
    /// * `flag`: The initialized flag instance to add.
    pub fn add_flag(&mut self, flag: F) {
        self.flags.push(flag);
    }

    /// Sets specific bits/values within a flag at the given index.
    ///
    /// # Errors
    /// Returns an `Err("Invalid index")` if the `index` is out of bounds.
    pub fn set_flag(&mut self, index: usize, flag_value: F::Value) -> Result<(), &'static str> {
        if let Some(flag) = self.flags.get_mut(index) {
            flag.set_flag(flag_value);
            Ok(())
        } else {
            Err("Invalid index")
        }
    }

    /// Checks if specific bits/values are active in the flag at the given index.
    ///
    /// # Returns
    /// * `Ok(true)` if the flag condition is met.
    /// * `Ok(false)` if the flag condition is not met.
    /// * `Err` if the index does not exist.
    pub fn check_flag(&self, index: usize, flag_value: F::Value) -> Result<bool, &'static str> {
        if let Some(flag) = self.flags.get(index) {
            Ok(flag.check_flag(flag_value))
        } else {
            Err("Invalid index")
        }
    }

    /// Clears (unsets) specific bits/values in the flag at the given index.
    ///
    /// # Errors
    /// Returns an `Err` if the `index` is out of bounds.
    pub fn clear_flag(&mut self, index: usize, flag_value: F::Value) -> Result<(), &'static str> {
        if let Some(flag) = self.flags.get_mut(index) {
            flag.clear_flag(flag_value);
            Ok(())
        } else {
            Err("Invalid index")
        }
    }

    /// Toggles the state of specific bits/values in the flag at the given index.
    ///
    /// If the bit is 1, it becomes 0; if it is 0, it becomes 1.
    pub fn toggle_flag(&mut self, index: usize, flag_value: F::Value) -> Result<(), &'static str> {
        if let Some(flag) = self.flags.get_mut(index) {
            flag.toggle_flag(flag_value);
            Ok(())
        } else {
            Err("Invalid index")
        }
    }

    /// Retrieves the raw underlying value of the flag at the given index.
    pub fn get_flags(&self, index: usize) -> Result<F::Value, &'static str> {
        if let Some(flag) = self.flags.get(index) {
            Ok(flag.get_flags())
        } else {
            Err("Invalid index")
        }
    }

    /// Overwrites the entire value of the flag at the given index.
    ///
    /// Unlike `set_flag`, which usually performs a bitwise OR, this replaces the full state.
    pub fn set_flags(&mut self, index: usize, value: F::Value) -> Result<(), &'static str> {
        if let Some(flag) = self.flags.get_mut(index) {
            flag.set_flags(value);
            Ok(())
        } else {
            Err("Invalid index")
        }
    }

    /// Returns an iterator over immutable references to the flags in this group.
    pub fn iter(&self) -> impl Iterator<Item = &F> {
        self.flags.iter()
    }

    /// Returns an iterator over mutable references to the flags in this group.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut F> {
        self.flags.iter_mut()
    }

    /// Returns the number of flags currently in the group.
    pub fn len(&self) -> usize {
        self.flags.len()
    }

    /// Returns `true` if the group contains no flags.
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }
}