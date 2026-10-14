use proptest::prelude::*;
use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;

use flaga::prelude::*;

fn reference_eval(tokens: &[String], flags: &std::collections::HashMap<String, ModuleFlags>) -> bool {
    let mut result: Option<bool> = None;
    let mut current_op: Option<&str> = None;

    for token in tokens {
        match token.as_str() {
            "&&" => current_op = Some("&&"),
            "||" => current_op = Some("||"),
            _ => {
                let negated = token.starts_with('!');
                let flag_expr = token.trim_start_matches('!');
                let value = match flag_expr.split_once('.') {
                    Some((module, index_str)) => {
                        if let Ok(index) = index_str.trim_start_matches("Flag").parse::<u64>() {
                            flags.get(module).map(|m| m.check_flag(index)).unwrap_or(false)
                        } else { false }
                    }
                    None => false,
                };
                let value = if negated { !value } else { value };

                result = Some(match (result, current_op) {
                    (Some(lhs), Some("&&")) => lhs && value,
                    (Some(lhs), Some("||")) => lhs || value,
                    (None, _) => value,
                    (Some(_), _) => value,
                });
            }
        }
    }

    result.unwrap_or(false)
}

fn tokenize_like_parser(input: &str) -> Vec<String> {
    input
        .replace("&&", " && ")
        .replace("||", " || ")
        .replace("!", " !")
        .split_whitespace()
        .map(String::from)
        .collect()
}

proptest! {
    #[test]
    fn parser_matches_reference(seed in any::<u64>()) {
        let mut parser = FlagParser::new();

        // small modules, deterministic.
        parser.parse_flag_string("Core:10101010 Net:11001100 UI:11110000");

        let mut rng = StdRng::seed_from_u64(seed);

        let mk_term = |rng: &mut StdRng| {
            let module = ["Core", "Net", "UI"][rng.gen_range(0..3)];
            let idx = rng.gen_range(0..8);
            let bang = if rng.gen_bool(0.25) { "!" } else { "" };
            format!("{bang}{module}.Flag{idx}")
        };

        let n_terms = rng.gen_range(1..40);
        let mut expr = String::new();
        expr.push_str(&mk_term(&mut rng));
        for _ in 1..n_terms {
            let op = if rng.gen_bool(0.5) { "&&" } else { "||" };
            expr.push(' ');
            expr.push_str(op);
            expr.push(' ');
            expr.push_str(&mk_term(&mut rng));
        }

        let got = parser.check_complex_pattern(&expr);
        let tokens = tokenize_like_parser(&expr);
        let expect = reference_eval(&tokens, &parser.module_flags);

        prop_assert_eq!(got, expect);
    }
}