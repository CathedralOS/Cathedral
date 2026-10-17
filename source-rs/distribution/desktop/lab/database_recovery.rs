//! Distribution-owned database failure exercise. Child apps keep their identities;
//! the owner replaces the disk provider and explicitly re-grants local catalogs.
use super::*;
use cathedral_platform_contracts::storage;
use cathedral_user_runtime::write;
fn receive(pair: Pair, expected: &[u8]) {
    let mut bytes = [0; 64];
    let n = pair
        .1
        .receive_until(&mut bytes, time::after(500).unwrap())
        .unwrap();
    assert_eq!(&bytes[..n], expected);
}
fn arm(control: Pair, phase: u8, opcode: u8) {
    control.0.send(&[0xf2, phase, opcode, 0]).unwrap();
    receive(control, b"armed");
}
pub(crate) fn arm_startup(control: Pair) -> bool {
    control.0.send(&[0xf4]).unwrap();
    let mut bytes = [0; 16];
    assert_eq!(control.1.receive(&mut bytes).unwrap(), 16);
    let fresh = bytes == [0; 16];
    if fresh {
        arm(control, 2, storage::PUBLISH as u8);
    }
    fresh
}
fn app(matrix: Pair, op: u64) -> wire::Packet {
    call(
        matrix,
        wire::route(wire::route([op, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(), 0).unwrap(),
    )
}
fn retire(session: &mut admission::Session) {
    session.storage.cancel().unwrap();
    assert_ne!(app(session.matrix, 13)[0], 0);
    for slot in 0..2 {
        owner(
            session.matrix,
            [wire::STORAGE_REVOKE, slot, 0, 0, 0, 0, 0, 0],
        );
    }
}
fn restart(session: &mut admission::Session) {
    persistence::supervise(session);
    // Reconnection is explicit. This consumes an unresolved intent by reading;
    // it must never toggle again because the old acknowledgement was missing.
    assert_eq!(app(session.matrix, 7)[0], 0);
    assert_eq!(
        call(
            session.matrix,
            wire::route([7, 0, 0, 0, 0, 0, 0, 0], 1).unwrap()
        )[0],
        0
    );
}
pub(super) fn exercise(session: &mut admission::Session) {
    if !session.database_fresh {
        stalls_peer::stop(session);
        return;
    }
    receive(session.storage_control, b"blocked");
    // Initial CREATE committed, but the app never received success.
    stalls_peer::interact(session);
    // Let the actual matrix deadline expire while the provider remains alive.
    // This reproduces the startup failure that used to panic on CREATE.
    assert_eq!(
        session.matrix.1.receive_until(
            &mut [0; 64],
            time::after(wire::STORAGE_DEADLINE_TICKS + 50).unwrap()
        ),
        Err(Error(abi::TIMED_OUT as i64))
    );
    assert_eq!(app(session.matrix, 13)[0], abi::TIMED_OUT);
    pixel(session.matrix, 170, 225, 0xe96f6f);
    let old = session.storage.raw();
    let deadline = time::after(1000).unwrap();
    while session.storage.raw() == old {
        persistence::supervise(session);
        stalls_peer::interact(session);
        assert!(!time::reached(time::now().unwrap(), deadline));
    }
    assert_eq!(app(session.matrix, 7)[0], 0);
    assert_eq!(
        call(
            session.matrix,
            wire::route([7, 0, 0, 0, 0, 0, 0, 0], 1).unwrap()
        )[0],
        0
    );
    write(b"Cathedral database desktop: live stall watchdog restored grants\n").unwrap();
    assert_eq!(app(session.matrix, wire::HEALTH)[7], 1);
    pixel(session.matrix, 140, 190, app::LEFT);
    write(b"Cathedral database desktop: uncertain CREATE reconciled without duplication\n")
        .unwrap();
    for phase in 1..=2 {
        arm(session.storage_control, phase, storage::PUBLISH as u8);
        let started = time::now().unwrap();
        assert_eq!(app(session.matrix, 12)[0], 0);
        receive(session.storage_control, b"blocked");
        stalls_peer::interact(session);
        assert!(time::now().unwrap().wrapping_sub(started) < 150);
        retire(session);
        assert_eq!(app(session.matrix, wire::HEALTH)[4], 0);
        stalls_peer::interact(session);
        restart(session);
        let health = app(session.matrix, wire::HEALTH);
        assert_eq!(health[4], u64::from(phase == 2));
        assert_eq!(health[7], if phase == 1 { 1 } else { 2 });
        pixel(
            session.matrix,
            140,
            190,
            if phase == 1 { app::LEFT } else { 0xd18b35 },
        );
    }
    assert_eq!(app(session.matrix, 8)[0], 0); // One deliberate, acknowledged toggle back.
    assert_eq!(app(session.matrix, wire::HEALTH)[7], 3);
    stalls_peer::stop(session);
    write(b"Cathedral database desktop: precommit and lost-reply replacement preserved live siblings and exact revisions\n").unwrap();
}
