// examples/hypergraph_scope.rs
use std::{pin::Pin, time::Duration};

use flaga::{Feature, FeatureManager, FlagParser};
use scope::{scoped, Cx, ScopedTask, TaskPoll};

#[derive(Default)]
struct Resources {
    polls: u64,
    executed: u64,
    checksum: u64,
    fm: FeatureManager,
    fp: FlagParser,
    names: Vec<&'static str>,
}

struct NoopFeature(u64);

impl Feature for NoopFeature {
    fn execute(&self) -> Result<(), String> {
        // Touch the field so the example does not warn about dead code.
        let _ = self.0;
        Ok(())
    }
}

struct QueryTask {
    iters_left: u64,
    iters_per_poll: u32,
    state: u64,
}

impl<'env> ScopedTask<'env, Resources> for QueryTask {
    fn poll(mut self: Pin<&mut Self>, cx: &mut Cx<'_, 'env, Resources>) -> TaskPoll {
        cx.resources.polls += 1;

        if self.iters_left == 0 {
            return TaskPoll::Ready;
        }

        let chunk = self.iters_per_poll.min(self.iters_left as u32);

        for _ in 0..chunk {
            self.iters_left -= 1;

            // Tiny xorshift-style PRNG step.
            self.state ^= self.state << 13;
            self.state ^= self.state >> 7;
            self.state ^= self.state << 17;

            // Pick a registered feature deterministically.
            let idx = (self.state as usize) % cx.resources.names.len();
            let name = cx.resources.names[idx];

            // Gate via parser.
            let enabled = cx
                .resources
                .fp
                .check_complex_pattern("Core.Flag1 && !Net.Flag3");

            if enabled && cx.resources.fm.execute_feature(name).is_ok() {
                cx.resources.executed += 1;
                cx.resources.checksum ^=
                    self.state ^ (idx as u64).wrapping_mul(0x9e37);
            }
        }

        cx.wake_self();
        TaskPoll::Pending
    }
}

fn main() {
    let mut res = Resources::default();

    // Register a lot of features and keep the leaked names for real lookup.
    for i in 0..50_000 {
        let nm: &'static str = Box::leak(format!("F{i}").into_boxed_str());
        res.fm.register_feature(nm, NoopFeature(i as u64));
        res.names.push(nm);
    }

    // Initial parser config:
    // Core.Flag1 = true, Net.Flag3 = true
    // so "Core.Flag1 && !Net.Flag3" starts false.
    res.fp.parse_flag_string("Core:0010 Net:1000");

    scoped(&mut res, |scope| {
        let workers = 32;

        for w in 0..workers {
            scope.spawn_named(
                Box::leak(format!("worker_{w}").into_boxed_str()),
                QueryTask {
                    iters_left: 200_000,
                    iters_per_poll: 256,
                    state: 0xDEAD ^ (w as u64),
                },
            );
        }

        // Flip Net.Flag3 off after a short delay so the gate becomes true.
        let t = scope.sleep("flip_timer", Duration::from_millis(10));
        scope.spawn_map_join("flip", t, |_r, cx| {
            cx.resources.fp.parse_flag_string("Core:0010 Net:0000");
            Ok::<(), ()>(())
        });

        scope.run();
    });

    println!(
        "polls={} executed={} checksum=0x{:016x}",
        res.polls, res.executed, res.checksum
    );
}