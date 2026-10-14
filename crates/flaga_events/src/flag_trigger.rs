use flaga_core::flag::Flag;

/// `FlagTrigger` is a functional wrapper that associates flag states with executable actions.
///
/// It uses Generics (`F: Flag`) to remain compatible with any type that implements the 
/// `Flag` trait, making it a powerful tool for event-driven architectures.
pub struct FlagTrigger<F: Flag> { 
    /// The underlying flag-storage instance.
    flags: F, 
}

impl<F: Flag> FlagTrigger<F> {
    /// Injects a flag-capable instance into a new Trigger coordinator.
    pub fn new(flags: F) -> Self { 
        Self { flags } 
    }

    /// Conditional Execution: Runs the provided `action` only if the specified `flag` is active.
    /// 
    /// # Example
    /// `trigger.trigger_if(FLAGS_ADMIN, || println!("Access Granted"));`
    pub fn trigger_if(&self, flag: F::Value, action: impl FnOnce()) { 
        if self.flags.check_flag(flag) { 
            action(); 
        } 
    }

    /// Negative Conditional Execution: Runs the provided `action` only if the `flag` is NOT active.
    /// 
    /// Useful for default behaviors or fallback logic.
    pub fn trigger_unless(&self, flag: F::Value, action: impl FnOnce()) { 
        if !self.flags.check_flag(flag) { 
            action(); 
        } 
    }

    /// Atomic-style Operation: Flips the state of the flag and then immediately executes the action.
    ///
    /// This is common in UI toggles or hardware state-switching.
    pub fn toggle_and_trigger(&mut self, flag: F::Value, action: impl FnOnce()) { 
        self.flags.toggle_flag(flag); 
        action(); 
    }

    /// Cleanup Operation: Force-clears the specified flag and then executes the action.
    ///
    /// Useful for "Shutdown" or "De-authorization" sequences.
    pub fn clear_and_trigger(&mut self, flag: F::Value, action: impl FnOnce()) { 
        self.flags.clear_flag(flag); 
        action(); 
    }
}