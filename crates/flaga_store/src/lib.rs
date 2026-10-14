// --- Store Modules ---
// Persistence and the thread-safe manager (std::fs, Arc, RwLock).
pub mod flag_manager_builder;// The Fluent Builder API for the manager
pub mod flag_manager;        // The main thread-safe state coordinator
pub mod flag_store;          // Persistence backends (memory, bincode file)
