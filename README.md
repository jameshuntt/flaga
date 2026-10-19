# flaga

- [`flaga_core`](./crates/flaga_core) — https://crates.io/crates/flaga_core
- [`flaga_events`](./crates/flaga_events) — https://crates.io/crates/flaga_events
- [`flaga_store`](./crates/flaga_store) — https://crates.io/crates/flaga_store
- [`flaga`](./crates/flaga) — https://crates.io/crates/flaga

## Releasing

Bump `version` in Cargo.toml, add a dated `## <version> (YYYY-MM-DD)` entry to CHANGELOG.md, and push to
`main`. CI (.github/workflows/ci.yml) runs the tests and clippy, then publishes to crates.io each crate whose
version is not there yet, in dependency order.
