// Every path below is the flaga 0.2.0 path; the 0.3.0 facade must keep them.
use flaga::prelude::*;
use flaga::error::FlagError;
use flaga::flag_manager::FlagManager as ManagerByPath;
use flaga::flag_manager_builder::FlagManagerBuilder as BuilderByPath;
use flaga::flag_parser::FlagParser as ParserByPath;
use flaga::flag_event::FlagEvent as EventByPath;
use flaga::flag_store::*;
use flaga::macros_prelude::impl_descriptor;
use std::error::Error as _;

flaga::impl_flag_trait!(FacadeFlags, u32);
impl_descriptor!(FacadeDescriptor, &'static str, derives = []);

#[test]
fn old_module_paths_resolve_to_the_same_types() {
    let m: ManagerByPath = FlagManager::new();
    m.add_flag("A", FlagType::Binary, 0b1, None, None).unwrap();
    m.set_flag("A").unwrap();
    assert!(m.check_flag("A").unwrap());

    let b: FlagManager = BuilderByPath::new()
        .with_store(MemoryFlagStore::new())
        .add_flag("B", FlagType::Hex, 0x10)
        .build()
        .unwrap();
    b.set_flag("B").unwrap();
    b.sync().unwrap();

    let mut p: FlagParser = ParserByPath::new();
    p.parse_flag_string("Core:101");
    assert!(p.check_complex_pattern("Core.Flag0 && !Core.Flag1"));

    let e: FlagEvent = EventByPath::new(0b11, "both");
    assert!(e.should_trigger(0b111));

    let err: Result<(), FlagError> = Err(FlagError::NoStoreConfigured);
    assert!(err.is_err());
}

#[test]
fn macros_reachable_through_the_facade() {
    let mut f = FacadeFlags::new(0);
    f.set_flag(0b100);
    assert!(f.check_flag(0b100));
    assert_eq!(f.value(), 0b100);

    let d = FacadeDescriptor::with_defaults("X", FlagType::Enum, 7u8);
    assert_eq!(d.name, "X");
}

#[test]
fn flag_error_from_io_error_still_exists() {
    fn lift(e: std::io::Error) -> FlagError {
        <FlagError as From<std::io::Error>>::from(e)
    }
    let err = lift(std::io::Error::other("gone"));
    assert!(matches!(err, FlagError::IOError(_)));
    assert_eq!(err.source().unwrap().to_string(), "gone");
}
