use proptest::prelude::*;

use flaga::{flag_group::FlagGroup, prelude::*};

#[derive(Clone, Debug)]
enum Op {
    Set { idx: usize, bit: u8 },
    Clear { idx: usize, bit: u8 },
    Toggle { idx: usize, bit: u8 },
}

fn bit_to_u64(bit: u8) -> u64 { bit as u64 }

proptest! {
    #[test]
    fn flag_group_matches_naive_model(
        len in 1usize..64,
        ops in prop::collection::vec(
            (0usize..64, 0u8..64, 0u8..3),
            1..2000
        )
    ) {
        let mut g = FlagGroup::<ModuleFlags>::new();
        let mut model: Vec<u64> = Vec::new();

        for _ in 0..len {
            g.add_flag(ModuleFlags::new());
            model.push(0);
        }

        for (idx_raw, bit, kind) in ops {
            let idx = idx_raw % len;
            let b = bit_to_u64(bit);
            match kind {
                0 => { // set
                    g.set_flag(idx, b).unwrap();
                    model[idx] |= 1u64 << b;
                }
                1 => { // clear
                    g.clear_flag(idx, b).unwrap();
                    model[idx] &= !(1u64 << b);
                }
                _ => { // toggle
                    g.toggle_flag(idx, b).unwrap();
                    model[idx] ^= 1u64 << b;
                }
            }

            // spot-check: raw u64 state matches
            let got = g.get_flags(idx).unwrap();
            prop_assert_eq!(got, model[idx]);
        }
    }
}