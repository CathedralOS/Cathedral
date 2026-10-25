//! The stalled saver and this interactive app share the same immediate matrix.
//! The peer has no catalog grant; keyboard changes are intentionally volatile.
use super::*;
fn inner(matrix: Pair, words: wire::Packet) -> wire::Packet {
    call(matrix, wire::route(words, 0).unwrap())
}
fn peer(matrix: Pair) -> wire::Packet {
    inner(
        matrix,
        wire::route([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0], 1).unwrap(),
    )
}
pub(super) fn start(matrix: Pair, pressure: bool) {
    if pressure {
        assert_eq!(inner(matrix, [wire::STOP, 1, 0, 0, 0, 0, 0, 0])[0], 0);
    }
    assert_eq!(inner(matrix, [wire::RUN_PAGES, 1, 1, 0, 0, 0, 0, 0])[0], 0);
    assert_eq!(inner(matrix, [wire::RUN, 1, 1, 2, 0, 0, 0, 0])[0], 0);
    assert_eq!(
        inner(matrix, [wire::SURFACE_LIMIT, 1, 4, 0, 0, 0, 0, 0])[0],
        0
    );
    assert_eq!(
        inner(
            matrix,
            [wire::PLACE, 1, 24, 175, 200, 40, wire::KEY_MASK, 0]
        )[0],
        0
    );
    pixel(matrix, 140, 320, app::RIGHT);
}
pub(super) fn interact(session: &admission::Session) {
    let before = peer(session.matrix);
    for buttons in [1, 0] {
        assert_eq!(
            call(
                session.source,
                [wire::POINTER_SOURCE, 170, 330, buttons, 0, 0, 0, 0]
            )[0],
            0
        );
    }
    for state in [input::PRESS, input::RELEASE] {
        assert_eq!(
            call(
                session.source,
                [
                    wire::KEY_SOURCE,
                    input::ENTER as u64,
                    state as u64,
                    0,
                    0,
                    0,
                    0,
                    0
                ]
            )[0],
            0
        );
    }
    pixel(
        session.matrix,
        140,
        320,
        if before[4] == 0 { 0xd18b35 } else { app::RIGHT },
    );
    let after = peer(session.matrix);
    assert_eq!(after[4], 1 - before[4]);
    assert!(
        after[6] > before[6],
        "nested peer did not consume pointer events"
    );
}
pub(super) fn stop(session: &admission::Session) {
    assert_eq!(
        inner(session.matrix, [wire::STOP, 1, 0, 0, 0, 0, 0, 0])[0],
        0
    );
    pixel(session.matrix, 140, 320, BACKGROUND);
    if session.pressure {
        assert_eq!(
            inner(session.matrix, [wire::RUN, 1, 2, 0, 0, 0, 0, 0])[0],
            0
        );
    }
}
pub(super) fn replace_writer(session: &admission::Session) {
    let matrix = session.matrix;
    assert_eq!(
        inner(matrix, wire::route([10, 0, 0, 0, 0, 0, 0, 0], 0).unwrap())[0],
        abi::PEER_CLOSED
    );
    pixel(matrix, 140, 190, BACKGROUND);
    interact(session);
    assert_eq!(
        inner(
            matrix,
            [wire::RUN_PAGES, 0, APP_PAGES, APP_READERS, 0, 0, 0, 0]
        )[0],
        0
    );
    assert_eq!(
        inner(matrix, [wire::RUN, 0, 1, 0, APP_CHANNELS, 0, 0, 0])[0],
        0
    );
    assert_eq!(
        inner(matrix, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0])[0],
        0
    );
    assert_eq!(
        inner(matrix, [wire::SURFACE_LIMIT, 0, 4, 0, 0, 0, 0, 0])[0],
        0
    );
    assert_eq!(
        inner(
            matrix,
            [wire::PLACE, 0, 24, 40, 200, 130, 1 << (input::ENTER - 1), 0]
        )[0],
        0
    );
    pixel(matrix, 140, 190, app::LEFT);
    interact(session);
    cathedral_user_runtime::write(b"Cathedral matrix desktop: nested peer survived catalog revocation, provider replacement and sibling death\n").unwrap();
}
