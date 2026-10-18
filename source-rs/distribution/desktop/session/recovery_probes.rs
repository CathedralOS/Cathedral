//! Copied-image fixture only. Once Notes has a visible draft, exercise real
//! owner authority over the user matrix, leaving its app alive for QMP checks.
use super::*;

pub(super) fn apply(matrix: Pair) -> bool {
    let pixel = call(matrix, [wire::READ_PIXEL, 237, 34, 0, 0, 0, 0, 0]);
    if pixel[..2] != [0, 0x9b7028] {
        return false;
    }
    let operation = if cfg!(feature = "launcher-stop-probe") {
        wire::STOP
    } else {
        wire::STORAGE_REVOKE
    };
    owner(
        matrix,
        wire::route([operation, 0, 0, 0, 0, 0, 0, 0], 1).unwrap(),
    );
    cathedral_user_runtime::write(b"Cathedral desktop: launcher owner probe applied\n").unwrap();
    true
}
