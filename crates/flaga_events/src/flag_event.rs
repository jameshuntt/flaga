/// Represents a conditional event tied to a specific bitmask state.
///
/// A `FlagEvent` acts as a listener that becomes relevant only when a set of 
/// bitwise flags (provided via a `u8`) satisfies its `flags_required` mask.
pub struct FlagEvent {
    /// The bitmask representing the specific bits that must be set (1) 
    /// for this event to be eligible for triggering.
    pub flags_required: u8,
    
    /// A human-readable label or payload describing the purpose of this event.
    pub event_description: String,
    
    /// An administrative toggle to enable or disable this event 
    /// regardless of whether the flag requirements are met.
    pub is_active: bool,

    /// The strategy used to determine if the flags match.
    pub strategy: TriggerStrategy,
}

impl FlagEvent {
    /// Constructs a new `FlagEvent` with the specified requirements and description.
    ///
    /// By default, new events are initialized as `is_active: true`.
    ///
    /// # Arguments
    /// * `flags_required` - The bitmask to check against.
    /// * `description` - Anything that can be converted into a `String` (e.g., `&str`).
    pub fn new(flags_required: u8, description: impl Into<String>) -> Self {
        Self {
            flags_required,
            event_description: description.into(),
            is_active: true,
            strategy: TriggerStrategy::AllRequired, // Default to strict matching

        }
    }


    /// Builder-style method to set a specific trigger strategy.
    pub fn with_strategy(mut self, strategy: TriggerStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Evaluates if the event conditions are satisfied by the provided bitmask.
    ///
    /// For this to return `true`, two conditions must be met:
    /// 1. The event itself must be marked as `is_active`.
    /// 2. Every bit set in `self.flags_required` must also be set in `current_flags`.
    ///
    /// # Logic Table (Bitwise AND)
    /// If `flags_required` is `0b0000_0110` (bits 1 and 2):
    /// * `0b0000_0111` -> `true` (bits 1 and 2 are present)
    /// * `0b0000_0010` -> `false` (bit 2 is missing)
    /// Evaluates if the event conditions are satisfied based on the chosen [`TriggerStrategy`].
    ///
    /// # Strategies
    /// * **AllRequired**: `(current & required) == required`
    /// * **AnyRequired**: `(current & required) != 0`
    pub fn should_trigger(&self, current_flags: u8) -> bool {
        if !self.is_active {
            return false;
        }

        match self.strategy {
            TriggerStrategy::AllRequired => {
                (current_flags & self.flags_required) == self.flags_required
            }
            TriggerStrategy::AnyRequired => {
                (current_flags & self.flags_required) != 0
            }
        }
    }

    /// Disables the event logic, preventing `should_trigger` from returning `true`.
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Enables the event logic.
    pub fn activate(&mut self) {
        self.is_active = true;
    }

    /// Formats the event's description and current state into a readable string.
    ///
    /// Returns a string in the format: `"Description [active/inactive]"`
    pub fn describe(&self) -> String {
        let strategy_label = match self.strategy {
            TriggerStrategy::AllRequired => "ALL",
            TriggerStrategy::AnyRequired => "ANY",
        };
        format!(
            "{} [{} | {}]",
            self.event_description,
            if self.is_active { "active" } else { "inactive" },
            strategy_label
        )
    }
}


// crates/binary_layer/flagger/src/flag_event.rs

/// Defines how the bitmask should be evaluated against the required flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerStrategy {
    /// All bits in `flags_required` must be set (Bitwise AND equality).
    AllRequired,
    /// At least one bit in `flags_required` must be set (Bitwise AND non-zero).
    AnyRequired,
}
