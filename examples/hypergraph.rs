use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Instant;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use flaga::prelude::*;

// ---------------------------
// R-tree-ish bulk-loaded index (STR-like)
// ---------------------------

#[derive(Clone, Copy, Debug)]
struct Rect {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl Rect {
    #[inline]
    fn union(a: Rect, b: Rect) -> Rect {
        Rect {
            min_x: a.min_x.min(b.min_x),
            min_y: a.min_y.min(b.min_y),
            max_x: a.max_x.max(b.max_x),
            max_y: a.max_y.max(b.max_y),
        }
    }

    #[inline]
    fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    #[inline]
    fn center(&self) -> (f32, f32) {
        ((self.min_x + self.max_x) * 0.5, (self.min_y + self.max_y) * 0.5)
    }
}

enum RNode {
    Leaf { bbox: Rect, items: Vec<usize> },
    Branch { bbox: Rect, kids: Vec<usize> },
}

struct RTree {
    nodes: Vec<RNode>,
    root: usize,
}

impl RTree {
    fn build_str(rects: &[Rect], fanout: usize) -> Self {
        assert!(fanout >= 4);

        // 1) order items by center x then center y (STR-ish)
        let mut idxs: Vec<usize> = (0..rects.len()).collect();
        idxs.sort_by(|&a, &b| {
            let (ax, ay) = rects[a].center();
            let (bx, by) = rects[b].center();
            ax.partial_cmp(&bx).unwrap().then(ay.partial_cmp(&by).unwrap())
        });

        let mut nodes: Vec<RNode> = Vec::new();
        let mut level: Vec<usize> = Vec::new();

        // 2) make leaves
        for chunk in idxs.chunks(fanout) {
            let mut bbox = rects[chunk[0]];
            for &i in &chunk[1..] {
                bbox = Rect::union(bbox, rects[i]);
            }
            let node_idx = nodes.len();
            nodes.push(RNode::Leaf { bbox, items: chunk.to_vec() });
            level.push(node_idx);
        }

        // 3) build up
        while level.len() > 1 {
            let mut next = Vec::new();
            for chunk in level.chunks(fanout) {
                let bbox = chunk.iter().fold(match &nodes[chunk[0]] {
                    RNode::Leaf { bbox, .. } => *bbox,
                    RNode::Branch { bbox, .. } => *bbox,
                }, |acc, &n| {
                    let b = match &nodes[n] {
                        RNode::Leaf { bbox, .. } => *bbox,
                        RNode::Branch { bbox, .. } => *bbox,
                    };
                    Rect::union(acc, b)
                });

                let node_idx = nodes.len();
                nodes.push(RNode::Branch { bbox, kids: chunk.to_vec() });
                next.push(node_idx);
            }
            level = next;
        }

        let root = level[0];
        Self { nodes, root }
    }

    fn query_point(&self, x: f32, y: f32, out: &mut Vec<usize>) {
        out.clear();
        let mut stack = vec![self.root];

        while let Some(n) = stack.pop() {
            match &self.nodes[n] {
                RNode::Leaf { bbox, items } => {
                    if bbox.contains_point(x, y) {
                        out.extend(items.iter().copied());
                    }
                }
                RNode::Branch { bbox, kids } => {
                    if bbox.contains_point(x, y) {
                        for &k in kids {
                            stack.push(k);
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------
// Hypergraph-gated Features
// ---------------------------

struct WorkFeature {
    id: u32,
    checksum: Arc<AtomicU64>,
}

impl Feature for WorkFeature {
    fn execute(&self) -> Result<(), String> {
        // cheap but non-trivial mixing to prevent “optimized away”
        let mut x = self.checksum.load(Ordering::Relaxed);
        x ^= (self.id as u64).wrapping_mul(0x9E3779B97F4A7C15);
        x = x.rotate_left((self.id % 63) + 1);
        x = x.wrapping_mul(0xC2B2AE3D27D4EB4F);
        self.checksum.store(x, Ordering::Relaxed);
        Ok(())
    }
}

fn main() {
    // Tune knobs
    let n_nodes: usize = 50_000;
    let fanout: usize = 32;
    let queries: usize = 200_000;
    let max_deps: usize = 3;

    let mut rng = StdRng::seed_from_u64(0xDEADBEEF);

    // Build rectangles
    let mut rects = Vec::with_capacity(n_nodes);
    for _ in 0..n_nodes {
        let x = rng.r#gen::<f32>();
        let y = rng.r#gen::<f32>();
        let w = rng.gen_range(0.0005..0.01);
        let h = rng.gen_range(0.0005..0.01);
        rects.push(Rect {
            min_x: (x - w).max(0.0),
            min_y: (y - h).max(0.0),
            max_x: (x + w).min(1.0),
            max_y: (y + h).min(1.0),
        });
    }

    // Build R-tree-ish spatial index
    let tree = RTree::build_str(&rects, fanout);

    // Build a hypergraph: each node depends on a few earlier nodes (DAG)
    let mut deps: Vec<Vec<usize>> = Vec::with_capacity(n_nodes);
    for i in 0..n_nodes {
        let k = rng.gen_range(0..=max_deps);
        let mut v = Vec::with_capacity(k);
        for _ in 0..k {
            if i == 0 { break; }
            v.push(rng.gen_range(0..i));
        }
        deps.push(v);
    }

    // Each node has a required *mask* (HexFlag semantics: overlap => allowed)
    let mut requires: Vec<u64> = Vec::with_capacity(n_nodes);
    for _ in 0..n_nodes {
        // 1-3 random bits required (overlap check)
        let bits = rng.gen_range(1..=3);
        let mut m = 0u64;
        for _ in 0..bits {
            m |= 1u64 << rng.gen_range(0..64);
        }
        requires.push(m);
    }

    // Register features (dynamic dispatch) keyed by &'static str
    let checksum = Arc::new(AtomicU64::new(0));
    let mut fm = FeatureManager::new();
    let mut names: Vec<&'static str> = Vec::with_capacity(n_nodes);
    for i in 0..n_nodes {
        let s = format!("N{i}");
        let name: &'static str = Box::leak(s.into_boxed_str());
        names.push(name);
        fm.register_feature(name, WorkFeature { id: i as u32, checksum: checksum.clone() });
    }

    // Runtime state flags
    let mut state = HexFlag::new(rng.r#gen::<u64>());

    // Per-query “done” stamps (avoid clearing 50k vec every query)
    let mut done_epoch: Vec<u32> = vec![0; n_nodes];
    let mut epoch: u32 = 1;

    let mut candidates: Vec<usize> = Vec::new();
    let mut executed: u64 = 0;

    let start = Instant::now();

    for q in 0..queries {
        // Occasionally mutate runtime flags (simulates live toggles / rollouts)
        if (q & 0xFF) == 0 {
            state.set_flags(rng.r#gen::<u64>());
            epoch = epoch.wrapping_add(1).max(1);
        }

        let px = rng.r#gen::<f32>();
        let py = rng.r#gen::<f32>();
        tree.query_point(px, py, &mut candidates);

        // Execute hypergraph closure of each candidate
        for &root in &candidates {
            // fast skip if already done this epoch
            if done_epoch[root] == epoch {
                continue;
            }

            // iterative DFS with post-order execution
            let mut stack: Vec<(usize, usize)> = Vec::new();
            stack.push((root, 0));

            while let Some((n, di)) = stack.pop() {
                if done_epoch[n] == epoch {
                    continue;
                }

                // gating: if the node's requirement does not overlap current state, skip it
                if !state.check_flag(requires[n]) {
                    continue;
                }

                let d = &deps[n];

                if di < d.len() {
                    // re-push current with next dep index
                    stack.push((n, di + 1));

                    let child = d[di];
                    if done_epoch[child] != epoch {
                        stack.push((child, 0));
                    }
                    continue;
                }

                // all deps visited -> execute node
                fm.execute_feature(names[n]).unwrap();
                done_epoch[n] = epoch;
                executed += 1;
            }
        }
    }

    let dt = start.elapsed();
    let checksum_val = checksum.load(Ordering::Relaxed);

    println!("nodes={n_nodes} fanout={fanout} queries={queries}");
    println!("executed={executed} elapsed={dt:?}");
    println!("queries/s={:.2}", queries as f64 / dt.as_secs_f64());
    println!("checksum={checksum_val:#x}");
    println!("(tip) run with --release for real numbers");
}