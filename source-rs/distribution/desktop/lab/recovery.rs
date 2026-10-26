//! One visible session proves persistence, revoked writes and interrupted work.
//! All control is owner-routed; the apps hold only their own catalog bindings.
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
fn visible(matrix: &mut RootControl, nested: bool) -> u64 {
    let deadline = time::after(1000).unwrap();
    loop {
        let reply = call(
            matrix,
            [
                wire::READ_PIXEL,
                if nested { 140 } else { 510 },
                190,
                0,
                0,
                0,
                0,
                0,
            ],
        );
        assert_eq!(reply[0], 0);
        if [app::LEFT, app::RIGHT, 0xd18b35].contains(&reply[1]) {
            return reply[1];
        }
        assert!(
            !time::reached(time::now().unwrap(), deadline),
            "app did not present: {reply:?}"
        );
    }
}
pub(super) fn sibling_saved(matrix: &mut RootControl) -> bool {
    visible(matrix, false) == 0xd18b35
}
pub(super) fn restored(matrix: &mut RootControl) -> bool {
    visible(matrix, true);
    visible(matrix, false);
    let states = [
        app(matrix, true, wire::HEALTH)[4],
        app(matrix, false, wire::HEALTH)[4],
    ];
    if states == [1, 1] {
        pixel(matrix, 140, 190, 0xd18b35);
        pixel(matrix, 510, 190, 0xd18b35);
        cathedral_user_runtime::write(
            b"Cathedral matrix desktop: persisted colors restored after reboot\n",
        )
        .unwrap();
        true
    } else {
        assert_eq!(states, [0, 0]);
        false
    }
}
pub(super) fn exercise(matrix: &mut RootControl, pressure: bool) {
    assert_eq!(app(matrix, true, 8)[0], 0);
    pixel(matrix, 140, 190, 0xd18b35);
    pixel(matrix, 510, 190, app::RIGHT);
    // Revoke a live app's inherited catalog while keeping its display grant.
    owner(matrix, [wire::STORAGE_REVOKE, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(app(matrix, true, 8)[0], abi::DENIED);
    pixel(matrix, 140, 190, 0xd18b35);
    assert_eq!(app(matrix, false, 8)[0], 0);
    pixel(matrix, 510, 190, 0xd18b35);
    owner(matrix, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(app(matrix, true, 8)[0], abi::DENIED);
    owner(
        matrix,
        wire::route([wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
    );
    assert_eq!(app(matrix, true, 8)[0], abi::DENIED); // Old leaf binding stays stale.
    assert_eq!(app(matrix, true, 7)[0], 0);
    for death in [false, true] {
        // Neither staged catalog data nor staged pixels may survive termination.
        owner(matrix, [wire::POINTER_SOURCE, 170, 220, 1, 0, 0, 0, 0]);
        owner(matrix, [wire::POINTER_SOURCE, 170, 220, 0, 0, 0, 0, 0]);
        assert_eq!(app(matrix, true, 9)[0], 0);
        pixel(matrix, 140, 190, 0xd18b35);
        if death {
            // Exit without acknowledging an owner request. Both routing hops must
            // finish their pending call and leave the outside sibling responsive.
            assert_eq!(app(matrix, true, 10)[0], abi::PEER_CLOSED);
            pixel(matrix, 140, 190, BACKGROUND);
        }
        owner(matrix, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
        pixel(matrix, 140, 190, BACKGROUND);
        pixel(matrix, 316, 292, BACKGROUND);
        pixel(matrix, 510, 190, 0xd18b35);
        assert_eq!(app(matrix, false, wire::HEALTH)[0], 0);
        inner(matrix, pressure);
        pixel(matrix, 140, 190, 0xd18b35); // Committed color, not staged toggle.
        bitmap(matrix);
        for state in [input::PRESS, input::RELEASE] {
            owner(
                matrix,
                [
                    wire::KEY_SOURCE,
                    input::ENTER as u64,
                    state as u64,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
            );
        }
        assert_eq!(app(matrix, true, wire::HEALTH)[4], 1); // Replacement did not inherit focus.
    }
    // Restore the two distinct defaults before the physical-input host check.
    assert_eq!(app(matrix, true, 8)[0], 0);
    assert_eq!(app(matrix, false, 8)[0], 0);
    pixel(matrix, 140, 190, app::LEFT);
    pixel(matrix, 510, 190, app::RIGHT);
    cathedral_user_runtime::write(b"Cathedral matrix desktop: private persisted colors, revoked writes, staged-work death and sibling recovery passed\n").unwrap();
}
