use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock}
};

use crate::{
    flag_descriptor::FlagDescriptor, 
    flag_manager::{FlagManager, Inner}, 
    flag_type::FlagType
};

/// A builder for constructing a [`FlagManager`] with optional persistence support.
/// 
/// This builder implements a "Load-or-Create" strategy. If a file path is provided 
/// and the file exists, it will attempt to restore the state from disk. Otherwise, 
/// it initializes a fresh manager with the provided flag definitions.
pub struct FlagManagerBuilder {
    /// Temporary storage for flag descriptors before the manager is finalized.
    /// Internal collection of flag descriptors.
    flags: HashMap<String, FlagDescriptor<u64>>,
    /// Path used for persistence if file-backed storage is required.
    file_path: Option<PathBuf>,
}

impl Default for FlagManagerBuilder {
    /// Provides a default, empty builder state.
    fn default() -> Self {
        Self::new()
    }
}

impl FlagManagerBuilder {
    /// Creates a new `FlagManagerBuilder` with empty defaults.
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
            file_path: None,
        }
    }

    /// Configures a file path for the manager to use for I/O operations.
    ///
    /// # Arguments
    /// * `file` - The path to the configuration or state file.
    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file_path = Some(file);
        self
    }

    /// Adds a new flag definition to the manager.
    ///
    /// This method is polymorphic over the name, accepting both `String` 
    /// and `&str` for convenience.
    ///
    /// # Arguments
    /// * `name` - The unique identifier for this flag.
    /// * `flag_type` - The categorization of the flag (Binary, Hex, Enum).
    /// * `value` - The initial value to be assigned to this flag.
    pub fn add_flag(mut self, name: impl Into<String>, flag_type: FlagType, value: u64) -> Self {
        let name_str = name.into();
        let descriptor = FlagDescriptor::with_defaults(name_str.clone(), flag_type, value);
        self.flags.insert(name_str, descriptor);
        self
    }
}

impl FlagManagerBuilder {
    /// Creates a fresh [`FlagManager`] ignoring any existing disk state.
    ///
    /// This is used internally by [`Self::build`] as a fallback, but can be called 
    /// directly if you want to explicitly bypass loading from a file.
    ///
    /// # Returns
    /// A new [`FlagManager`] with empty bitmasks and the builder's configured flags.
    pub fn build_new_manager(self) -> FlagManager {
        FlagManager {
            file_path: self.file_path,
            file_extension_flags: HashMap::new(),
            extension_flags: HashMap::new(),
            // Arc allows shared ownership; RwLock allows multiple readers OR one writer.
            state: Arc::new(RwLock::new(Inner {
                binary_flags: 0,
                hex_flags: 0,
                enum_flags: 0,
                flags: self.flags,
            })),
        }
    }

    /// Finalizes the `FlagManager` using a "smart-load" strategy.
    ///
    /// **Execution Steps:**
    /// 1. **Check Persistence:** If `file_path` is set and the file exists on disk...
    /// 2. **Attempt Load:** Try to deserialize the existing [`Inner`] state.
    /// 3. **Merge State:** If load succeeds, merge the code-defined flags into the loaded definitions.
    /// 4. **Fallback:** If any step fails or no file exists, call [`Self::build_new_manager`].
    pub fn build(self) -> FlagManager {
        // Attempt to load existing state if file exists
        if let Some(ref path) = self.file_path {
            if path.exists() {
                if let Ok(manager) = FlagManager::load_from_file(path) {
                    // Update the manager's path in case the file was moved
//                     let mut final_manager = manager;
//                     final_manager.file_path = self.file_path.clone();
//                     
//                     // Merge builder flags (newly defined in code) into the persistent state
//                     let mut inner = final_manager.state.write().unwrap();
// //                     inner.flags.extend(self.flags);
// //                     
// //                     // Release lock and return the manager
// //                     drop(inner);
// //                     return final_manager;
//                     inner.flags.extend(self.flags);
//                     drop(inner);
// 
//                     // critical: clear unknown persisted bits after the descriptor set changes
//                     final_manager.sanitize_masks();
// 
//                     return final_manager;
                    // AFTER (correct)
let mut final_manager = manager;
final_manager.file_path = self.file_path.clone();

// ✅ sanitize while only the *persisted* descriptor set exists
// This clears bits that were not defined in the saved file’s flag registry.
final_manager.sanitize_masks();

// Now merge new code-defined flags (deploy-time additions)
let mut inner = final_manager.state.write().unwrap();
inner.flags.extend(self.flags);
drop(inner);

// Optional: sanitize again if you want to clear bits not in (persisted ∪ new)
// but it won’t “re-add” anything.
final_manager.sanitize_masks();

return final_manager;
                }
            }
        }
        
        // No file found or load failed: create from scratch
        self.build_new_manager()
    }
}