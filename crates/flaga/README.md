# 🚀 flaga: High-Performance Binary Flag System

**flaga** is a robust, thread-safe Rust library designed for managing complex feature flags, permissions, and system states using optimized bitwise arithmetic. It bridges the gap between low-level performance and high-level developer ergonomics.


### flaga is polish for flag, i was thinkin about a name, but many things were taken so i sought out the polish word and ran with it


---

## 🏗️ Core Philosophy

1.  **Zero Boilerplate:** Use macros to generate entire flag structures in a single line.
2.  **Thread Safety:** Built-in support for concurrent access using `Arc` and `RwLock`.
3.  **Persistence:** Built-in support for binary (`bincode`) and human-readable (`JSON`) serialization.
4.  **Flexibility:** Choose between **Static** (stack-allocated) and **Dynamic** (heap-allocated) flag descriptors.

---

## 📦 Quick Start

### 1. Define Your Flags
Using the `impl_flag_trait!` macro, you can create a new flag type with all bitwise operations ready to go.

```rust
use flagger::impl_flag_trait;

// Create a 64-bit flag container
impl_flag_trait!(SystemFlags, u64);
```

### 2. Create a Manager
The `FlagManager` is the brain of your system. Use the builder pattern to set it up.

```rust
let manager = FlagManager::new()
    .with_file_path("flags.bin")
    .add_extension_flag(".rs", "AUTO_SAVE", FlagType::Binary, 0b0001);
```

### 3. Set and Check Flags
The API is designed to be intuitive and fast.

```rust
manager.add_flag("DEBUG_MODE", FlagType::Binary, 0b0001, Some("Enables verbose logging"), None)?;

// Activate the flag
manager.set_flag("DEBUG_MODE")?;

// Check the state
if manager.check_flag("DEBUG_MODE")? {
    println!("Debug mode is active!");
}
```

---

## 🛠️ Advanced Features

### Functional Triggers
Execute logic automatically based on flag states without writing `if/else` blocks everywhere.



```rust
let trigger = FlagTrigger::new(my_flags);

trigger.trigger_if(0b0001, || {
    // This closure only runs if the 1st bit is set
    initiate_shutdown();
});
```

### Complex Pattern Evaluation
Parse and evaluate complex boolean logic strings at runtime.

```rust
let is_allowed = parser.check_complex_pattern("!Network.Flag1 && Core.Flag0");
```

---

## 🗂️ Project Structure

| Module | Purpose |
| :--- | :--- |
| `flag` | Defines the core `Flag` trait for bitwise behavior. |
| `flag_manager` | The thread-safe coordinator for all flags. |
| `macros` | Code generation tools to eliminate boilerplate. |
| `flag_trigger` | Event-driven execution based on flag states. |
| `error` | Exhaustive error handling via `thiserror`. |



---

## 💾 Persistence Strategies

Flagger supports two modes of saving:
* **Total Recall:** Saves the entire manager state, including name mappings and extension logic.
* **Data-Only:** Saves only the raw bitmasks (perfect for minimal database storage).

---

## ⚖️ License
This project is licensed under the MIT License - see the `LICENSE` file for details.

---

## Crates

Since 0.3.0 flaga is a small workspace. `flaga` is the facade: it re-exports every module under its 0.2.0 path, so `flaga::prelude::*`, `flaga::error::FlagError`, `flaga::flag_manager::FlagManager` and the `flaga::impl_flag_trait!` macros keep working.

| Crate | Holds |
| :--- | :--- |
| `flaga` | The facade. Re-exports the three crates below and the `prelude`. |
| `flaga_core` | `flag`, `flag_type`, `flag_types`, `flag_descriptor`, `flag_group`, `module_flag`, `enum_flag_counter`, `flag_parser`, `state`, `macros`, `macros_prelude`, `error`. `no_std` + `alloc`; `std` feature on by default. |
| `flaga_events` | `flag_event`, `flag_event_manager`, `flag_trigger`, `features`. |
| `flaga_store` | `flag_store`, `flag_manager`, `flag_manager_builder`. std only. |

The `flaga_solver` crate is not wired in yet; a `solver` feature on `flaga` comes in 0.3.1.
