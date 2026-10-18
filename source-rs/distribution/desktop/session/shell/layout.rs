//! Distribution presentation phases: the picker yields the entire display to
//! the user. The boot matrix remains the authority and composition parent.
use super::*;

/// Also restores selection if the running user is lost; there is no logout UI.
pub(in crate::session) fn select_user(matrix: Pair, [width, height]: [u64; 2]) {
    owner(matrix, [wire::SURFACE_LIMIT, 0, 5, 0, 0, 0, 0, 0]);
    owner(
        matrix,
        [wire::PLACE, 0, 0, 0, width, height, wire::KEY_MASK, 0],
    );
}

pub(super) fn enter_user(matrix: Pair, [width, height]: [u64; 2]) {
    // Revocation removes both retained surfaces and input, including focus and
    // capture. Covering the picker with another surface would leave it active.
    owner(matrix, [wire::REVOKE, 0, 0, 0, 0, 0, 0, 0]);
    owner(
        matrix,
        wire::route([wire::SURFACE_LIMIT, 0, 5, 0, 0, 0, 0, 0], 1).unwrap(),
    );
    owner(
        matrix,
        wire::route(
            [wire::PLACE, 0, 24, 24, 192, height - 48, wire::KEY_MASK, 0],
            1,
        )
        .unwrap(),
    );
    owner(
        matrix,
        wire::route(
            [
                wire::LAUNCH_PLACE,
                1,
                232,
                24,
                width - 256,
                height - 48,
                KEYS,
                0,
            ],
            1,
        )
        .unwrap(),
    );
}
