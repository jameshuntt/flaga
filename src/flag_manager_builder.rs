use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    error::FlagError,
    flag_descriptor::FlagDescriptor,
    flag_manager::FlagManager,
    flag_store::FlagStore,
    flag_type::FlagType,
};

#[cfg(feature = "serde")]
use crate::flag_store::BincodeFileFlagStore;

/// Builder for constructing a [`FlagManager`] with optional persistence.
///
/// This builder supports two modes:
///
/// 1. Fresh manager construction
/// 2. Load-or-default construction from a configured store
///
/// Code-defined descriptors are merged into the manager after load, with
/// builder-supplied definitions taking precedence on name collision.
pub struct FlagManagerBuilder {
    /// Code-defined descriptor set.
    flags: HashMap<String, FlagDescriptor<u64>>,

    /// Optional persistence backend.
    store: Option<Arc<dyn FlagStore>>,

    /// If true, attempt to load from the configured store before merging flags.
    load_from_store: bool,
}

impl Default for FlagManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FlagManagerBuilder {
    /// Creates a new empty builder.
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
            store: None,
            load_from_store: false,
        }
    }

    /// Attach a concrete store backend.
    pub fn with_store<S>(mut self, store: S) -> Self
    where
        S: FlagStore + 'static,
    {
        self.store = Some(Arc::new(store));
        self
    }

    /// Attach a shared store backend.
    pub fn with_store_arc(mut self, store: Arc<dyn FlagStore>) -> Self {
        self.store = Some(store);
        self
    }

    /// Convenience: use a bincode file-backed store.
    #[cfg(feature = "serde")]
    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.store = Some(Arc::new(BincodeFileFlagStore::new(file)));
        self
    }

    /// Configure the builder to load existing persisted state if a store exists.
    ///
    /// For file-backed stores, missing files should generally resolve to a default
    /// empty snapshot at the store layer.
    pub fn load_or_default(mut self) -> Self {
        self.load_from_store = true;
        self
    }

    /// Add a simple flag descriptor with default metadata.
    pub fn add_flag(
        mut self,
        name: impl Into<String>,
        flag_type: FlagType,
        value: u64,
    ) -> Self {
        let name = name.into();
        let descriptor = FlagDescriptor::with_defaults(name.clone(), flag_type, value);
        self.flags.insert(name, descriptor);
        self
    }

    /// Add a fully specified descriptor.
    pub fn add_flag_full(
        mut self,
        name: impl Into<String>,
        flag_type: FlagType,
        value: u64,
        description: Option<String>,
        created_at: Option<String>,
    ) -> Self {
        let name = name.into();
        let descriptor = FlagDescriptor::new(
            name.clone(),
            flag_type,
            value,
            description,
            created_at,
        );
        self.flags.insert(name, descriptor);
        self
    }

    /// Add a prebuilt descriptor directly.
    pub fn add_descriptor(mut self, descriptor: FlagDescriptor<u64>) -> Self {
        self.flags.insert(descriptor.name.clone(), descriptor);
        self
    }
}

impl FlagManagerBuilder {
    /// Build a fresh manager shell without loading persisted state.
    ///
    /// Attached store is preserved, but no load is attempted.
    pub fn build_new_manager(self) -> Result<FlagManager, FlagError> {
        let mut manager = FlagManager::new();

        if let Some(store) = self.store {
            manager = manager.with_store_arc(store);
        }

        for descriptor in self.flags.into_values() {
            manager.upsert_flag_descriptor(descriptor)?;
        }

        // Fresh managers should still be normalized.
        manager.sanitize_masks()?;
        Ok(manager)
    }

    /// Finalize the manager.
    ///
    /// Behavior:
    /// - if `load_or_default()` was set and a store exists:
    ///   - load persisted snapshot
    ///   - persisted-only sanitize has already happened in `from_store_arc`
    /// - merge code-defined descriptors into loaded/fresh state
    /// - sanitize again against the merged descriptor set
    ///
    /// Builder descriptors are authoritative on name collision.
    pub fn build(self) -> Result<FlagManager, FlagError> {
        let FlagManagerBuilder {
            flags,
            store,
            load_from_store,
        } = self;

        let manager = match (load_from_store, store.clone()) {
            (true, Some(store)) => FlagManager::from_store_arc(store)?,
            (_, Some(store)) => FlagManager::new().with_store_arc(store),
            (_, None) => FlagManager::new(),
        };

        // Merge compile-time / code-defined descriptors after load.
        // This allows new deployments to introduce new flags or replace
        // existing definitions intentionally.
        for descriptor in flags.into_values() {
            manager.upsert_flag_descriptor(descriptor)?;
        }

        // Important:
        // - load path already sanitized stale persisted bits against the old registry
        // - after merge, sanitize again against the merged/authoritative registry
        manager.sanitize_masks()?;

        Ok(manager)
    }
}