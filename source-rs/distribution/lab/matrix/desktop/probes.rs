//! Check quiet clients and retained placement while a capped sibling spins.
use super::*;
fn stats(matrix: Pair, nested: bool) -> wire::Packet {
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
pub(super) fn exercise(matrix: Pair) {
    // Drain startup wake hints before measuring the quiet interval.
    let _ = matrix
        .1
        .receive_until(&mut [0; 64], time::after(30).unwrap());
    let before = [stats(matrix, true), stats(matrix, false)];
    let _ = matrix
        .1
        .receive_until(&mut [0; 64], time::after(100).unwrap());
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
    assert_eq!(stats(matrix, true)[..3], after[0][..3]);
    owner(
        matrix,
        [wire::PLACE, 0, 110, 140, 280, 220, wire::KEY_MASK, 0],
    );
    pixel(matrix, 140, 190, app::LEFT);
    surfaces(matrix);
    cathedral_user_runtime::write(b"Cathedral matrix desktop: quiet apps below 20ms CPU per second; retained move without app repaint\n").unwrap();
}

fn surfaces(matrix: Pair) {
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
