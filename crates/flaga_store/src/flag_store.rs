
use flaga_core::{error::FlagError, state::Inner};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::RwLock,
};

/// Object-safe persistence boundary.
///
/// This lets `FlagManager` depend on "a thing that can save/load Inner"
/// without caring whether that thing is:
/// - bincode file storage
/// - JSON file storage
/// - SQLite
/// - memory-only test storage
/// - remote store
pub trait FlagStore: Send + Sync + Debug {
    fn save(&self, data: &Inner) -> Result<(), FlagError>;
    fn load(&self) -> Result<Inner, FlagError>;
}

/// Useful for tests.
#[derive(Debug, Default)]
pub struct MemoryFlagStore {
    slot: RwLock<Inner>,
}

impl MemoryFlagStore {
    pub fn new() -> Self {
        Self {
            slot: RwLock::new(Inner::default()),
        }
    }

    pub fn from_inner(inner: Inner) -> Self {
        Self {
            slot: RwLock::new(inner),
        }
    }
}

impl FlagStore for MemoryFlagStore {
    fn save(&self, data: &Inner) -> Result<(), FlagError> {
        let mut slot = self
            .slot
            .write()
            .map_err(|_| FlagError::LockPoisoned)?;
        *slot = data.clone();
        Ok(())
    }

    fn load(&self) -> Result<Inner, FlagError> {
        let slot = self
            .slot
            .read()
            .map_err(|_| FlagError::LockPoisoned)?;
        Ok(slot.clone())
    }
}

#[cfg(feature = "serde")]
#[derive(Debug, Clone)]
pub struct BincodeFileFlagStore {
    path: PathBuf,
}

#[cfg(feature = "serde")]
impl BincodeFileFlagStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(feature = "serde")]
impl FlagStore for BincodeFileFlagStore {
    fn save(&self, data: &Inner) -> Result<(), FlagError> {
        let encoded = bincode::serialize(data)
            .map_err(|e|
                // FlagError::Encode(e.to_string())
                FlagError::Encode { item: e.to_string() }
            )?;

        std::fs::write(&self.path, encoded)?;
        Ok(())
    }

    fn load(&self) -> Result<Inner, FlagError> {
        let bytes = std::fs::read(&self.path)?;
        let decoded = bincode::deserialize::<Inner>(&bytes)
            .map_err(|e|
                // FlagError::Decode(e.to_string())
                FlagError::Decode { item: e.to_string() }
            )?;
        Ok(decoded)
    }
}
