#![no_main]
use libfuzzer_sys::fuzz_target;

use flagger::prelude::*;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else { return; };

    let mut parser = FlagParser::new();
    parser.parse_flag_string("Core:1010 Net:0101 UI:1111");

    // Must never panic:
    let _ = parser.check_complex_pattern(s);
});