//! Distribution presentation phases: the picker yields the entire display to
//! the user. The boot matrix remains the authority and composition parent.
use super::*;

/// Also restores selection if the running user is lost; there is no logout UI.
pub(in crate::session) fn select_user(matrix: Pair, [width, height]: [u64; 2]) -> Result<(), u64> {
    request(matrix, [wire::SURFACE_LIMIT, 0, 5, 0, 0, 0, 0, 0])?;
    request(
        matrix,
        [wire::PLACE, 0, 0, 0, width, height, wire::KEY_MASK, 0],
    )
}
