//! Distribution-owned database failure exercise. Child apps keep their identities;
//! the owner replaces the disk provider and explicitly re-grants local catalogs.
use super::*;
use cathedral_user_runtime::write;
pub(crate) fn arm_startup(provider: &mut persistence::Provider) -> bool {
    let (bytes, n) = provider.command(&[0xf4]);
    assert_eq!(n, 16);
    let fresh = bytes[..n] == [0; 16];
    if fresh {
        gate::arm(provider, 2);
    }
    fresh
}
fn app(matrix: &mut RootControl, op: u64) -> wire::Packet {
    call(
        matrix,
        wire::route(wire::route([op, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(), 0).unwrap(),
    )
}
fn retire(session: &mut admission::Session) {
    session.provider().child.cancel().unwrap();
    assert_ne!(app(&mut session.matrix, 13)[0], 0);
    for slot in 0..2 {
        owner(
            &mut session.matrix,
            [wire::STORAGE_REVOKE, slot, 0, 0, 0, 0, 0, 0],
        );
    }
}
fn restart(session: &mut admission::Session) {
    persistence::supervise(session);
    // Reconnection is explicit. This consumes an unresolved intent by reading;
    // it must never toggle again because the old acknowledgement was missing.
    assert_eq!(app(&mut session.matrix, 7)[0], 0);
    assert_eq!(
        call(
            &mut session.matrix,
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
    gate::blocked(session.provider());
    // Initial CREATE committed, but the app never received success.
    stalls_peer::interact(session);
    // Let the actual matrix deadline expire while the provider remains alive.
    // This reproduces the startup failure that used to panic on CREATE.
    assert_eq!(
        session
            .matrix
            .idle(time::after(wire::STORAGE_DEADLINE_TICKS + 50).unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
    assert_eq!(app(&mut session.matrix, 13)[0], abi::TIMED_OUT);
    pixel(&mut session.matrix, 170, 225, 0xe96f6f);
    let old = session.provider().child.raw();
    let deadline = time::after(1000).unwrap();
    while session.provider().child.raw() == old {
        persistence::supervise(session);
        stalls_peer::interact(session);
        assert!(!time::reached(time::now().unwrap(), deadline));
    }
    assert_eq!(app(&mut session.matrix, 7)[0], 0);
    assert_eq!(
        call(
            &mut session.matrix,
            wire::route([7, 0, 0, 0, 0, 0, 0, 0], 1).unwrap()
        )[0],
        0
    );
    write(b"Cathedral database desktop: live stall watchdog restored grants\n").unwrap();
    assert_eq!(app(&mut session.matrix, wire::HEALTH)[7], 1);
    pixel(&mut session.matrix, 140, 190, app::LEFT);
    write(b"Cathedral database desktop: uncertain CREATE reconciled without duplication\n")
        .unwrap();
    for phase in 1..=2 {
        gate::arm(session.provider(), phase);
        let started = time::now().unwrap();
        assert_eq!(app(&mut session.matrix, 12)[0], 0);
        gate::blocked(session.provider());
        stalls_peer::interact(session);
        assert!(time::now().unwrap().wrapping_sub(started) < 150);
        retire(session);
        assert_eq!(app(&mut session.matrix, wire::HEALTH)[4], 0);
        stalls_peer::interact(session);
        restart(session);
        let health = app(&mut session.matrix, wire::HEALTH);
        assert_eq!(health[4], u64::from(phase == 2));
        assert_eq!(health[7], if phase == 1 { 1 } else { 2 });
        pixel(
            &mut session.matrix,
            140,
            190,
            if phase == 1 { app::LEFT } else { 0xd18b35 },
        );
    }
    assert_eq!(app(&mut session.matrix, 8)[0], 0); // One deliberate, acknowledged toggle back.
    assert_eq!(app(&mut session.matrix, wire::HEALTH)[7], 3);
    stalls_peer::stop(session);
    write(b"Cathedral database desktop: precommit and lost-reply replacement preserved live siblings and exact revisions\n").unwrap();
}
