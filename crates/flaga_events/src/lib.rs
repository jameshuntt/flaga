// --- Event Modules ---
// Event triggering on top of the flaga_core flag types.
pub mod features;            // Feature-gate management
pub mod flag_event_manager;  // Coordinator for triggered flag events
pub mod flag_event;          // Individual event definitions
pub mod flag_trigger;        // Functional wrappers for conditional execution
