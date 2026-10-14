// // tests/flag_manager_sanitize_ghost_bits.rs
// 
// use tempfile::tempdir;
// use flaga::{flag_manager::FlagManager, prelude::*};
// 
// #[test]
// fn sanitize_clears_ghost_bits_so_new_flags_dont_accidentally_enable() {
//     let dir = tempdir().unwrap();
//     let path = dir.path().join("state.bin");
// 
//     // v1: only B0 exists.
//     let mgr_v1 = FlagManagerBuilder::new()
//         .with_file(path.clone())
//         .add_flag("B0", FlagType::Binary, 0b0000_0001)
//         .build();
// 
//     mgr_v1.set_flag("B0").unwrap();
// 
//     // Inject a ghost bit (simulates old versions).
//     mgr_v1.__test_inject_masks(0b0000_0101, 0, 0); // B0 + ghost 0b0100
//     mgr_v1.sync_to_disk().unwrap();
// 
//     // v2 introduces NEW_FEATURE = 0b0100; sanitize-on-load should clear it.
//     let mgr_v2 = FlagManagerBuilder::new()
//         .with_file(path.clone())
//         .add_flag("B0", FlagType::Binary, 0b0000_0001)
//         .add_flag("NEW_FEATURE", FlagType::Binary, 0b0000_0100)
//         .build();
// 
//     assert!(mgr_v2.check_flag("B0").unwrap());
//     assert!(!mgr_v2.check_flag("NEW_FEATURE").unwrap());
// }