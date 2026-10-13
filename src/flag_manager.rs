
use crate::{
    error::FlagError,
    flag_descriptor::FlagDescriptor,
    flag_store::FlagStore,
    flag_type::FlagType,
    state::Inner,
};
#[cfg(feature = "logging")]
use log::info;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct FlagManager {
    pub(crate) state: Arc<RwLock<Inner>>,
    pub(crate) file_extension_flags: HashMap<String, Vec<FlagDescriptor<u64>>>,
    pub(crate) extension_flags: HashMap<String, Vec<String>>,
    pub(crate) store: Option<Arc<dyn FlagStore>>,
}

impl Default for FlagManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FlagManager {
    /// Empty manager with no persistence backend attached.
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(Inner::default())),
            file_extension_flags: HashMap::new(),
            extension_flags: HashMap::new(),
            store: None,
        }
    }

    /// Attach a persistence backend.
    pub fn with_store<S>(mut self, store: S) -> Self
    where
        S: FlagStore + 'static,
    {
        self.store = Some(Arc::new(store));
        self
    }

    /// Replace the persistence backend after construction.
    pub fn set_store<S>(&mut self, store: S)
    where
        S: FlagStore + 'static,
    {
        self.store = Some(Arc::new(store));
    }

    /// Build a manager from a backend's previously persisted state.
    pub fn from_store<S>(store: S) -> Result<Self, FlagError>
    where
        S: FlagStore + 'static,
    {
        let store = Arc::new(store);
        let inner = store.load()?;

        let manager = Self {
            state: Arc::new(RwLock::new(inner)),
            file_extension_flags: HashMap::new(),
            extension_flags: HashMap::new(),
            store: Some(store),
        };

        manager.sanitize_masks()?;
        Ok(manager)
    }

    /// Snapshot current pure state.
    pub fn snapshot(&self) -> Result<Inner, FlagError> {
        let inner = self
            .state
            .read()
            .map_err(|_| FlagError::LockPoisoned)?;
        Ok(inner.clone())
    }

    /// Restore from a pure state snapshot.
    pub fn restore(&self, inner: Inner) -> Result<(), FlagError> {
        let mut current = self
            .state
            .write()
            .map_err(|_| FlagError::LockPoisoned)?;
        *current = inner;
        Ok(())
    }

    pub fn add_extension_flag<S1, S2>(
        mut self,
        extension: S1,
        name: S2,
        flag_type: FlagType,
        value: u64,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let descriptor = FlagDescriptor::new(name.into(), flag_type, value, None, None);

        self.file_extension_flags
            .entry(extension.into())
            .or_insert_with(Vec::new)
            .push(descriptor);

        self
    }

    pub fn add_flag(
        &self,
        name: &str,
        flag_type: FlagType,
        value: u64,
        description: Option<String>,
        created_at: Option<String>,
    ) -> Result<(), FlagError> {
        let mut inner = self
            .state
            .write()
            .map_err(|_| FlagError::LockPoisoned)?;

        if inner.flags.contains_key(name) {
            return Err(FlagError::FlagAlreadyExists);
        }

        let flag = FlagDescriptor::new(
            name.to_string(),
            flag_type,
            value,
            description,
            created_at,
        );

        inner.flags.insert(name.to_string(), flag);
        
        #[cfg(feature = "logging")]
        info!("Registered flag: {}", name);

        Ok(())
    }

    pub fn set_flag(&self, name: &str) -> Result<(), FlagError> {
        let mut inner = self
            .state
            .write()
            .map_err(|_| FlagError::LockPoisoned)?;

        let (val, f_type) = inner
            .flags
            .get(name)
            .map(|f| (f.value, f.flag_type))
            .ok_or_else(||
                // FlagError::NotFound(name.to_string())
                FlagError::NotFound { name: name.to_string() }
            )?;

        match f_type {
            FlagType::Binary => inner.binary_flags |= val as u8,
            FlagType::Hex => inner.hex_flags |= val,
            FlagType::Enum => inner.enum_flags |= val as u8,
        }

        #[cfg(feature = "logging")]
        info!("Set active bit(s) for: {}", name);

        Ok(())
    }

    pub fn clear_flag(&self, name: &str) -> Result<(), FlagError> {
        let mut inner = self
            .state
            .write()
            .map_err(|_| FlagError::LockPoisoned)?;

        let (val, f_type) = inner
            .flags
            .get(name)
            .map(|f| (f.value, f.flag_type))
            .ok_or_else(||
                // FlagError::NotFound(name.to_string())
                FlagError::NotFound { name: name.to_string() }
            )?;
        match f_type {
            FlagType::Binary => inner.binary_flags &= !(val as u8),
            FlagType::Hex => inner.hex_flags &= !val,
            FlagType::Enum => inner.enum_flags &= !(val as u8),
        }

        Ok(())
    }

    pub fn check_flag(&self, name: &str) -> Result<bool, FlagError> {
        let inner = self
            .state
            .read()
            .map_err(|_| FlagError::LockPoisoned)?;

        let flag = inner
            .flags
            .get(name)
            .ok_or_else(||
                // FlagError::NotFound(name.to_string())
                FlagError::NotFound { name: name.to_string() }
            )?;
        let is_set = match flag.flag_type {
            FlagType::Binary => (inner.binary_flags & flag.value as u8) != 0,
            FlagType::Hex => (inner.hex_flags & flag.value) != 0,
            FlagType::Enum => (inner.enum_flags & flag.value as u8) != 0,
        };

        Ok(is_set)
    }

    /// Save current `Inner` through the attached store.
    pub fn sync(&self) -> Result<(), FlagError> {
        let store = self.store.as_ref().ok_or(FlagError::NoStoreConfigured)?;
        let snapshot = self.snapshot()?;
        store.save(&snapshot)
    }

    /// Reload `Inner` through the attached store.
    pub fn refresh(&self) -> Result<(), FlagError> {
        let store = self.store.as_ref().ok_or(FlagError::NoStoreConfigured)?;
        let loaded = store.load()?;
        self.restore(loaded)?;
        self.sanitize_masks()?;
        
        #[cfg(feature = "logging")]
        info!("State refreshed from store.");

        Ok(())
    }

    /// Clears any bits that no longer correspond to registered descriptors.
    pub fn sanitize_masks(&self) -> Result<(), FlagError> {
        let mut inner = self
            .state
            .write()
            .map_err(|_| FlagError::LockPoisoned)?;

        let mut allowed_binary: u8 = 0;
        let mut allowed_hex: u64 = 0;
        let mut allowed_enum: u8 = 0;

        for d in inner.flags.values() {
            match d.flag_type {
                FlagType::Binary => allowed_binary |= d.value as u8,
                FlagType::Hex => allowed_hex |= d.value,
                FlagType::Enum => allowed_enum |= d.value as u8,
            }
        }

        inner.binary_flags &= allowed_binary;
        inner.hex_flags &= allowed_hex;
        inner.enum_flags &= allowed_enum;

        Ok(())
    }
}

#[cfg(test)]
impl FlagManager {
    pub fn __test_inject_masks(&self, binary: u8, hex: u64, en: u8) {
        let mut inner = self.state.write().unwrap();
        inner.binary_flags = binary;
        inner.hex_flags = hex;
        inner.enum_flags = en;
    }
}



// impl FlagManager {
//     /// Attach a pre-built shared store object.
//     pub fn with_store_arc(mut self, store: Arc<dyn FlagStore>) -> Self {
//         self.store = Some(store);
//         self
//     }

//     /// Builder/internal helper: insert or replace a descriptor definition.
//     ///
//     /// This is intentionally separate from `add_flag`, because builders often
//     /// need "code is authoritative" behavior when merging loaded state with
//     /// compile-time definitions.
//     pub(crate) fn upsert_flag_descriptor(
//         &self,
//         descriptor: FlagDescriptor<u64>,
//     ) -> Result<(), FlagError> {
//         let mut state = self.state.write().map_err(|_| FlagError::LockPoisoned)?;
//         let key = descriptor.name.clone();
//         state.snapshot.inner.flags.insert(key, descriptor);
//         state.dirty = true;
//         Ok(())
//     }

//     pub fn from_store_arc(store: Arc<dyn FlagStore>) -> Result<Self, FlagError> {
//         let mut snapshot = store.load()?;
//         validate_snapshot_version(&snapshot)?;

//         let changed = sanitize_snapshot_masks(&mut snapshot);

//         Ok(Self {
//             state: Arc::new(RwLock::new(StateCell {
//                 snapshot,
//                 dirty: changed,
//             })),
//             file_extension_flags: HashMap::new(),
//             extension_flags: HashMap::new(),
//             store: Some(store),
//         })
//     }
// }


impl FlagManager {
    pub fn with_store_arc(mut self, store: Arc<dyn FlagStore>) -> Self {
        self.store = Some(store);
        self
    }

    pub(crate) fn upsert_flag_descriptor(
        &self,
        descriptor: FlagDescriptor<u64>,
    ) -> Result<(), FlagError> {
        let mut inner = self
            .state
            .write()
            .map_err(|_| FlagError::LockPoisoned)?;

        inner.flags.insert(descriptor.name.clone(), descriptor);
        Ok(())
    }

    pub fn from_store_arc(store: Arc<dyn FlagStore>) -> Result<Self, FlagError> {
        let inner = store.load()?;

        let manager = Self {
            state: Arc::new(RwLock::new(inner)),
            file_extension_flags: HashMap::new(),
            extension_flags: HashMap::new(),
            store: Some(store),
        };

        manager.sanitize_masks()?;
        Ok(manager)
    }
}




#[cfg(feature = "serde")]
use std::path::{Path, PathBuf};

#[cfg(feature = "serde")]
use crate::flag_store::BincodeFileFlagStore;


impl FlagManager {
    #[cfg(feature = "serde")]
    pub fn with_file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.store = Some(Arc::new(BincodeFileFlagStore::new(path)));
        self
    }

    pub fn sync_to_disk(&self) -> Result<(), FlagError> {
        self.sync()
    }

    #[cfg(feature = "serde")]
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, FlagError> {
        Self::from_store(BincodeFileFlagStore::new(path.as_ref().to_path_buf()))
    }
}