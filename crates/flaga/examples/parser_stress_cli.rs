use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use flaga::prelude::*;

fn main() {
    let mut parser = FlagParser::new();

    // Build 64 modules, each with 64 bits.
    let mut cfg = String::new();
    for m in 0..64 {
        let name = format!("M{m}");
        // 64-bit string: 50% density
        let bits: String = (0..64)
            .map(|i| if (i + m) % 2 == 0 { '1' } else { '0' })
            .collect();
        cfg.push_str(&format!("{name}:{bits} "));
    }
    parser.parse_flag_string(&cfg);

    let mut rng = StdRng::seed_from_u64(1234);

    let mk_term = |rng: &mut StdRng| {
        let m = rng.gen_range(0..64);
        let i = rng.gen_range(0..64);
        let bang = if rng.gen_bool(0.25) { "!" } else { "" };
        format!("{bang}M{m}.Flag{i}")
    };

    let mk_expr = |rng: &mut StdRng| {
        let n_terms = rng.gen_range(8..64);
        let mut s = String::new();
        s.push_str(&mk_term(rng));
        for _ in 1..n_terms {
            let op = if rng.gen_bool(0.5) { "&&" } else { "||" };
            s.push(' ');
            s.push_str(op);
            s.push(' ');
            s.push_str(&mk_term(rng));
        }
        s
    };

    let iters = 200_000usize;
    let mut trues = 0usize;

    let start = std::time::Instant::now();
    for _ in 0..iters {
        let expr = mk_expr(&mut rng);
        if parser.check_complex_pattern(&expr) {
            trues += 1;
        }
    }
    let dt = start.elapsed();

    println!("evals={iters} true_count={trues} elapsed={dt:?} eval/s={:.2}",
        iters as f64 / dt.as_secs_f64()
    );
}