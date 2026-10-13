// examples/hypergraph_scope.rs
use std::{pin::Pin, time::Duration};
use scope::{scoped, Cx, Scope, ScopedTask, TaskPoll};
use flaga::{Feature, FeatureManager, FlagManagerBuilder, FlagType, FlagParser};

#[derive(Default)]
struct Resources {
    polls: u64,
    executed: u64,
    checksum: u64,
    fm: FeatureManager,
    fp: FlagParser,
}

struct NoopFeature(u64);
impl Feature for NoopFeature {
    fn execute(&self) -> Result<(), String> { Ok(()) }
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

        // do a chunk of work per poll
        let chunk = self.iters_per_poll.min(self.iters_left as u32);
        for _ in 0..chunk {
            self.iters_left -= 1;

            // tiny prng step
            self.state ^= self.state << 13;
            self.state ^= self.state >> 7;
            self.state ^= self.state << 17;

            // pick a “feature” deterministically
            let idx = (self.state as usize) % 50_000;
            let name = unsafe { std::mem::transmute::<&str, &'static str>("F") }; // replace w/ leaked names in real example

            // gate via parser (or manager)
            // ex pattern: "!Net.Flag3 && Core.Flag1"
            let enabled = cx.resources.fp.check_complex_pattern("Core.Flag1 && !Net.Flag3");

            if enabled {
                let _ = cx.resources.fm.execute_feature(name);
                cx.resources.executed += 1;
                cx.resources.checksum ^= self.state ^ (idx as u64).wrapping_mul(0x9e37);
            }
        }

        cx.wake_self();
        TaskPoll::Pending
    }
}

fn main() {
    let mut res = Resources::default();

    // FeatureManager: register a lot of features
    for i in 0..50_000 {
        // in your real stress test you already leak &'static str names; do that here too
        let nm: &'static str = Box::leak(format!("F{i}").into_boxed_str());
        res.fm.register_feature(nm, NoopFeature(i as u64));
    }

    // FlagParser config
    res.fp.parse_flag_string("Core:0010 Net:1000"); // Core.Flag1 true, Net.Flag3 true

    scoped(&mut res, |scope| {
        // spawn N query workers
        let workers = 32;
        for w in 0..workers {
            scope.spawn_named(
                Box::leak(format!("worker_{w}").into_boxed_str()),
                QueryTask { iters_left: 200_000, iters_per_poll: 256, state: 0xDEAD ^ (w as u64) },
            );
        }

        // optional: periodic toggles to force different gating paths
        let t = scope.sleep("flip_timer", Duration::from_millis(10));
        scope.spawn_map_join("flip", t, |_r, cx| {
            // flip Net.Flag3 by rewriting parser state (or use FlagManager + event manager)
            cx.resources.fp.parse_flag_string("Core:0010 Net:0000");
            Ok::<(), ()>(())
        });

        scope.run();
    });

    println!("polls={} executed={} checksum=0x{:016x}", res.polls, res.executed, res.checksum);
}