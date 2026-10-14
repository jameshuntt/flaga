# flaga_core

The core of flaga: the `Flag` trait, the `BinaryFlag` / `HexFlag` containers, `ModuleFlags`, `FlagType`, flag descriptors, `FlagGroup`, `FlagParser`, the persisted `state::Inner`, the `impl_flag_trait!` family of macros and `FlagError`.

`#![no_std]` with `alloc`. The `std` feature (on by default) adds `flag_parser`, `state` and the `FlagError::IOError` variant. Optional `serde`, `json` and `bincode` features.

Most people want the [`flaga`](https://crates.io/crates/flaga) facade crate, which re-exports this crate under the old paths.

Licensed under MIT or Apache-2.0, at your option.
