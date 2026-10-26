//! Deliberately block saves on the provider, exercise the other branch, then
//! reconcile a lost acknowledgement through explicit fresh storage grants.
use super::*;
fn app(matrix: &mut RootControl, nested: bool, op: u64) -> wire::Packet {
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
pub(super) fn exercise(session: &mut admission::Session) {
    let mut source = source::Source::open(session.source.take().expect("untouched lab source"));
    stalls_peer::start(&mut session.matrix, session.pressure);
    // Normal completion, timeout with a late reply, durable commit without reply.
    for mode in 0..3 {
        let mut marker = *b"Cathedral storage stall mode=0\n";
        marker[29] = b'0' + mode;
        cathedral_user_runtime::write(&marker).unwrap();
        // Stall the nested app's next publication on its root, before execution
        // or after commit; the sibling's root stays unaffected.
        gate::arm(session.provider(), if mode == 2 { 2 } else { 1 });
        let started = time::now().unwrap();
        assert_eq!(app(&mut session.matrix, true, 12)[0], 0);
        gate::blocked(session.provider());
        // Root control, physical-source ingress and sibling rendering still run
        // while storage is demonstrably blocked, not merely after it recovers.
        assert_eq!(
            source.request([wire::POINTER_SOURCE, 550, 220, 0, 0, 0, 0, 0])[0],
            0
        );
        assert_eq!(app(&mut session.matrix, false, 1)[0], 0);
        pixel(&mut session.matrix, 540, 225, app::RIGHT);
        assert_eq!(app(&mut session.matrix, false, 2)[0], 0);
        pixel(&mut session.matrix, 540, 225, 0xe5e7eb);
        stalls_peer::interact(session, &mut source);
        let elapsed = time::now().unwrap().wrapping_sub(started);
        assert!(
            elapsed < 150,
            "sibling waited for storage timeout: {elapsed} ticks"
        );
        if mode == 0 {
            gate::release(session.provider());
            assert_eq!(app(&mut session.matrix, true, wire::HEALTH)[5], 0);
            pixel(&mut session.matrix, 140, 190, 0xd18b35);
            assert_eq!(app(&mut session.matrix, true, 8)[0], 0);
            continue;
        }
        if mode == 1 {
            let _ = session
                .matrix
                .idle(time::after(wire::STORAGE_DEADLINE_TICKS + 50).unwrap());
            assert_eq!(
                app(&mut session.matrix, true, wire::HEALTH)[5],
                abi::TIMED_OUT
            );
            // A still-live accepted link cannot be recycled after timeout.
            assert_eq!(
                call(
                    &mut session.matrix,
                    [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]
                )[0],
                abi::BUSY
            );
            gate::release(session.provider());
            // Force the now-late reply to be sent before provider retirement.
            gate::expect(
                session.provider(),
                cathedral_platform_contracts::storage::HEALTH,
                b"ready",
            );
        }
        session.provider().child.cancel().unwrap();
        session.provider().child.wait().unwrap();
        let status = app(&mut session.matrix, true, wire::HEALTH);
        assert_eq!(status[4], 0); // No acknowledgement: app still shows old state.
        if mode == 2 {
            assert_eq!(status[5], abi::PEER_CLOSED);
        }
        for slot in 0..2 {
            owner(
                &mut session.matrix,
                [wire::STORAGE_REVOKE, slot, 0, 0, 0, 0, 0, 0],
            );
        }
        stalls_peer::interact(session, &mut source);
        session.storage.take().unwrap().retire();
        session.storage = Some(persistence::Provider::start().unwrap());
        for slot in 0..2 {
            owner(
                &mut session.matrix,
                [wire::STORAGE_GRANT, slot, slot, 0, 0, 0, 0, 0],
            );
        }
        owner(
            &mut session.matrix,
            wire::route([wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
        );
        assert_eq!(app(&mut session.matrix, true, 8)[0], abi::DENIED);
        for nested in [true, false] {
            assert_eq!(app(&mut session.matrix, nested, 7)[0], 0);
        }
        // A request stalled before execution expires without a write. A request
        // stalled after commit is durable despite its missing acknowledgement.
        // Readback distinguishes them; neither uncertain mutation is retried.
        pixel(
            &mut session.matrix,
            140,
            190,
            if mode == 1 { app::LEFT } else { 0xd18b35 },
        );
        assert_eq!(app(&mut session.matrix, true, 8)[0], 0);
        if mode == 1 {
            pixel(&mut session.matrix, 140, 190, 0xd18b35);
            assert_eq!(app(&mut session.matrix, true, 8)[0], 0);
        }
        pixel(&mut session.matrix, 140, 190, app::LEFT);
        stalls_peer::interact(session, &mut source);
    }
    stalls_peer::replace_writer(session, &mut source);
    stalls_peer::stop(session);
    cathedral_user_runtime::write(b"Cathedral matrix desktop: nested sibling consumed pointer and keyboard and repainted during blocked persistence\n").unwrap();
    cathedral_user_runtime::write(b"Cathedral matrix desktop: stalled storage preserved sibling rendering; timeout, late reply and lost commit acknowledgement reconciled\n").unwrap();
}
