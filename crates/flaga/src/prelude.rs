// Core traits
pub use crate::flag::Flag;
pub use crate::features::Feature;

// Managers / coordinators
pub use crate::features::FeatureManager;
pub use crate::flag_manager::FlagManager;
pub use crate::flag_manager_builder::FlagManagerBuilder;
pub use crate::flag_event_manager::FlagEventManager;
pub use crate::flag_trigger::FlagTrigger;

// Types / metadata
pub use crate::error::FlagError;
pub use crate::flag_type::FlagType;
pub use crate::flag_descriptor::{Dynamic, Static};

// Common flag containers
pub use crate::module_flag::ModuleFlags;
pub use crate::flag_types::{BinaryFlag, HexFlag};

// Optional “engine” utilities (include if you expect most users to want them)
pub use crate::flag_event::{FlagEvent, TriggerStrategy};
pub use crate::flag_parser::FlagParser;

// Enum-based example types (include only if you want to “bless” them as public API)
// pub use crate::enum_flag_counter::{EnumFlag, EnumFlagContainer};