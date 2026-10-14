use tempfile::tempdir;

use flaga::prelude::*;

#[test]
fn persistence_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state.bin");

    let mgr = FlagManager::new().with_file_path(&path);
    mgr.add_flag("B0", FlagType::Binary, 0b0000_0001, None, None).unwrap();
    mgr.add_flag("B1", FlagType::Binary, 0b0000_0010, None, None).unwrap();
    mgr.add_flag("H0", FlagType::Hex, 0x1, None, None).unwrap();

    mgr.set_flag("B0").unwrap();
    mgr.set_flag("H0").unwrap();

    mgr.sync_to_disk().unwrap();

    let loaded = FlagManager::load_from_file(&path).unwrap();
    assert!(loaded.check_flag("B0").unwrap());
    assert!(!loaded.check_flag("B1").unwrap());
    assert!(loaded.check_flag("H0").unwrap());
}