use crate::{
    error::FlagError,
    flag_descriptor::FlagDescriptor,
    flag_type::FlagType
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock}
};
use {
    log::info,
    serde::{Serialize, Deserialize}
};

// --- Data Structures ---

/// The thread-safe state container for flags and their definitions.
/// 
/// This is the "Single Source of Truth." By housing the bitmasks and the 
/// flag definitions together, we ensure that updates to flag values 
/// are always synchronized with their type-metadata.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) struct Inner {
    /// 8-bit mask for binary/boolean logic.
    pub(crate) binary_flags: u8,
    /// 64-bit mask for complex or large hex-based flags.
    pub(crate) hex_flags: u64,
    /// 8-bit mask reserved for enumerable state flags.
    pub(crate) enum_flags: u8,
    /// A registry of flag names and their associated descriptors.
    pub(crate) flags: HashMap<String, FlagDescriptor<u64>>,
}

/// A private helper struct used to define the disk-persisted format of [`FlagManager`].
/// 
/// Since `Arc<RwLock<T>>` does not implement `Serialize`/`Deserialize` by default,
/// this "Shadow" struct acts as a snapshot of the manager's data without the 
/// synchronization primitives.
#[derive(Serialize, Deserialize)]
struct FlagManagerShadow {
    state: Inner,
    file_path: Option<PathBuf>,
    file_extension_flags: HashMap<String, Vec<FlagDescriptor<u64>>>,
    extension_flags: HashMap<String, Vec<String>>,
}

/// A thread-safe coordinator for binary, hex, and enum-based flags.
///
/// `FlagManager` enables shared ownership across threads via `Arc` and 
/// fine-grained access control via `RwLock`. It supports dynamic flag 
/// registration, bitwise state manipulation, and file-based persistence.
#[derive(Debug, Clone)]
pub struct FlagManager {
    /// The synchronized internal state.
    pub(crate) state: Arc<RwLock<Inner>>,
    /// The default file path used for [`Self::sync_to_disk`].
    pub(crate) file_path: Option<PathBuf>,
    /// Mappings of file extensions to specific flag descriptors.
    pub(crate) file_extension_flags: HashMap<String, Vec<FlagDescriptor<u64>>>,
    /// Mappings of file extensions to lists of flag names.
    pub(crate) extension_flags: HashMap<String, Vec<String>>,
}

// --- Trait Implementations ---

impl Serialize for FlagManager {
    /// Serializes the manager by taking a read-lock snapshot of the current state.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer 
    {
        // Lock the state for a moment to take a snapshot for saving
        let inner = self.state.read().map_err(serde::ser::Error::custom)?;
        let shadow = FlagManagerShadow {
            state: inner.clone(),
            file_path: self.file_path.clone(),
            file_extension_flags: self.file_extension_flags.clone(),
            extension_flags: self.extension_flags.clone(),
        };
        shadow.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FlagManager {
    /// Deserializes the manager and re-establishes the internal `RwLock`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> 
    {
        let shadow = FlagManagerShadow::deserialize(deserializer)?;
        Ok(Self {
            state: Arc::new(RwLock::new(shadow.state)),
            file_path: shadow.file_path,
            file_extension_flags: shadow.file_extension_flags,
            extension_flags: shadow.extension_flags,
        })
    }
}

// --- Initialization & Configuration ---

impl FlagManager {
    /// Creates a new, empty `FlagManager`.
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(Inner::default())),
            file_path: None,
            file_extension_flags: HashMap::new(),
            extension_flags: HashMap::new(),
        }
    }

    /// Appends a descriptor directly to a specific file extension mapping.
    pub fn add_extension_flag<S1, S2>(
        mut self, extension: S1, name: S2, flag_type: FlagType, value: u64
    ) -> Self where S1: Into<String>, S2: Into<String> {
        let descriptor = FlagDescriptor::new(name.into(), flag_type, value, None, None);
        self.file_extension_flags
            .entry(extension.into())
            .or_insert_with(Vec::new)
            .push(descriptor);
        self
    }

    /// Sets the file path for persistence operations.
    pub fn with_file_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = Some(path.into());
        self
    }
}

// --- Operational Logic ---

impl FlagManager {
    /// Registers a new flag definition into the manager.
    /// 
    /// # Errors
    /// Returns [`FlagError::FlagAlreadyExists`] if a flag with the same name is already registered.
    pub fn add_flag(
        &self,
        name: &str,
        flag_type: FlagType,
        value: u64,
        description: Option<String>,
        created_at: Option<String>,
    ) -> Result<(), FlagError> {
        let mut inner = self.state.write().unwrap();
        if inner.flags.contains_key(name) {
            return Err(FlagError::FlagAlreadyExists);
        }
        let flag = FlagDescriptor::new(name.to_string(), flag_type, value, description, created_at);
        inner.flags.insert(name.to_string(), flag);
        info!("Registered flag: {}", name);
        Ok(())
    }

    /// Activates a specific flag using bitwise OR logic.
    ///
    /// # Errors
    /// Returns [`FlagError::NotFound`] if the flag name has not been registered.
    pub fn set_flag(&self, name: &str) -> Result<(), FlagError> {
        let mut inner = self.state.write().unwrap();
        
        // We must find the definition to know the bit value and type
        let (val, f_type) = inner.flags.get(name)
            .map(|f| (f.value, f.flag_type))
            .ok_or_else(|| FlagError::NotFound(name.to_string()))?;

        match f_type {
            FlagType::Binary => inner.binary_flags |= val as u8,
            FlagType::Hex => inner.hex_flags |= val,
            FlagType::Enum => inner.enum_flags |= val as u8,
        }
        info!("Set active bit(s) for: {}", name);
        Ok(())
    }

    /// Checks if the bits associated with a flag name are currently set.
    pub fn check_flag(&self, name: &str) -> Result<bool, FlagError> {
        let inner = self.state.read().unwrap();
        let flag = inner.flags.get(name).ok_or_else(|| FlagError::NotFound(name.to_string()))?;
        
        let is_set = match flag.flag_type {
            FlagType::Binary => (inner.binary_flags & flag.value as u8) != 0,
            FlagType::Hex => (inner.hex_flags & flag.value) != 0,
            FlagType::Enum => (inner.enum_flags & flag.value as u8) != 0,
        };
        Ok(is_set)
    }
}

// --- Persistence Logic ---

#[cfg(not(feature = "inner_only"))]
impl FlagManager {
    /// Serializes and saves the **entire** manager state to a file.
    ///
    /// This includes all bitmasks, flag definitions, and extension mappings.
    pub fn save_to_file(&self, path: &Path) -> Result<(), FlagError> {
        // Uses the custom Serialize implementation (via the Shadow struct)
        let encoded = bincode::serialize(self).map_err(|e| FlagError::NotFound(e.to_string()))?;
        std::fs::write(path, encoded).map_err(|e| FlagError::NotFound(e.to_string()))?;
        Ok(())
    }
    
    /// Loads a full `FlagManager` state from a file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, FlagError> {
        let path_ref = path.as_ref();
        let data = std::fs::read(path_ref).map_err(|e| FlagError::NotFound(e.to_string()))?;
        let decoded: Self = bincode::deserialize(&data).map_err(|e| FlagError::NotFound(e.to_string()))?;
        Ok(decoded)
    }
}

#[cfg(feature = "inner_only")]
impl FlagManager {
    /// **Data-Only Persistence:** Saves only the bitmasks and descriptors.
    pub fn save_inner_to_file(&self, path: &Path) -> Result<(), FlagError> {
        let inner = self.state.read().unwrap();
        let encoded = bincode::serialize(&*inner)?;
        std::fs::write(path, encoded)?;
        Ok(())
    }

    /// **Data-Only Persistence:** Loads bitmasks into a fresh manager shell.
    /// Attempts to load a `FlagManager` state from a file.
    ///
    /// If the file is found, it deserializes the `Inner` state and returns 
    /// a manager configured with that state and the provided path.
    ///
    /// # Errors
    /// Returns `FlagError` if the file is corrupted or unreachable.
    pub fn load_inner_from_file<P: AsRef<Path>>(path: P) -> Result<Self, FlagError> {
        let path_ref = path.as_ref();
        let data = std::fs::read(path_ref)?;
        
        // Deserialize the Source of Truth
        let decoded: Inner = bincode::deserialize(&data)?;

        Ok(Self {
            state: Arc::new(RwLock::new(decoded)),
            file_path: Some(path_ref.to_path_buf()),
            file_extension_flags: HashMap::new(),
            extension_flags: HashMap::new(),
        })
    }
}

impl FlagManager {
    /// Reloads the internal state of the current manager from its `file_path`.
    ///
    /// This is useful if an external process has modified the state file 
    /// and you need to synchronize.
    pub fn refresh(&self) -> Result<(), FlagError> {
        if let Some(ref path) = self.file_path {
            let fresh = Self::load_from_file(path)?;
            let mut inner = self.state.write().unwrap();
            let fresh_inner = fresh.state.read().unwrap();
            *inner = fresh_inner.clone();
            info!("State refreshed from disk.");
            Ok(())
        } else {
            Err(FlagError::NotFound("no path provided".to_string()))
        }
    }

    /// Saves the current state to the pre-configured `file_path`.
    /// 
    /// Returns an error if `file_path` was never set.
    pub fn sync_to_disk(&self) -> Result<(), FlagError> {
        if let Some(ref path) = self.file_path {
            self.save_to_file(path)
        } else {
            Err(FlagError::NotFound("no path provided".to_string()))
        }
    }
}
