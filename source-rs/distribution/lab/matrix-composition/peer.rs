//! An ordinary peer can relay packets only through its explicitly supplied link.
use super::*;
use cathedral_user_runtime::{link::Link, task::Launch};
pub fn run(environment: cathedral_user_runtime::task::Environment) -> u64 {
    assert!(Launch::at(0).is_err());
    assert!(Link::at(0).is_err());
    assert!(cathedral_user_runtime::display::mapping().is_err());
    assert!(cathedral_user_runtime::keyboard::read(false).is_err());
    let connections =
        cathedral_user_runtime::matrix::OptionalConnections::load(&environment).unwrap();
    assert!(connections.storage.is_none());
    let parent = connections.interaction;
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    loop {
        let mut bytes = [0; 64];
        let len = control.0.receive(&mut bytes).unwrap();
        // Relay malformed payloads too, so the real provider checks them.
        parent.0.send(&bytes[..len]).unwrap();
        let len = parent
            .1
            .receive_until(&mut bytes, time::after(300).unwrap())
            .unwrap();
        control.1.send(&bytes[..len]).unwrap();
    }
}
