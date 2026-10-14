use flaga::error::FlagError;
use std::error::Error as _;
#[test]
fn io_error_lifts_through_from_and_exposes_source() {
    let io = std::io::Error::new(std::io::ErrorKind::Other, "disk gone");
    let err: FlagError = io.into();
    assert!(matches!(err, FlagError::IOError(_)));
    assert_eq!(err.source().unwrap().to_string(), "disk gone");
}
