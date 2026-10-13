use std::{pin::Pin, time::Duration, sync::Arc};
use scope::{scoped, Cx, Scope, ScopedTask, TaskPoll};
use flaga::{Feature, FeatureManager, FlagParser};





// ... imports same as before ...

struct HypernodeTask {
    node_id: usize,
    // PRE-CALCULATE these to avoid allocations in poll()
    gate_query: String, 
    next_zone_update: String,
    feature_name: &'static str,
}

// impl<'env> ScopedTask<'env, Resources> for HypernodeTask {
//     fn poll(self: Pin<&mut Self>, cx: &mut Cx<'_, 'env, Resources>) -> TaskPoll {
//         cx.resources.polls += 1;

//         // 1. FAST GATING (No allocations here!)
//         if !cx.resources.fp.check_complex_pattern(&self.gate_query) {
//             // OPTION: Remove cx.wake_self() to let OTHER tasks (like chaos_worker) 
//             // run more frequently. The chaos_worker or other tasks will 
//             // eventually trigger a global wake or the timer will fire.
//             cx.wake_self(); 
//             return TaskPoll::Pending;
//         }

//         // 2. EXECUTION
//         let _ = cx.resources.fm.execute_feature(self.feature_name);
//         cx.resources.executed += 1;
        
//         // 3. MUTATION
//         if self.node_id % 10 == 0 {
//             cx.resources.fp.parse_flag_string(&self.next_zone_update);
//         }

//         TaskPoll::Ready
//     }
// }
impl<'env> ScopedTask<'env, Resources> for HypernodeTask {
    fn poll(self: Pin<&mut Self>, cx: &mut Cx<'_, 'env, Resources>) -> TaskPoll {
        cx.resources.polls += 1;

        // Check the gate
        if !cx.resources.fp.check_complex_pattern(&self.gate_query) {
            cx.wake_self(); 
            return TaskPoll::Pending;
        }
// // 2. In HypernodeTask::poll
// if cx.resources.executed >= cx.resources.max_executions {
//     return TaskPoll::Ready; // Finally exit
// }
        // Simulating a "Work Cycle"
        let _ = cx.resources.fm.execute_feature(self.feature_name);
        cx.resources.executed += 1;
        
        // Instead of returning Ready, we trigger a neighbor and go back to sleep
        if self.node_id % 5 == 0 {
            cx.resources.fp.parse_flag_string(&self.next_zone_update);
        }

        // CRITICAL: We don't return Ready. We yield and wait to be woken again.
        // This keeps the task alive in the hypergraph mesh.
        cx.wake_self(); 
        TaskPoll::Pending 
    }
}
fn main() {
    let mut res = Resources { graph_size: 10_000, ..Default::default() };

    for i in 0..res.graph_size {
        let nm: &'static str = Box::leak(format!("F{i}").into_boxed_str());
        res.fm.register_feature(nm, HyperEdgeFeature { id: i as u64 });
    }

    res.fp.parse_flag_string("Zone0:0001 Global:0001");

    scoped(&mut res, |scope| {
        for i in 0..1000 {
            let zone = i / 10;
            // Pre-calculate strings
            let gate_query = format!("Zone{}.Flag0 && Global.Flag0", zone);
            let next_zone_update = format!("Zone{}:0001", (i / 10) + 1);
            let feature_name: &'static str = Box::leak(format!("F{i}").into_boxed_str());

            scope.spawn_named(
                Box::leak(format!("node_{i}").into_boxed_str()),
                HypernodeTask {
                    node_id: i,
                    gate_query,
                    next_zone_update,
                    feature_name,
                },
            );
        }

        // Increase timer frequency so it "interleaves" better
        let ticker = scope.sleep("chaos", Duration::from_micros(500)); 
        scope.spawn_map_join("chaos_worker", ticker, |_r, cx| {
            // Toggle a flag to unblock different parts of the hypergraph
            let state = if cx.resources.polls % 2 == 0 { "0000" } else { "0001" };
            cx.resources.fp.parse_flag_string(&format!("Global:{}", state));
            Ok::<(), ()>(())
        });
// 3. Update the Chaos Worker to act as a "Monitor"
// scope.spawn_map_join("monitor", ticker, |_r, cx| {
//     // Print progress every time the chaos timer fires
//     println!("Progress: {}/{}", cx.resources.executed, cx.resources.max_executions);
    
//     // Toggle the flags as usual
//     let state = if cx.resources.polls % 2 == 0 { "0000" } else { "0001" };
//     cx.resources.fp.parse_flag_string(&format!("Global:{}", state));
    
//     Ok::<(), ()>(())
// });
        scope.run();
    });

    println!("Completed: polls={} executed={}", res.polls, res.executed);
}


// // 1. Add a limit to Resources
// struct Resources {
//     polls: u64,
//     executed: u64,
//     fm: FeatureManager,
//     fp: FlagParser,
//     max_executions: u64, // NEW: Exit condition
// }







#[derive(Default)]
struct Resources {
    polls: u64,
    executed: u64,
    // Using Arc/Mutex if you want to scale to multithreaded scope executors, 
    // but keeping it simple for local scope access here.
    fm: FeatureManager,
    fp: FlagParser,
    graph_size: usize,
}

struct HyperEdgeFeature { id: u64 }
impl Feature for HyperEdgeFeature {
    fn execute(&self) -> Result<(), String> {
        // Simulate work on a hypergraph edge
        let _ = self.id.wrapping_mul(0x517c);
        Ok(())
    }
}

// struct HypernodeTask {
//     node_id: usize,
//     dependencies: Vec<String>, // Flags that must be true to "fire"
//     state: u64,
// }

// impl<'env> ScopedTask<'env, Resources> for HypernodeTask {
//     fn poll(mut self: Pin<&mut Self>, cx: &mut Cx<'_, 'env, Resources>) -> TaskPoll {
//         cx.resources.polls += 1;

//         // 1. COMPLEX GATING
//         // We check if all hyper-dependencies are met using flaga's logic
//         let mut gate_query = String::new();
//         for (i, dep) in self.dependencies.iter().enumerate() {
//             if i > 0 { gate_query.push_str(" && "); }
//             gate_query.push_str(dep);
//         }

//         // Only execute if the hyper-edge is "active"
//         if !cx.resources.fp.check_complex_pattern(&gate_query) {
//             cx.wake_self(); // Wait for flag flip
//             return TaskPoll::Pending;
//         }

//         // 2. EXECUTION
//         let feature_name = format!("F{}", self.node_id);
//         let _ = cx.resources.fm.execute_feature(Box::leak(feature_name.into_boxed_str()));
        
//         cx.resources.executed += 1;
        
//         // 3. MUTATION (The "Hypergraph" effect)
//         // Once this node fires, it might flip a flag that triggers another cluster
//         if self.node_id % 10 == 0 {
//             let next_zone = format!("Zone{}:0001", self.node_id / 10);
//             cx.resources.fp.parse_flag_string(&next_zone);
//         }

//         TaskPoll::Ready
//     }
// }

// fn main() {
//     let mut res = Resources {
//         graph_size: 10_000,
//         ..Default::default()
//     };

//     // Initialize 10k features (nodes/edges)
//     for i in 0..res.graph_size {
//         let nm: &'static str = Box::leak(format!("F{i}").into_boxed_str());
//         res.fm.register_feature(nm, HyperEdgeFeature { id: i as u64 });
//     }

//     // Initial state: Only Zone0 is active
//     res.fp.parse_flag_string("Zone0:0001 Global:0001");

//     scoped(&mut res, |scope| {
//         // Spawn 1000 tasks with overlapping dependencies
//         for i in 0..1000 {
//             let zone = i / 10;
//             let deps = vec![
//                 format!("Zone{}.Flag0", zone),
//                 "Global.Flag0".to_string()
//             ];

//             scope.spawn_named(
//                 Box::leak(format!("node_{i}").into_boxed_str()),
//                 HypernodeTask {
//                     node_id: i,
//                     dependencies: deps,
//                     state: i as u64,
//                 },
//             );
//         }

//         // CHAOS INJECTOR: Randomly disable/enable global flags
//         let ticker = scope.sleep("chaos", Duration::from_millis(5));
//         scope.spawn_map_join("chaos_worker", ticker, |_r, cx| {
//             if cx.resources.polls % 2 == 0 {
//                 cx.resources.fp.parse_flag_string("Global:0000");
//             } else {
//                 cx.resources.fp.parse_flag_string("Global:0001");
//             }
//             Ok::<(), ()>(())
//         });

//         scope.run();
//     });

//     println!("Completed: polls={} executed={}", res.polls, res.executed);
// }