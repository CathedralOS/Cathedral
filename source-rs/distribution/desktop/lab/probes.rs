//! Check quiet clients and retained placement while a capped sibling spins.
use super::*;
fn stats(matrix: &mut RootControl, nested: bool) -> wire::Packet {
    let mut words = wire::route(
        [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0],
        if nested { 0 } else { 1 },
    )
    .unwrap();
    if nested {
        words = wire::route(words, 0).unwrap();
    }
    call(matrix, words)
}
pub(super) fn exercise(matrix: &mut RootControl) {
    // Let startup settle before measuring quiet work; only the root lifetime
    // endpoint participates, never a notification hint as an unsolicited reply.
    let _ = matrix.idle(time::after(30).unwrap());
    let before = [stats(matrix, true), stats(matrix, false)];
    let _ = matrix.idle(time::after(100).unwrap());
    let after = [stats(matrix, true), stats(matrix, false)];
    for (before, after) in before.into_iter().zip(after) {
        assert_eq!(before[..3], after[..3]);
        assert!(
            after[3] - before[3] < 20_000,
            "idle app burned CPU: {before:?} -> {after:?}"
        );
    }
    owner(
        matrix,
        [wire::PLACE, 0, 160, 140, 280, 220, wire::KEY_MASK, 0],
    );
    pixel(matrix, 140, 190, BACKGROUND);
    pixel(matrix, 190, 190, app::LEFT);
    pixel(matrix, 316, 292, app::LEFT);
    pixel(matrix, 366, 292, 0xffcc44);
    pixel(matrix, 366, 299, 0x7755dd);
    assert_eq!(stats(matrix, true)[..3], after[0][..3]);
    owner(
        matrix,
        [wire::PLACE, 0, 110, 140, 280, 220, wire::KEY_MASK, 0],
    );
    pixel(matrix, 140, 190, app::LEFT);
    bitmap(matrix);
    cathedral_user_runtime::write(b"Cathedral matrix desktop: sealed bitmap forwarding, clipping, replay and subtree reclamation passed\n").unwrap();
    surfaces(matrix);
    presentation(matrix);
    cursor(matrix);
    cathedral_user_runtime::write(b"Cathedral matrix desktop: quiet apps below 20ms CPU per second; retained move without app repaint\n").unwrap();
}

fn cursor(matrix: &mut RootControl) {
    let app = |op| wire::route(wire::route([op, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(), 0).unwrap();
    assert_eq!(call(matrix, app(14))[0], 0);
    pixel(matrix, 170, 225, 0xe5e7eb); // Staging has not changed the scene.
    assert_eq!(call(matrix, app(4))[0], 0);
    pixel(matrix, 164, 220, 0xffd166);
    pixel(matrix, 170, 225, app::LEFT); // Transparent cursor pixels preserve the app.
    owner(
        matrix,
        [wire::PLACE, 0, 160, 140, 280, 220, wire::KEY_MASK, 0],
    );
    pixel(matrix, 214, 220, 0xffd166); // Retained move without child repaint.
    pixel(matrix, 164, 220, BACKGROUND);
    owner(
        matrix,
        [wire::PLACE, 0, 110, 140, 280, 220, wire::KEY_MASK, 0],
    );
    assert_eq!(call(matrix, app(15))[0], 0);
    pixel(matrix, 164, 220, 0xffd166);
    assert_eq!(call(matrix, app(4))[0], 0);
    pixel(matrix, 332, 180, 0xffd166);
    pixel(matrix, 333, 182, 0xffd166);
    pixel(matrix, 334, 182, BACKGROUND); // No escape past the leaf's right edge.
    assert_eq!(call(matrix, app(6))[0], 0);
    pixel(matrix, 332, 180, app::LEFT);
    pixel(matrix, 170, 225, 0xe5e7eb);
    cathedral_user_runtime::write(b"Cathedral matrix desktop: scoped cursor clipping, transparency, retained move and atomic publication passed\n").unwrap();
}

fn presentation(matrix: &mut RootControl) {
    let app = |op| wire::route(wire::route([op, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(), 0).unwrap();
    assert_eq!(call(matrix, app(3))[0], 0);
    pixel(matrix, 140, 190, app::LEFT);
    pixel(matrix, 170, 225, 0xe5e7eb);
    assert_eq!(call(matrix, app(4))[0], 0);
    assert_eq!(call(matrix, app(4))[0], 0); // Retry belongs in the proof, not every repaint.
    pixel(matrix, 140, 190, 0x884422);
    pixel(matrix, 170, 225, 0x224488);
    assert_eq!(call(matrix, app(5))[0], 0);
    pixel(matrix, 140, 190, 0x884422);
    pixel(matrix, 170, 225, 0x224488);
    pixel(matrix, 510, 190, app::RIGHT);
    assert_eq!(call(matrix, app(11))[0], 0);
    pixel(matrix, 140, 190, 0x884422); // Neither compact rectangle was published.
    assert_eq!(call(matrix, app(6))[0], 0);
    assert_eq!(call(matrix, app(4))[0], 0); // Compact publication shares commit retry history.
    pixel(matrix, 140, 190, app::LEFT);
    pixel(matrix, 170, 225, 0xe5e7eb);
    cathedral_user_runtime::write(
        b"Cathedral matrix desktop: atomic presentation, rejection and commit retry passed\n",
    )
    .unwrap();
}

fn surfaces(matrix: &mut RootControl) {
    let app = |operation| {
        wire::route(wire::route([operation, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(), 0).unwrap()
    };
    pixel(matrix, 170, 225, 0xe5e7eb);
    assert_eq!(call(matrix, app(1))[0], 0);
    pixel(matrix, 170, 225, app::LEFT);
    pixel(matrix, 510, 190, app::RIGHT);
    assert_eq!(call(matrix, app(2))[0], 0);
    pixel(matrix, 170, 225, 0xe5e7eb);
    owner(
        matrix,
        [wire::PLACE, 0, 500, 140, 280, 220, wire::KEY_MASK, 0],
    );
    pixel(matrix, 530, 190, app::RIGHT);
    owner(
        matrix,
        [wire::PLACE, 0, 110, 140, 280, 220, wire::KEY_MASK, 0],
    );
    pixel(matrix, 170, 225, 0xe5e7eb);
    cathedral_user_runtime::write(b"Cathedral matrix desktop: owned surface quotas, stale names, removal and subtree stacking passed\n").unwrap();
}

pub(super) fn startup(matrix: &mut RootControl, pressure: bool) {
    if !recovery::restored(matrix) {
        pixel(matrix, 140, 190, app::LEFT);
        pixel(matrix, 510, 190, app::RIGHT);
        bitmap(matrix);
        pixel(matrix, 334, 310, BACKGROUND); // Oversized leaf draw was clipped twice.
        for _ in 0..3 {
            owner(matrix, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
            pixel(matrix, 140, 190, BACKGROUND);
            pixel(matrix, 316, 292, BACKGROUND);
            pixel(matrix, 682, 292, 0xffcc44);
            pixel(matrix, 510, 190, app::RIGHT);
            inner(matrix, pressure);
            pixel(matrix, 140, 190, app::LEFT);
            bitmap(matrix);
        }
        cathedral_user_runtime::write(
        b"Cathedral matrix desktop: dynamic catalogue admission, rollback and pool reuse passed\n",
    )
    .unwrap();
        cathedral_user_runtime::write(
            b"Cathedral matrix desktop: subtree replacement cleared pixels and preserved sibling\n",
        )
        .unwrap();
        // The source endpoint grants input ingress, never owner or display authority.
        exercise(matrix);
        recovery::exercise(matrix, pressure);
    }
}
