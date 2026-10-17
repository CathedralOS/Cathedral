//! Deliberately block saves on the provider, exercise the other branch, then
//! reconcile a lost acknowledgement through explicit fresh storage grants.
use super::*;
use cathedral_user_runtime::task::Port;
fn app(matrix: Pair, nested: bool, op: u64) -> wire::Packet {
    let words = wire::route([op, 0, 0, 0, 0, 0, 0, 0], if nested { 0 } else { 1 }).unwrap();
    call(
        matrix,
        if nested {
            wire::route(words, 0).unwrap()
        } else {
            words
        },
    )
}
fn receive(pair: Pair, expected: &[u8]) {
    let mut bytes = [0; 64];
    let len = pair
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    assert_eq!(&bytes[..len], expected);
}
pub(super) fn exercise(session: &mut admission::Session) {
    let matrix = session.matrix;
    stalls_peer::start(session.matrix, session.pressure);
    // Normal completion, timeout with a late reply, durable commit without reply.
    for mode in 0..3 {
        let mut marker = *b"Cathedral storage stall mode=0\n";
        marker[29] = b'0' + mode;
        cathedral_user_runtime::write(&marker).unwrap();
        // Stall the nested app's next publication on its root, before execution
        // or after commit; the sibling's root stays unaffected.
        session
            .storage_control
            .0
            .send(&[
                0xf2,
                if mode == 2 { 2 } else { 1 },
                cathedral_platform_contracts::storage::PUBLISH as u8,
                0,
            ])
            .unwrap();
        receive(session.storage_control, b"armed");
        let started = time::now().unwrap();
        assert_eq!(app(matrix, true, 12)[0], 0);
        receive(session.storage_control, b"blocked");
        // Root control, physical-source ingress and sibling rendering still run
        // while storage is demonstrably blocked, not merely after it recovers.
        assert_eq!(
            call(
                session.source,
                [wire::POINTER_SOURCE, 550, 220, 0, 0, 0, 0, 0]
            )[0],
            0
        );
        assert_eq!(app(matrix, false, 1)[0], 0);
        pixel(matrix, 540, 225, app::RIGHT);
        assert_eq!(app(matrix, false, 2)[0], 0);
        pixel(matrix, 540, 225, 0xe5e7eb);
        stalls_peer::interact(session);
        let elapsed = time::now().unwrap().wrapping_sub(started);
        assert!(
            elapsed < 150,
            "sibling waited for storage timeout: {elapsed} ticks"
        );
        if mode == 0 {
            session.storage_control.0.send(&[0xf3]).unwrap();
            assert_eq!(app(matrix, true, wire::HEALTH)[5], 0);
            pixel(matrix, 140, 190, 0xd18b35);
            assert_eq!(app(matrix, true, 8)[0], 0);
            continue;
        }
        if mode == 1 {
            let _ = matrix
                .1
                .receive_until(&mut [0; 64], time::after(180).unwrap());
            assert_eq!(app(matrix, true, wire::HEALTH)[5], abi::TIMED_OUT);
            // A still-live accepted link cannot be recycled after timeout.
            assert_eq!(
                call(matrix, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0])[0],
                abi::BUSY
            );
            session.storage_control.0.send(&[0xf3]).unwrap();
            // Force the now-late reply to be sent before provider retirement.
            session
                .storage_control
                .0
                .send(cathedral_platform_contracts::storage::HEALTH)
                .unwrap();
            receive(session.storage_control, b"ready");
        }
        session.storage.cancel().unwrap();
        session.storage.wait().unwrap();
        let status = app(matrix, true, wire::HEALTH);
        assert_eq!(status[4], 0); // No acknowledgement: app still shows old state.
        if mode == 2 {
            assert_eq!(status[5], abi::PEER_CLOSED);
        }
        for slot in 0..2 {
            owner(matrix, [wire::STORAGE_REVOKE, slot, 0, 0, 0, 0, 0, 0]);
        }
        stalls_peer::interact(session);
        session.storage = Launch::at(3).unwrap().spawn(0).unwrap();
        session.storage_control = Port::at(3).unwrap().connect().unwrap();
        session
            .storage_control
            .0
            .send(cathedral_platform_contracts::storage::HEALTH)
            .unwrap();
        receive(session.storage_control, b"ready");
        for slot in 0..2 {
            owner(matrix, [wire::STORAGE_GRANT, slot, slot, 0, 0, 0, 0, 0]);
        }
        owner(
            matrix,
            wire::route([wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
        );
        assert_eq!(app(matrix, true, 8)[0], abi::DENIED);
        for nested in [true, false] {
            assert_eq!(app(matrix, nested, 7)[0], 0);
        }
        // The timeout's released request and the unacknowledged durable write
        // both committed. Readback discovers that fact; no mutation is retried.
        pixel(matrix, 140, 190, 0xd18b35);
        assert_eq!(app(matrix, true, 8)[0], 0);
        pixel(matrix, 140, 190, app::LEFT);
        stalls_peer::interact(session);
    }
    stalls_peer::replace_writer(session);
    stalls_peer::stop(session);
    cathedral_user_runtime::write(b"Cathedral matrix desktop: nested sibling consumed pointer and keyboard and repainted during blocked persistence\n").unwrap();
    cathedral_user_runtime::write(b"Cathedral matrix desktop: stalled storage preserved sibling rendering; timeout, late reply and lost commit acknowledgement reconciled\n").unwrap();
}
