// use std::path::PathBuf;

// use flaga::prelude::*;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let path = PathBuf::from("target/flagger_demo_state.bin");
//     std::fs::create_dir_all("target")?;

//     // First run: create (or load), define baseline flags, set some, sync.
//     let mgr = FlagManagerBuilder::new()
//         .with_file(path.clone())
//         .add_flag("ENABLE_LOGGING", FlagType::Binary, 0b0000_0001)
//         .add_flag("TRACE_IO", FlagType::Binary, 0b0000_0010)
//         .add_flag("HEX_A", FlagType::Hex, 0x0000_0000_0000_0001)
//         .build();

//     mgr.set_flag("ENABLE_LOGGING")?;
//     mgr.set_flag("HEX_A")?;
//     mgr.sync_to_disk()?;

//     // "Next deploy": same file, but code now introduces a new flag definition.
//     let mgr2 = FlagManagerBuilder::new()
//         .with_file(path.clone())
//         .add_flag("ENABLE_LOGGING", FlagType::Binary, 0b0000_0001)
//         .add_flag("TRACE_IO", FlagType::Binary, 0b0000_0010)
//         .add_flag("NEW_FEATURE", FlagType::Binary, 0b0000_0100)
//         .add_flag("HEX_A", FlagType::Hex, 0x0000_0000_0000_0001)
//         .build();

//     // The old bits should still be set after load.
//     assert!(mgr2.check_flag("ENABLE_LOGGING")?);
//     assert!(mgr2.check_flag("HEX_A")?);
//     assert!(!mgr2.check_flag("NEW_FEATURE")?);

//     // External process could modify the file; refresh pulls it in.
//     mgr2.set_flag("NEW_FEATURE")?;
//     mgr2.sync_to_disk()?;

//     let mgr3 = FlagManager::load_from_file(&path)?;
//     assert!(mgr3.check_flag("NEW_FEATURE")?);

//     println!("OK: hot reload + merge + persistence works.");
//     Ok(())
// }

use std::path::PathBuf;
use flaga::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("target/flagger_demo_state.bin");
    std::fs::create_dir_all("target")?;

    // Make the demo deterministic.
    if path.exists() {
        std::fs::remove_file(&path)?;
        println!("(demo) removed old persisted state: {}", path.display());
    }

    // First run: create, define baseline flags, set some, sync.
    let mgr = FlagManagerBuilder::new()
        .with_file(path.clone())
        .add_flag("ENABLE_LOGGING", FlagType::Binary, 0b0000_0001)
        .add_flag("TRACE_IO",       FlagType::Binary, 0b0000_0010)
        .add_flag("HEX_A",          FlagType::Hex,    0x1)
        .build()
        .unwrap();

    mgr.set_flag("ENABLE_LOGGING")?;
    mgr.set_flag("HEX_A")?;
    mgr.sync_to_disk()?;

    // "Next deploy": same file, code introduces new flag definition.
    let mgr2 = FlagManagerBuilder::new()
        .with_file(path.clone())
        .load_or_default() // merge the persisted state; without this build() starts fresh
        .add_flag("ENABLE_LOGGING", FlagType::Binary, 0b0000_0001)
        .add_flag("TRACE_IO",       FlagType::Binary, 0b0000_0010)
        .add_flag("NEW_FEATURE",    FlagType::Binary, 0b0000_0100)
        .add_flag("HEX_A",          FlagType::Hex,    0x1)
        .build()
        .unwrap();

    assert!(mgr2.check_flag("ENABLE_LOGGING")?);
    assert!(mgr2.check_flag("HEX_A")?);
    assert!(!mgr2.check_flag("NEW_FEATURE")?);

    mgr2.set_flag("NEW_FEATURE")?;
    mgr2.sync_to_disk()?;

    let mgr3 = FlagManager::load_from_file(&path)?;
    assert!(mgr3.check_flag("NEW_FEATURE")?);

    println!("OK: hot reload + merge + persistence works.");
    Ok(())
}