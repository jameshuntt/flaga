//! `flaga::solver` is `flaga_solver`, reachable through the `solver` feature.
#![cfg(feature = "solver")]

use big_bad_bool::bitpack;
use flaga::solver::{authorize, check, schema_of, Policy};

#[bitpack(storage = u16)]
pub struct Job16 {
    #[bits(3)]
    #[labels(signal = 0, timer = 1)]
    #[requires(timer => profile.inline_queue)]
    reason: u16,
    #[bits(2)]
    #[labels(pooled = 0, inline_queue = 1)]
    profile: u16,
    #[bits(2)]
    #[flags(hot = 0, cold = 1)]
    flags: u16,
}

#[test]
fn check_and_authorize_through_the_facade() {
    let schema = schema_of!(Job16);
    let v = check(&schema, Job16::new(1, 0, 0).raw() as u128);
    assert_eq!(v.to_string(), "`reason`=timer requires `profile`=inline_queue");
    let (hot, cold) = (1u128 << 5, 1u128 << 6);
    let v = authorize(&schema, cold, &Policy::new(hot | cold, cold));
    assert_eq!(v.to_string(), "flag `flags`.cold denied by policy");
}
