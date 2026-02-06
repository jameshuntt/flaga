// /// Generates the `BinaryFlag` struct using an 8-bit unsigned integer.
// /// Ideal for high-density, low-memory boolean toggles.
impl_flag_trait!(BinaryFlag, u8);

// /// Generates the `HexFlag` struct using a 64-bit unsigned integer.
// /// Designed for complex feature flagging and large bitmask configurations.
impl_flag_trait!(HexFlag, u64);