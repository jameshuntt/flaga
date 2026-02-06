use std::collections::HashMap;

/// Defines the interface for a unit of executable logic.
///
/// Any type implementing `Feature` can be registered within a [`FeatureManager`].
/// This trait is designed for the Command Pattern, where the execution logic
/// is encapsulated within the implementing structure.
pub trait Feature {
    /// Executes the feature's core logic.
    ///
    /// # Returns
    /// * `Ok(())` if the execution was successful.
    /// * `Err(String)` containing an error message if the execution failed.
    fn execute(&self) -> Result<(), String>;
}

/// A centralized registry for managing and invoking dynamic features.
///
/// `FeatureManager` stores features mapped by a unique string identifier. 
/// It uses dynamic dispatch (trait objects) to store different concrete types 
/// that implement the [`Feature`] trait.
pub struct FeatureManager {
    /// Internal storage mapping unique names to heap-allocated feature objects.
    features: HashMap<&'static str, Box<dyn Feature>>,
}

impl FeatureManager {
    /// Initializes a new, empty `FeatureManager`.
    ///
    /// # Example
    /// ```
    /// let manager = FeatureManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }

    /// Registers a feature into the manager.
    ///
    /// If a feature with the same name already exists, it will be overwritten.
    ///
    /// # Type Parameters
    /// * `F`: The concrete type of the feature. Must have a `'static` lifetime 
    ///   because it is stored in a `Box` and managed by the `FeatureManager`.
    ///
    /// # Arguments
    /// * `name`: A unique identifier for the feature.
    /// * `feature`: The feature instance to be registered.
    pub fn register_feature<F: Feature + 'static>(&mut self, name: &'static str, feature: F) {
        self.features.insert(name, Box::new(feature));
    }

    /// Looks up a feature by name and triggers its `execute` method.
    ///
    /// # Arguments
    /// * `name`: The identifier of the feature to run.
    ///
    /// # Errors
    /// Returns an `Err` if:
    /// 1. No feature is registered under the provided `name`.
    /// 2. The feature itself fails during execution (returning its own `Err`).
    pub fn execute_feature(&self, name: &'static str) -> Result<(), String> {
        if let Some(feature) = self.features.get(name) {
            feature.execute()
        } else {
            Err(format!("Feature '{}' not found", name))
        }
    }
}