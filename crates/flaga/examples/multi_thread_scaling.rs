use std::{
    fmt,
    hint::black_box,
    pin::Pin,
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};

use flaga::{Feature, FeatureManager, FlagParser};
use scope::{scoped, Cx, Scope, ScopedTask, TaskPoll};

#[derive(Debug)]
pub enum MultiScopeError {
    InvalidThreadCount,
    ThreadPanicked { shard: usize },
}

impl fmt::Display for MultiScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidThreadCount => write!(f, "thread count must be greater than zero"),
            Self::ThreadPanicked { shard } => write!(f, "worker thread {shard} panicked"),
        }
    }
}

impl std::error::Error for MultiScopeError {}

/// Run many independent `scope::scoped(...)` instances, one per thread,
/// and reduce each thread's local state to a sendable summary.
///
/// Timing uses a barrier so the reported wall time focuses on the actual
/// parallel run phase rather than per-thread initialization/setup.
pub fn scoped_on_threads_reduce<Global, Local, InitLocal, BuildScope, Reduce, Summary>(
    threads: usize,
    global: Arc<Global>,
    init_local: InitLocal,
    build_scope: BuildScope,
    reduce: Reduce,
) -> Result<(Vec<Summary>, Duration), MultiScopeError>
where
    Global: Send + Sync + 'static,
    Local: 'static,
    Summary: Send + 'static,
    InitLocal: Fn(usize, &Arc<Global>) -> Local + Send + Sync + 'static,
    BuildScope:
        for<'env> Fn(usize, &Arc<Global>, &mut Scope<'env, Local>) + Send + Sync + 'static,
    Reduce: Fn(usize, Local, Duration) -> Summary + Send + Sync + 'static,
{
    if threads == 0 {
        return Err(MultiScopeError::InvalidThreadCount);
    }

    let init_local = Arc::new(init_local);
    let build_scope = Arc::new(build_scope);
    let reduce = Arc::new(reduce);
    let barrier = Arc::new(Barrier::new(threads + 1));

    let mut joins = Vec::with_capacity(threads);

    for shard in 0..threads {
        let global = Arc::clone(&global);
        let init_local = Arc::clone(&init_local);
        let build_scope = Arc::clone(&build_scope);
        let reduce = Arc::clone(&reduce);
        let barrier = Arc::clone(&barrier);

        joins.push(thread::spawn(move || {
            let mut local = init_local(shard, &global);

            // Wait until all shards are initialized.
            barrier.wait();

            let run_started = Instant::now();

            scoped(&mut local, |scope| {
                build_scope(shard, &global, scope);
                scope.run();
            });

            let run_elapsed = run_started.elapsed();
            reduce(shard, local, run_elapsed)
        }));
    }

    // Release all shards into the timed run phase at once.
    let wall_started = Instant::now();
    barrier.wait();

    let mut out = Vec::with_capacity(threads);

    for (shard, join) in joins.into_iter().enumerate() {
        match join.join() {
            Ok(summary) => out.push(summary),
            Err(_) => return Err(MultiScopeError::ThreadPanicked { shard }),
        }
    }

    let wall_elapsed = wall_started.elapsed();
    Ok((out, wall_elapsed))
}

#[derive(Default)]
struct LocalResources {
    polls: u64,
    executed: u64,
    checksum: u64,
    fm: FeatureManager,
    fp: FlagParser,
    names: Vec<&'static str>,
}

#[derive(Clone)]
struct GlobalConfig {
    feature_count_per_shard: usize,
    total_workers: usize,
    iters_per_worker: u64,
    iters_per_poll: u32,
    gate_expr: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct ShardSummary {
    polls: u64,
    executed: u64,
    checksum: u64,
    run_elapsed: Duration,
}

#[derive(Debug, Clone, Copy)]
struct LayoutResult {
    threads: usize,
    workers_per_thread: usize,
    polls: u64,
    executed: u64,
    checksum: u64,
    wall_elapsed: Duration,
    max_shard_elapsed: Duration,
    avg_shard_elapsed: Duration,
}

struct NoopFeature(u64);

impl Feature for NoopFeature {
    fn execute(&self) -> Result<(), String> {
        black_box(self.0);
        Ok(())
    }
}

struct QueryTask {
    iters_left: u64,
    iters_per_poll: u32,
    state: u64,
    gate_expr: &'static str,
}

impl<'env> ScopedTask<'env, LocalResources> for QueryTask {
    fn poll(mut self: Pin<&mut Self>, cx: &mut Cx<'_, 'env, LocalResources>) -> TaskPoll {
        cx.resources.polls += 1;

        if self.iters_left == 0 {
            return TaskPoll::Ready;
        }

        let chunk = self.iters_per_poll.min(self.iters_left as u32);

        for _ in 0..chunk {
            self.iters_left -= 1;

            self.state ^= self.state << 13;
            self.state ^= self.state >> 7;
            self.state ^= self.state << 17;

            let idx = (self.state as usize) % cx.resources.names.len();
            let name = cx.resources.names[idx];

            let enabled = cx.resources.fp.check_complex_pattern(self.gate_expr);

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

fn run_layout(
    threads: usize,
    workers_per_thread: usize,
    global: Arc<GlobalConfig>,
) -> Result<LayoutResult, MultiScopeError> {
    assert_eq!(
        threads * workers_per_thread,
        global.total_workers,
        "layout must preserve total worker count"
    );

    let (summaries, wall_elapsed) = scoped_on_threads_reduce(
        threads,
        Arc::clone(&global),
        |shard, global| {
            let mut local = LocalResources::default();

            for i in 0..global.feature_count_per_shard {
                let nm: &'static str =
                    Box::leak(format!("F{shard}_{i}").into_boxed_str());
                local.fm.register_feature(nm, NoopFeature(i as u64));
                local.names.push(nm);
            }

            // Deterministic always-enabled gate:
            // Core.Flag1 = true, Net.Flag3 = false
            local.fp.parse_flag_string("Core:0010 Net:0000");
            local
        },
        move |shard, global, scope| {
            for w in 0..workers_per_thread {
                // Stable worker identity across layouts.
                let global_worker_id = shard * workers_per_thread + w;

                scope.spawn_named(
                    Box::leak(format!("worker_{global_worker_id}").into_boxed_str()),
                    QueryTask {
                        iters_left: global.iters_per_worker,
                        iters_per_poll: global.iters_per_poll,
                        state: 0xDEAD_BEEF ^ (global_worker_id as u64),
                        gate_expr: global.gate_expr,
                    },
                );
            }
        },
        |_shard, local, run_elapsed| ShardSummary {
            polls: local.polls,
            executed: local.executed,
            checksum: local.checksum,
            run_elapsed,
        },
    )?;

    let mut polls = 0u64;
    let mut executed = 0u64;
    let mut checksum = 0u64;
    let mut max_shard_elapsed = Duration::ZERO;
    let mut total_shard_nanos: u128 = 0;

    for s in summaries {
        polls += s.polls;
        executed += s.executed;
        checksum ^= s.checksum;
        max_shard_elapsed = max_shard_elapsed.max(s.run_elapsed);
        total_shard_nanos += s.run_elapsed.as_nanos();
    }

    let avg_shard_elapsed =
        Duration::from_nanos((total_shard_nanos / threads as u128) as u64);

    let expected_executed = (global.total_workers as u64) * global.iters_per_worker;
    assert_eq!(
        executed,
        expected_executed,
        "deterministic always-enabled benchmark should execute every iteration"
    );

    Ok(LayoutResult {
        threads,
        workers_per_thread,
        polls,
        executed,
        checksum,
        wall_elapsed,
        max_shard_elapsed,
        avg_shard_elapsed,
    })
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let global = Arc::new(GlobalConfig {
        feature_count_per_shard: 50_000,
        total_workers: 32,
        iters_per_worker: 200_000,
        iters_per_poll: 256,
        gate_expr: "Core.Flag1 && !Net.Flag3",
    });

    let layouts = [(1, 32), (2, 16), (4, 8), (8, 4)];
    let mut results = Vec::with_capacity(layouts.len());

    for (threads, workers_per_thread) in layouts {
        results.push(run_layout(threads, workers_per_thread, Arc::clone(&global))?);
    }

    let baseline_ms = ms(results[0].wall_elapsed);

    println!(
        "{:<8} {:<8} {:<10} {:<12} {:<18} {:<12} {:<14} {:<14} {:<10} {:<10}",
        "threads",
        "workers",
        "polls",
        "executed",
        "checksum",
        "wall_ms",
        "max_shard_ms",
        "avg_shard_ms",
        "speedup",
        "eff_%",
    );

    for r in results {
        let wall_ms = ms(r.wall_elapsed);
        let speedup = baseline_ms / wall_ms;
        let efficiency = (speedup / r.threads as f64) * 100.0;

        println!(
            "{:<8} {:<8} {:<10} {:<12} 0x{:<016x} {:<12.3} {:<14.3} {:<14.3} {:<10.3} {:<10.2}",
            r.threads,
            r.workers_per_thread,
            r.polls,
            r.executed,
            r.checksum,
            wall_ms,
            ms(r.max_shard_elapsed),
            ms(r.avg_shard_elapsed),
            speedup,
            efficiency,
        );
    }

    Ok(())
}



























// use std::{
//     fmt,
//     hint::black_box,
//     pin::Pin,
//     sync::Arc,
//     thread,
//     time::{Duration, Instant},
// };

// use flaga::{Feature, FeatureManager, FlagParser};
// use scope::{scoped, Cx, Scope, ScopedTask, TaskPoll};

// #[derive(Debug)]
// pub enum MultiScopeError {
//     InvalidThreadCount,
//     ThreadPanicked { shard: usize },
// }

// impl fmt::Display for MultiScopeError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::InvalidThreadCount => write!(f, "thread count must be greater than zero"),
//             Self::ThreadPanicked { shard } => {
//                 write!(f, "worker thread {shard} panicked")
//             }
//         }
//     }
// }

// impl std::error::Error for MultiScopeError {}

// /// Run many independent `scope::scoped(...)` instances, one per thread,
// /// and reduce each thread's local state to a sendable summary.
// ///
// /// `Local` never crosses the thread boundary.
// /// Only `Summary` does.
// pub fn scoped_on_threads_reduce<Global, Local, InitLocal, BuildScope, Reduce, Summary>(
//     threads: usize,
//     global: Arc<Global>,
//     init_local: InitLocal,
//     build_scope: BuildScope,
//     reduce: Reduce,
// ) -> Result<Vec<Summary>, MultiScopeError>
// where
//     Global: Send + Sync + 'static,
//     Local: 'static,
//     Summary: Send + 'static,
//     InitLocal: Fn(usize, &Arc<Global>) -> Local + Send + Sync + 'static,
//     BuildScope:
//         for<'env> Fn(usize, &Arc<Global>, &mut Scope<'env, Local>) + Send + Sync + 'static,
//     Reduce: Fn(usize, Local, Duration) -> Summary + Send + Sync + 'static,
// {
//     if threads == 0 {
//         return Err(MultiScopeError::InvalidThreadCount);
//     }

//     let init_local = Arc::new(init_local);
//     let build_scope = Arc::new(build_scope);
//     let reduce = Arc::new(reduce);

//     let mut joins = Vec::with_capacity(threads);

//     for shard in 0..threads {
//         let global = Arc::clone(&global);
//         let init_local = Arc::clone(&init_local);
//         let build_scope = Arc::clone(&build_scope);
//         let reduce = Arc::clone(&reduce);

//         joins.push(thread::spawn(move || {
//             let mut local = init_local(shard, &global);

//             let run_started = Instant::now();

//             scoped(&mut local, |scope| {
//                 build_scope(shard, &global, scope);
//                 scope.run();
//             });

//             let run_elapsed = run_started.elapsed();
//             reduce(shard, local, run_elapsed)
//         }));
//     }

//     let mut out = Vec::with_capacity(threads);

//     for (shard, join) in joins.into_iter().enumerate() {
//         match join.join() {
//             Ok(summary) => out.push(summary),
//             Err(_) => return Err(MultiScopeError::ThreadPanicked { shard }),
//         }
//     }

//     Ok(out)
// }

// #[derive(Default)]
// struct LocalResources {
//     polls: u64,
//     executed: u64,
//     checksum: u64,
//     fm: FeatureManager,
//     fp: FlagParser,
//     names: Vec<&'static str>,
// }

// #[derive(Clone)]
// struct GlobalConfig {
//     feature_count_per_shard: usize,
//     total_workers: usize,
//     iters_per_worker: u64,
//     iters_per_poll: u32,
//     gate_expr: &'static str,
// }

// #[derive(Debug, Clone, Copy)]
// struct ShardSummary {
//     polls: u64,
//     executed: u64,
//     checksum: u64,
//     run_elapsed: Duration,
// }

// #[derive(Debug, Clone, Copy)]
// struct LayoutResult {
//     threads: usize,
//     workers_per_thread: usize,
//     polls: u64,
//     executed: u64,
//     checksum: u64,
//     wall_elapsed: Duration,
//     max_shard_elapsed: Duration,
//     avg_shard_elapsed: Duration,
// }

// struct NoopFeature(u64);

// impl Feature for NoopFeature {
//     fn execute(&self) -> Result<(), String> {
//         black_box(self.0);
//         Ok(())
//     }
// }

// struct QueryTask {
//     iters_left: u64,
//     iters_per_poll: u32,
//     state: u64,
//     gate_expr: &'static str,
// }

// impl<'env> ScopedTask<'env, LocalResources> for QueryTask {
//     fn poll(mut self: Pin<&mut Self>, cx: &mut Cx<'_, 'env, LocalResources>) -> TaskPoll {
//         cx.resources.polls += 1;

//         if self.iters_left == 0 {
//             return TaskPoll::Ready;
//         }

//         let chunk = self.iters_per_poll.min(self.iters_left as u32);

//         for _ in 0..chunk {
//             self.iters_left -= 1;

//             // xorshift-ish step
//             self.state ^= self.state << 13;
//             self.state ^= self.state >> 7;
//             self.state ^= self.state << 17;

//             let idx = (self.state as usize) % cx.resources.names.len();
//             let name = cx.resources.names[idx];

//             let enabled = cx.resources.fp.check_complex_pattern(self.gate_expr);

//             if enabled && cx.resources.fm.execute_feature(name).is_ok() {
//                 cx.resources.executed += 1;
//                 cx.resources.checksum ^=
//                     self.state ^ (idx as u64).wrapping_mul(0x9e37);
//             }
//         }

//         cx.wake_self();
//         TaskPoll::Pending
//     }
// }

// fn run_layout(
//     threads: usize,
//     workers_per_thread: usize,
//     global: Arc<GlobalConfig>,
// ) -> Result<LayoutResult, MultiScopeError> {
//     assert_eq!(
//         threads * workers_per_thread,
//         global.total_workers,
//         "layout must preserve total worker count"
//     );

//     let wall_started = Instant::now();

//     let summaries = scoped_on_threads_reduce(
//         threads,
//         Arc::clone(&global),
//         |shard, global| {
//             let mut local = LocalResources::default();

//             for i in 0..global.feature_count_per_shard {
//                 let nm: &'static str =
//                     Box::leak(format!("F{shard}_{i}").into_boxed_str());
//                 local.fm.register_feature(nm, NoopFeature(i as u64));
//                 local.names.push(nm);
//             }

//             // Deterministic always-enabled gate:
//             // Core.Flag1 = true, Net.Flag3 = false
//             local.fp.parse_flag_string("Core:0010 Net:0000");
//             local
//         },
//         |shard, global, scope| {
//             for w in 0..global.workers_per_thread.min(workers_per_thread) {
//                 scope.spawn_named(
//                     Box::leak(format!("worker_{shard}_{w}").into_boxed_str()),
//                     QueryTask {
//                         iters_left: global.iters_per_worker,
//                         iters_per_poll: global.iters_per_poll,
//                         state: 0xDEAD_BEEF ^ ((shard as u64) << 32) ^ (w as u64),
//                         gate_expr: global.gate_expr,
//                     },
//                 );
//             }
//         },
//         |_shard, local, run_elapsed| ShardSummary {
//             polls: local.polls,
//             executed: local.executed,
//             checksum: local.checksum,
//             run_elapsed,
//         },
//     )?;

//     let wall_elapsed = wall_started.elapsed();

//     let mut polls = 0u64;
//     let mut executed = 0u64;
//     let mut checksum = 0u64;
//     let mut max_shard_elapsed = Duration::ZERO;
//     let mut total_shard_nanos: u128 = 0;

//     for s in summaries {
//         polls += s.polls;
//         executed += s.executed;
//         checksum ^= s.checksum;
//         max_shard_elapsed = max_shard_elapsed.max(s.run_elapsed);
//         total_shard_nanos += s.run_elapsed.as_nanos();
//     }

//     let avg_shard_elapsed = Duration::from_nanos(
//         (total_shard_nanos / threads as u128) as u64
//     );

//     let expected_executed = (global.total_workers as u64) * global.iters_per_worker;
//     assert_eq!(
//         executed, expected_executed,
//         "deterministic always-enabled benchmark should execute every iteration"
//     );

//     Ok(LayoutResult {
//         threads,
//         workers_per_thread,
//         polls,
//         executed,
//         checksum,
//         wall_elapsed,
//         max_shard_elapsed,
//         avg_shard_elapsed,
//     })
// }

// fn ms(d: Duration) -> f64 {
//     d.as_secs_f64() * 1000.0
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let global = Arc::new(GlobalConfig {
//         feature_count_per_shard: 50_000,
//         total_workers: 32,
//         iters_per_worker: 200_000,
//         iters_per_poll: 256,
//         gate_expr: "Core.Flag1 && !Net.Flag3",
//     });

//     let layouts = [(1, 32), (2, 16), (4, 8), (8, 4)];

//     println!(
//         "{:<8} {:<8} {:<10} {:<12} {:<18} {:<12} {:<14} {:<14}",
//         "threads",
//         "workers",
//         "polls",
//         "executed",
//         "checksum",
//         "wall_ms",
//         "max_shard_ms",
//         "avg_shard_ms",
//     );

//     for (threads, workers_per_thread) in layouts {
//         let r = run_layout(threads, workers_per_thread, Arc::clone(&global))?;

//         println!(
//             "{:<8} {:<8} {:<10} {:<12} 0x{:<016x} {:<12.3} {:<14.3} {:<14.3}",
//             r.threads,
//             r.workers_per_thread,
//             r.polls,
//             r.executed,
//             r.checksum,
//             ms(r.wall_elapsed),
//             ms(r.max_shard_elapsed),
//             ms(r.avg_shard_elapsed),
//         );
//     }

//     Ok(())
// }