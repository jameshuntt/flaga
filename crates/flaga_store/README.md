# flaga_store

The persistence and management layer of flaga: the `FlagStore` trait with `MemoryFlagStore` and `BincodeFileFlagStore`, the thread-safe `FlagManager` (`Arc<RwLock<_>>`), and `FlagManagerBuilder`. std only. Features `serde`, `json`, `bincode`, `logging`, `inner_only`.

Most people want the [`flaga`](https://crates.io/crates/flaga) facade crate, which re-exports this crate under the old paths.

Licensed under MIT or Apache-2.0, at your option.
