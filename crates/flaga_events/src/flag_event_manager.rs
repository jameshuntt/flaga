use crate::flag_event::{FlagEvent, TriggerStrategy};

/// Manages a collection of events that are triggered based on bitwise flag states.
///
/// The `FlagEventManager` maintains a master `current_flags` bitmask. Whenever
/// the flags are updated, it re-evaluates which [`FlagEvent`]s should be active
/// based on their specific bit requirements.
pub struct FlagEventManager {
    /// The registry of all possible events this manager monitors.
    pub events: Vec<FlagEvent>,
    /// The current state of the system represented as a bitmask (8-bit).
    pub current_flags: u8,
}

impl FlagEventManager {
    /// Updates the internal state with new flags and synchronizes the active status of all events.
    ///
    /// This method performs a bitwise AND check for every registered event to determine
    /// if the requirements for that event are currently met.
    ///
    /// # Arguments
    /// * `new_flags` - The updated bitmask to apply to the manager.
    pub fn update_flags(&mut self, new_flags: u8) {
        self.current_flags = new_flags;

        // Iterate through events to update their active state based on current flags.
        // An event is active if ALL bits in flags_required are present in current_flags.
        for event in &mut self.events {
            event.is_active = (self.current_flags & event.flags_required) == event.flags_required;
        }
    }

    /// Returns a list of references to all events whose criteria are currently met.
    pub fn get_active_events(&self) -> Vec<&FlagEvent> {
        self.events.iter().filter(|event| event.is_active).collect()
    }

    /// Registers a new event condition within the manager.
    ///
    /// # Arguments
    /// * `flags_required` - The bitmask that must be present for this event to activate.
    /// * `event_description` - A human-readable identifier or payload for the event.
    pub fn add_event(&mut self, flags_required: u8, event_description: String) {
        self.events.push(FlagEvent {
            flags_required,
            event_description,
            is_active: false,
            strategy: TriggerStrategy::AllRequired
        });
    }

    /// Detects events that were **not** active in a previous state but **are** active now.
    ///
    /// This is useful for "Edge Triggering" (detecting the exact moment a condition becomes true)
    /// rather than "Level Triggering" (checking if a condition is currently true).
    ///
    /// # Arguments
    /// * `prev_flags` - The bitmask from the previous update cycle to compare against.
    ///
    /// # Returns
    /// A vector of references to events that just transitioned from inactive to active.
    pub fn get_triggered_events(&self, prev_flags: u8) -> Vec<&FlagEvent> {
        self.events.iter()
            .filter(|e| {
                let was_active = (prev_flags & e.flags_required) == e.flags_required;
                let is_now_active = (self.current_flags & e.flags_required) == e.flags_required;
                
                !was_active && is_now_active
            })
            .collect()
    }
}