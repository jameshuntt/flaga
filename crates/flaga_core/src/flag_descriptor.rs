use crate::impl_descriptor;
use alloc::string::String;

// --- Dynamic Descriptor ---
// Uses `String`, meaning names can be created at runtime (e.g., from a database).
// impl_descriptor!(
//     FlagDescriptor,
//     String,
//     derives = [serde::Serialize, serde::Deserialize]
// );
#[cfg(feature = "serde")]
impl_descriptor!(
    FlagDescriptor,
    String,
    derives = [serde::Serialize, serde::Deserialize]
);

#[cfg(not(feature = "serde"))]
impl_descriptor!(
    FlagDescriptor,
    String,
    derives = []
);

/// Type alias for easier readability when using String-based descriptors.
pub type Dynamic<T> = FlagDescriptor<T>;


// --- Static Descriptor ---
// Uses `&'static str`, meaning names are hardcoded in the binary.
// This is faster and uses less memory, but is less flexible.
// impl_descriptor!(
//     FlagDescriptorStatic,
//     &'static str,
//     derives = [serde::Serialize]
// );
#[cfg(feature = "serde")]
impl_descriptor!(
    FlagDescriptorStatic,
    &'static str,
    derives = [serde::Serialize]
);

#[cfg(not(feature = "serde"))]
impl_descriptor!(
    FlagDescriptorStatic,
    &'static str,
    derives = []
);
/// Type alias for easier readability when using hardcoded string-slice descriptors.
pub type Static<T> = FlagDescriptorStatic<T>;