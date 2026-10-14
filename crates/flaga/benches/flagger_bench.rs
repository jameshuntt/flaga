use criterion::{criterion_group, criterion_main, Criterion, black_box};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use flaga::prelude::*;

fn bench_flag_manager_set_check(c: &mut Criterion) {
    let mgr = FlagManager::new();
    mgr.add_flag("B0", FlagType::Binary, 0b0000_0001, None, None).unwrap();
    mgr.add_flag("H0", FlagType::Hex, 1, None, None).unwrap();

    c.bench_function("FlagManager::set_flag(B0)", |b| {
        b.iter(|| mgr.set_flag(black_box("B0")).unwrap())
    });

    c.bench_function("FlagManager::check_flag(B0)", |b| {
        b.iter(|| black_box(mgr.check_flag(black_box("B0")).unwrap()))
    });

    c.bench_function("FlagManager::set_flag(H0)", |b| {
        b.iter(|| mgr.set_flag(black_box("H0")).unwrap())
    });
}

fn bench_parser_eval(c: &mut Criterion) {
    let mut parser = FlagParser::new();
    parser.parse_flag_string("Core:11110000 Net:11001100 UI:10101010");

    let expr = "!Core.Flag0 && Net.Flag1 || UI.Flag2 && !Net.Flag3 || Core.Flag4 && UI.Flag5";

    c.bench_function("FlagParser::check_complex_pattern", |b| {
        b.iter(|| black_box(parser.check_complex_pattern(black_box(expr))))
    });
}

fn bench_event_update(c: &mut Criterion) {
    let mut em = FlagEventManager { events: Vec::new(), current_flags: 0 };

    // 50k events with random masks
    let mut rng = StdRng::seed_from_u64(123);
    for i in 0..50_000 {
        let mask: u8 = rng.r#gen();
        em.add_event(mask, format!("E{i}"));
    }

    c.bench_function("FlagEventManager::update_flags(50k)", |b| {
        let mut x: u8 = 0;
        b.iter(|| {
            x = x.wrapping_add(1);
            em.update_flags(black_box(x));
            black_box(em.current_flags);
        })
    });
}

criterion_group!(
    benches,
    bench_flag_manager_set_check,
    bench_parser_eval,
    bench_event_update
);
criterion_main!(benches);