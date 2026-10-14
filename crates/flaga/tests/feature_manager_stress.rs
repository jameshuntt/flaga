use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use flaga::prelude::*;

struct IncFeature {
    ctr: Arc<AtomicU64>,
    n: u64,
}

impl Feature for IncFeature {
    fn execute(&self) -> Result<(), String> {
        self.ctr.fetch_add(self.n, Ordering::Relaxed);
        Ok(())
    }
}

#[test]
fn feature_manager_10k_features() {
    let ctr = Arc::new(AtomicU64::new(0));
    let mut fm = FeatureManager::new();

    // FeatureManager requires &'static str keys; for stress tests we can leak.
    for i in 0..10_000u64 {
        let name = format!("F{i}");
        let static_name: &'static str = Box::leak(name.into_boxed_str());
        fm.register_feature(static_name, IncFeature { ctr: ctr.clone(), n: 1 });
    }

    for i in 0..10_000u64 {
        let name = format!("F{i}");
        let static_name: &'static str = Box::leak(name.into_boxed_str());
        fm.execute_feature(static_name).unwrap();
    }

    assert_eq!(ctr.load(std::sync::atomic::Ordering::Relaxed), 10_000);
}