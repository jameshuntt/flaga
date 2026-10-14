/// `impl_flag_trait!` - The Master Flag Factory.
/// 
/// This macro is a "High-Level Orchestrator." It does three critical things:
/// 1. **Defines the Schema**: Creates a private-field struct with essential derives.
/// 2. **Injects Utility**: Calls the "Boilerplate Assassins" (new, default, from, value).
/// 3. **Fulfills the Contract**: Implements the `Flag` trait using direct bitwise masking.
///
/// # Logic Style: Masking
/// Unlike the index-based shifting (1 << i), this macro assumes the `Value` passed in 
/// is a **pre-calculated mask**. This is highly optimized for performance.
///
/// # Arguments
/// * `$name` - The identifier for your new Type (e.g., `NetworkFlags`).
/// * `$type` - The underlying integer width (e.g., `u8`, `u16`, `u32`, `u64`).

#[macro_export]
macro_rules! impl_flag_trait {
    ($name:ident, $type:ty) => {
        /// Generated flag container. 
        /// 
        /// Derives `Debug` for logging, `Clone`/`Copy` for stack-efficiency, 
        /// and `PartialEq`/`Eq` for logical comparisons.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            /// Internal bit-field storage.
            value: $type,
        }

        // --- Boilerplate Injection ---
        // We use $crate to ensure these macros are found even if the user 
        // hasn't explicitly imported them into the current module.
        $crate::impl_new_flag!($name, $type);
        $crate::impl_default!($name, $type);
        $crate::impl_from_flag!($name, $type);
        $crate::impl_flag_value!($name, $type);

        // --- Trait Implementation ---
        impl $crate::flag::Flag for $name {
            /// In this macro, the Value and the Storage type are identical.
            type Value = $type;

            /// **Bitwise OR**: Merges the bits of the mask into the current state.
            fn set_flag(&mut self, flag: Self::Value) { self.value |= flag; }

            /// **Bitwise AND**: Intersects the state with the mask; returns true if any bits overlap.
            fn check_flag(&self, flag: Self::Value) -> bool { self.value & flag != 0 }

            /// **Bitwise AND-NOT**: Clears only the bits specified by the mask.
            fn clear_flag(&mut self, flag: Self::Value) { self.value &= !flag; }

            /// **Bitwise XOR**: Flips the bits specified by the mask.
            fn toggle_flag(&mut self, flag: Self::Value) { self.value ^= flag; }

            /// **Getter**: Returns the raw underlying integer.
            fn get_flags(&self) -> Self::Value { self.value }

            /// **Setter**: Completely replaces the internal bit-field.
            fn set_flags(&mut self, value: Self::Value) { self.value = value; }
        }
    };
}

/// `impl_new_flag!` - The Basic Constructor Generator.
/// 
/// This macro automates the creation of a standard `new()` method for any struct 
/// that follows the "Newtype" pattern (a struct wrapping a single value).
///
/// # Arguments
/// * `$name` - The identifier of your struct (e.g., `BinaryFlag`).
/// * `$type` - The underlying numeric type (e.g., `u8`, `u32`, `u64`).
///
/// # Expansion
/// Generates: `impl Name { pub fn new(v: Type) -> Self { Name { value: v } } }`
#[macro_export]
macro_rules! impl_new_flag {
    ($name:ident, $type:ty) => {
        impl $name {
            pub fn new(v: $type) -> Self {
                $name { value: v }
            }
        }
    };
}

/// `impl_default!` - The Zero-Initializer.
/// 
/// Implements the `Default` trait for a struct, ensuring it starts at a "clean" state.
/// This is vital for bitsets where you want the initial state to be 0 (all flags off).
///
/// # Logic
/// It uses `0 as $type` to ensure the integer literal matches the specific width 
/// of the underlying container, avoiding type-inference ambiguity.
#[macro_export]
macro_rules! impl_default {
    ($name:ident, $type:ty) => {
        impl Default for $name {
            fn default() -> Self {
                Self { value: 0 as $type }
            }
        }
    }
}

/// `impl_flag_value!` - The Getter Generator.
/// 
/// Standardizes how you access the internal raw bits of a flag container.
///
/// # Why use this?
/// By using a `.value()` method instead of making the internal field `pub`, 
/// you maintain better encapsulation while still allowing read-access to the bits.
#[macro_export]
macro_rules! impl_flag_value {
    ($name:ident, $type:ty) => {
        impl $name {
            pub fn value(&self) -> $type {
                self.value
            }
        }
    }
}

/// `impl_from_flag!` - The Conversion Bridge.
/// 
/// Implements the `From<T>` trait, which is the "Golden Standard" in Rust for 
/// type conversion. 
///
/// # Benefits
/// Once implemented, you can use `.into()` or `Name::from(raw_value)` to 
/// wrap your raw integers. This is highly idiomatic and plays perfectly 
/// with generic functions that require `Into<Name>`.
#[macro_export]
macro_rules! impl_from_flag {
    ($name:ident, $type:ty) => {
        impl From<$type> for $name {
            fn from(value: $type) -> Self {
                Self { value }
            }
        }
    };
}

/// `impl_descriptor!` - The Metadata Structure Factory.
/// 
/// This macro generates a standardized "Descriptor" struct. Unlike a raw bitmask, 
/// a Descriptor attaches context (name, metadata, and type-tags) to a flag value.
///
/// # Arguments
/// * `$name`: The identifier for the struct (e.g., `FlagDescriptor`).
/// * `$ty`: The type used for the name (usually `String` or `&'static str`).
/// * `derives`: A flexible list of traits to derive (e.g., `Serialize`, `Deserialize`).
///
/// # Special Syntax: `$( $derive:path ),*`
/// This is a macro "repetition." It allows you to pass a comma-separated list of 
/// traits which the macro will expand into a standard `#[derive(...)]` attribute.

#[macro_export]
macro_rules! impl_descriptor {
    ($name:ident, $ty:ty, derives = [$($derive:path),* $(,)?]) => {
        /// A comprehensive container for a flag's definition and its current state.
        /// 
        /// This struct is generic over `T`, allowing it to hold `u8`, `u64`, 
        /// or even custom enum-based values.
        #[derive(Debug, Clone $(, $derive)* )]
        pub struct $name<T> {
            /// The human-readable identifier (e.g., "ENABLE_LOGGING").
            pub name: $ty,
            /// Categorizes the flag's logic (Binary, Hex, or Enum).
            pub flag_type: $crate::flag_type::FlagType,
            /// The actual bitmask or value associated with this specific flag.
            pub value: T,
            /// Optional context explaining what this flag controls.
            pub description: Option<String>,
            /// Metadata for auditing when this flag was first defined.
            pub created_at: Option<String>,
        }

        impl<T> $name<T> {
            /// The "Full" Constructor.
            /// 
            /// Use this when you have complete metadata, such as loading from 
            /// a database or a configuration file.
            pub fn new(
                name: $ty,
                flag_type: $crate::flag_type::FlagType,
                value: T,
                description: Option<String>,
                created_at: Option<String>,
            ) -> Self {
                Self {
                    name,
                    flag_type,
                    value,
                    description,
                    created_at,
                }
            }
            
            /// The "Lazy" Constructor.
            /// 
            /// Perfect for quick, in-code definitions where you only care 
            /// about the functional parts (name, type, and value).
            pub fn with_defaults(
                name: $ty,
                flag_type: $crate::flag_type::FlagType,
                value: T
            ) -> Self {
                Self {
                    name,
                    flag_type,
                    value,
                    description: None,
                    created_at: None,
                }
            }
        }
    }
}