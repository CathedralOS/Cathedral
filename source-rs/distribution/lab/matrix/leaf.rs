//! Test peer relays probes over its sole parent endpoint. No world/global lookup.
use super::*;
use cathedral_user_runtime::{link::Link, task::Launch};
pub const EXIT_PROBE: u64 = 0xfffe;
pub fn run(environment: cathedral_user_runtime::task::Environment) -> u64 {
    assert_eq!(environment.len(), 3);
    assert!(matches!(Launch::at(0), Err(Error(code)) if code == abi::DENIED as i64));
    assert!(Link::at(0).is_err());
    assert_eq!(
        cathedral_user_runtime::display::mapping(),
        Err(Error(abi::DENIED as i64))
    );
    assert_eq!(
        cathedral_user_runtime::keyboard::read(false),
        Err(Error(abi::DENIED as i64))
    );
    assert_eq!(
        cathedral_user_runtime::disk::read(7),
        Err(Error(abi::DENIED as i64))
    );
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let parent = (
        environment.endpoint(0).unwrap(),
        environment.endpoint(1).unwrap(),
    );
    loop {
        let mut bytes = [0; 64];
        let Ok(len) = control.0.receive(&mut bytes) else {
            return 0;
        };
        let Some(words) = wire::decode(&bytes[..len]) else {
            return 1;
        };
        if words == [EXIT_PROBE, 0, 0, 0, 0, 0, 0, 0] {
            return 0;
        }
        control.1.send(&wire::encode(call(parent, words))).unwrap();
    }
}
