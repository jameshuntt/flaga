use std::{fmt, sync::Arc, thread};

use scope::{scoped, Scope};

#[derive(Debug)]
pub enum MultiScopeError {
    InvalidThreadCount,
    ThreadPanicked { shard: usize },
}

impl fmt::Display for MultiScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidThreadCount => write!(f, "thread count must be greater than zero"),
            Self::ThreadPanicked { shard } => {
                write!(f, "worker thread {shard} panicked")
            }
        }
    }
}

impl std::error::Error for MultiScopeError {}

/// Run many independent `scope::scoped(...)` instances, one per thread,
/// and reduce each thread's local state to a sendable summary.
///
/// This version does NOT require `Local: Send`, because `Local` never crosses
/// the thread boundary. Only `Summary` does.
pub fn scoped_on_threads_reduce<Global, Local, InitLocal, BuildScope, Reduce, Summary>(
    threads: usize,
    global: Arc<Global>,
    init_local: InitLocal,
    build_scope: BuildScope,
    reduce: Reduce,
) -> Result<Vec<Summary>, MultiScopeError>
where
    Global: Send + Sync + 'static,
    Local: 'static,
    Summary: Send + 'static,
    InitLocal: Fn(usize, &Arc<Global>) -> Local + Send + Sync + 'static,
    BuildScope:
        for<'env> Fn(usize, &Arc<Global>, &mut Scope<'env, Local>) + Send + Sync + 'static,
    Reduce: Fn(usize, Local) -> Summary + Send + Sync + 'static,
{
    if threads == 0 {
        return Err(MultiScopeError::InvalidThreadCount);
    }

    let init_local = Arc::new(init_local);
    let build_scope = Arc::new(build_scope);
    let reduce = Arc::new(reduce);

    let mut joins = Vec::with_capacity(threads);

    for shard in 0..threads {
        let global = Arc::clone(&global);
        let init_local = Arc::clone(&init_local);
        let build_scope = Arc::clone(&build_scope);
        let reduce = Arc::clone(&reduce);

        joins.push(thread::spawn(move || {
            let mut local = init_local(shard, &global);

            scoped(&mut local, |scope| {
                build_scope(shard, &global, scope);
                scope.run();
            });

            reduce(shard, local)
        }));
    }

    let mut out = Vec::with_capacity(threads);

    for (shard, join) in joins.into_iter().enumerate() {
        match join.join() {
            Ok(summary) => out.push(summary),
            Err(_) => return Err(MultiScopeError::ThreadPanicked { shard }),
        }
    }

    Ok(out)
}

use std::{pin::Pin, time::Duration};

use flaga::{Feature, FeatureManager, FlagParser};
use scope::{Cx, ScopedTask, TaskPoll};

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
    feature_count: usize,
    workers_per_thread: usize,
    iters_per_worker: u64,
    iters_per_poll: u32,
    gate_expr: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct LocalSummary {
    polls: u64,
    executed: u64,
    checksum: u64,
}

struct NoopFeature(u64);

impl Feature for NoopFeature {
    fn execute(&self) -> Result<(), String> {
        let _ = self.0;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
// run summaries...
    let threads = 8;

    let global = Arc::new(GlobalConfig {
        feature_count: 50_000,
        workers_per_thread: 4,
        iters_per_worker: 200_000,
        iters_per_poll: 256,
        gate_expr: "Core.Flag1 && !Net.Flag3",
    });

    let summaries = scoped_on_threads_reduce(
        threads,
        Arc::clone(&global),
        |shard, global| {
            let mut local = LocalResources::default();

            for i in 0..global.feature_count {
                let nm: &'static str =
                    Box::leak(format!("F{shard}_{i}").into_boxed_str());
                local.fm.register_feature(nm, NoopFeature(i as u64));
                local.names.push(nm);
            }

            local.fp.parse_flag_string("Core:0010 Net:1000");
            local
        },
        |shard, global, scope| {
            for w in 0..global.workers_per_thread {
                scope.spawn_named(
                    Box::leak(format!("worker_{shard}_{w}").into_boxed_str()),
                    QueryTask {
                        iters_left: global.iters_per_worker,
                        iters_per_poll: global.iters_per_poll,
                        state: 0xDEAD_BEEF ^ ((shard as u64) << 32) ^ (w as u64),
                        gate_expr: global.gate_expr,
                    },
                );
            }

            let t = scope.sleep(
                Box::leak(format!("flip_timer_{shard}").into_boxed_str()),
                Duration::from_millis(10),
            );

            scope.spawn_map_join(
                Box::leak(format!("flip_{shard}").into_boxed_str()),
                t,
                |_r, cx| {
                    cx.resources.fp.parse_flag_string("Core:0010 Net:0000");
                    Ok::<(), ()>(())
                },
            );
        },
        |_shard, local| LocalSummary {
            polls: local.polls,
            executed: local.executed,
            checksum: local.checksum,
        },
    )?;

    let mut polls = 0u64;
    let mut executed = 0u64;
    let mut checksum = 0u64;

    for s in summaries {
        polls += s.polls;
        executed += s.executed;
        checksum ^= s.checksum;
    }

let elapsed = start.elapsed();
println!(
    "threads={} polls={} executed={} checksum=0x{:016x} elapsed={:?}",
    threads, polls, executed, checksum, elapsed
);
    println!(
        "threads={} polls={} executed={} checksum=0x{:016x}",
        threads, polls, executed, checksum
    );

    Ok(())
}



















// use std::{
//     fmt,
//     thread,
// };

// use scope::{scoped, Scope};

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

// /// Run many independent `scope::scoped(...)` instances, one per thread.
// ///
// /// This is the correct wrapper shape for a single-threaded `Scope` runtime.
// ///
// /// - `Global` is shared across all threads via `Arc`.
// /// - `Local` is thread-local and mutably borrowed by each thread's `Scope`.
// /// - `init_local` constructs each thread's local resource object.
// /// - `build_scope` populates one thread's scope with tasks.
// ///
// /// Returns the final local resource object from every thread.
// pub fn scoped_on_threads<Global, Local, InitLocal, BuildScope>(
//     threads: usize,
//     global: Arc<Global>,
//     init_local: InitLocal,
//     build_scope: BuildScope,
// ) -> Result<Vec<Local>, MultiScopeError>
// where
//     Global: Send + Sync + 'static,
//     Local: Send + 'static,
//     InitLocal: Fn(usize, &Arc<Global>) -> Local + Send + Sync + 'static,
//     BuildScope: for<'env> Fn(usize, &Arc<Global>, &mut Scope<'env, Local>) + Send + Sync + 'static,
// {
//     if threads == 0 {
//         return Err(MultiScopeError::InvalidThreadCount);
//     }

//     let init_local = Arc::new(init_local);
//     let build_scope = Arc::new(build_scope);

//     let mut joins = Vec::with_capacity(threads);

//     for shard in 0..threads {
//         let global = Arc::clone(&global);
//         let init_local = Arc::clone(&init_local);
//         let build_scope = Arc::clone(&build_scope);

//         joins.push(thread::spawn(move || {
//             let mut local = init_local(shard, &global);

//             scoped(&mut local, |scope| {
//                 build_scope(shard, &global, scope);
//                 scope.run();
//             });

//             local
//         }));
//     }

//     let mut out = Vec::with_capacity(threads);

//     for (shard, join) in joins.into_iter().enumerate() {
//         match join.join() {
//             Ok(local) => out.push(local),
//             Err(_panic_payload) => {
//                 return Err(MultiScopeError::ThreadPanicked { shard });
//             }
//         }
//     }

//     Ok(out)
// }


// use std::{pin::Pin, sync::Arc, time::Duration};

// use flaga::{Feature, FeatureManager, FlagParser};
// use scope::{Cx, ScopedTask, TaskPoll};


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
//     feature_count: usize,
//     workers_per_thread: usize,
//     iters_per_worker: u64,
//     iters_per_poll: u32,
//     gate_expr: &'static str,
// }

// struct NoopFeature(u64);

// impl Feature for NoopFeature {
//     fn execute(&self) -> Result<(), String> {
//         let _ = self.0;
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

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let threads = 8;

//     let global = Arc::new(GlobalConfig {
//         feature_count: 50_000,
//         workers_per_thread: 4,
//         iters_per_worker: 200_000,
//         iters_per_poll: 256,
//         gate_expr: "Core.Flag1 && !Net.Flag3",
//     });

//     let locals = scoped_on_threads(
//         threads,
//         Arc::clone(&global),
//         |shard, global| {
//             let mut local = LocalResources::default();

//             for i in 0..global.feature_count {
//                 let nm: &'static str =
//                     Box::leak(format!("F{shard}_{i}").into_boxed_str());
//                 local.fm.register_feature(nm, NoopFeature(i as u64));
//                 local.names.push(nm);
//             }

//             // start disabled; timer will flip it
//             local.fp.parse_flag_string("Core:0010 Net:1000");
//             local
//         },
//         |shard, global, scope| {
//             for w in 0..global.workers_per_thread {
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

//             let t = scope.sleep(
//                 Box::leak(format!("flip_timer_{shard}").into_boxed_str()),
//                 Duration::from_millis(10),
//             );

//             scope.spawn_map_join(
//                 Box::leak(format!("flip_{shard}").into_boxed_str()),
//                 t,
//                 |_r, cx| {
//                     cx.resources.fp.parse_flag_string("Core:0010 Net:0000");
//                     Ok::<(), ()>(())
//                 },
//             );
//         },
//     )?;

//     let mut polls = 0u64;
//     let mut executed = 0u64;
//     let mut checksum = 0u64;

//     for local in locals {
//         polls += local.polls;
//         executed += local.executed;
//         checksum ^= local.checksum;
//     }

//     println!(
//         "threads={} polls={} executed={} checksum=0x{:016x}",
//         threads, polls, executed, checksum
//     );

//     Ok(())
// }