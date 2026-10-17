//! Distribution recovery policy: replace a dead provider, then explicitly restore
//! the existing matrix grants. Apps retain their state and reconcile on Enter.
use super::*;
use cathedral_platform_contracts::storage;
use cathedral_user_runtime::{task::Port, write};
pub(super) fn regrant(session: &mut admission::Session) {
    for slot in 0..2 {
        owner(
            session.matrix,
            [wire::STORAGE_REVOKE, slot, 0, 0, 0, 0, 0, 0],
        );
    }
    session.storage = Launch::at(3).unwrap().spawn(0).unwrap();
    session.storage_control = Port::at(3).unwrap().connect().unwrap();
    session.storage_control.0.send(storage::HEALTH).unwrap();
    let mut bytes = [0; 64];
    let n = session
        .storage_control
        .1
        .receive_until(&mut bytes, time::after(500).unwrap())
        .unwrap();
    assert_eq!(&bytes[..n], b"ready");
    for slot in 0..2 {
        owner(
            session.matrix,
            [wire::STORAGE_GRANT, slot, slot, 0, 0, 0, 0, 0],
        );
    }
    owner(
        session.matrix,
        wire::route([wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
    );
    write(b"Cathedral matrix desktop: storage replaced; explicit grants restored\n").unwrap();
}
pub(super) fn supervise(session: &mut admission::Session) {
    match session.storage.wait_until(time::now().unwrap()) {
        Ok(_) => regrant(session),
        Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
        Err(error) => panic!("storage supervision: {error:?}"),
    }
}
