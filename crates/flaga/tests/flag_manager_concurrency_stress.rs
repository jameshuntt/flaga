use std::sync::{Arc, Barrier};
use std::thread;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use flaga::prelude::*;

fn mk_manager() -> FlagManager {
    let mgr = FlagManager::new();
    // Register a bunch of flags (monotonic set_flag makes it easy to validate).
    for i in 0..8 {
        mgr.add_flag(
            &format!("B{i}"),
            FlagType::Binary,
            1u64 << i,
            None,
            None,
        ).unwrap();
    }
    for i in 0..32 {
        mgr.add_flag(
            &format!("H{i}"),
            FlagType::Hex,
            1u64 << i,
            None,
            None,
        ).unwrap();
    }
    mgr
}

#[test]
#[ignore]
fn concurrency_stress_32_threads() {
    let mgr = Arc::new(mk_manager());
    let threads = 32usize;
    let iters = 200_000usize;

    let barrier = Arc::new(Barrier::new(threads));
    let mut handles = Vec::new();

    for t in 0..threads {
        let mgr = mgr.clone();
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(0xBADC0FFE_u64 ^ (t as u64));
            barrier.wait();

            for _ in 0..iters {
                if rng.gen_bool(0.5) {
                    let i = rng.gen_range(0..8);
                    let name = format!("B{i}");
                    mgr.set_flag(&name).unwrap();
                    assert!(mgr.check_flag(&name).unwrap());
                } else {
                    let i = rng.gen_range(0..32);
                    let name = format!("H{i}");
                    mgr.set_flag(&name).unwrap();
                    assert!(mgr.check_flag(&name).unwrap());
                }

                // Negative test: unknown flag is always NotFound.
                if rng.gen_bool(0.0005) {
                    let e = mgr.check_flag("DOES_NOT_EXIST").unwrap_err();
                    let msg = format!("{e:?}");
                    assert!(msg.contains("NotFound") || msg.contains("not found"));
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}