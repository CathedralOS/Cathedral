// Resolve the login programs, grant the user ceiling, then ask that matrix
// to configure itself. The shell never interprets user-internal object order.
use super::*;
fn request(matrix: Pair, words: wire::Packet) -> Result<(), u64> {
    match call(matrix, words)[0] {
        0 => Ok(()),
        code => Err(code),
    }
}
fn resolve(matrix: Pair, path: &[u8]) -> Result<u64, u64> {
    let mut bytes = [0; 64];
    bytes[..8].copy_from_slice(&wire::RESOLVE_PATH.to_le_bytes());
    bytes[8..8 + path.len()].copy_from_slice(path);
    let reply = call(matrix, wire::decode(&bytes).unwrap());
    if reply[0] == 0 {
        Ok(reply[1])
    } else {
        Err(reply[0])
    }
}
/// Three 12-page tiles and one replacement fit in the chooser allowance.
const CHOOSER_PAGES: u64 = 64;

/// Root slot 0: the chooser, shown the folder of users and nothing else.
fn chooser(login: &mut Login, matrix: Pair, extent: [u64; 2], failed: bool) -> Result<(), u64> {
    let chooser = resolve(matrix, b"Login")?;
    let users = resolve(matrix, b"Users")?;
    // Keep an existing recovery screen visible through the path reads.
    stop(matrix, 0)?;
    *login = Login::Stopped;
    request(
        matrix,
        [
            wire::RUN_PAGES,
            0,
            CHOOSER_PAGES,
            cathedral_desktop_matrix_contracts::matrix::CHOOSER_READERS,
            0,
            0,
            0,
            0,
        ],
    )?;
    request(
        matrix,
        [
            wire::RUN_OBJECT,
            0,
            chooser,
            users,
            cathedral_desktop_matrix_contracts::matrix::launch_argument(
                if failed { 2 } else { 1 },
                chooser,
            ),
            2,
            0,
            0,
        ],
    )?;
    select_user(matrix, extent)
}

/// Root slot 1: a user matrix with a launcher and a matrix-capable app slot.
/// Only eleven surfaces are lent here; the root matrix recognizes the picker
/// it is running over `Users`, retires it before admitting the chosen user and
/// lends that user the full allowance itself, so `enter` finds both done.
pub(crate) fn workspace(width: u64, height: u64) -> [wire::Packet; 3] {
    [
        [wire::LAUNCH_IMAGES, 1, 1, 0, 0, 0, 0, 0],
        [wire::LAUNCH_POLICY, 1, 960, 3584, 168, 24, 0, 11],
        [wire::LAUNCH_PLACE, 1, 0, 0, width, height, KEYS, 0],
    ]
}

/// Retire the picker and request the user matrix's own saved startup policy.
pub(super) fn enter(matrix: Pair) -> Result<(), u64> {
    // Selection admitted a sibling, not a descendant of the picker. Reclaim
    // the picker process, surfaces, input and realm binding before any slow
    // storage reads needed to start the user's launcher.
    stop(matrix, 0)?;
    request(
        matrix,
        [
            wire::SURFACE_LIMIT,
            1,
            wire::MAX_SURFACES as u64,
            0,
            0,
            0,
            0,
            0,
        ],
    )?;
    cathedral_user_runtime::write(b"Cathedral desktop: picker stopped; loading user launcher\n")
        .unwrap();
    request(
        matrix,
        wire::route(
            [
                wire::LAUNCHER_FALLBACK,
                1,
                crate::recovery_protocol::LAUNCHER,
                0,
                0,
                0,
                0,
                0,
            ],
            1,
        )
        .unwrap(),
    )?;
    let reply = call(
        matrix,
        wire::route([wire::CONFIGURE, 0, 0, 0, 0, 0, 0, 0], 1).unwrap(),
    );
    if reply[0] != 0 {
        return Err(reply[0]);
    }
    request(matrix, [wire::SAVE_RECOVERY_GRANT, 1, 1, 0, 0, 0, 0, 0])
}

/// Collection may win the race with startup failure handling.
pub(super) fn stop(matrix: Pair, slot: u64) -> Result<(), u64> {
    match call(matrix, [wire::STOP, slot, 0, 0, 0, 0, 0, 0])[0] {
        0 | abi::PEER_CLOSED => Ok(()),
        code => Err(code),
    }
}

/// Observe only the direct user matrix in slot 1.
/// Losing a storage binding is not process death. Keep the last known realm
/// for recovery until the child's control link confirms it has gone away;
/// busy or timed-out probes never turn a live session into a new picker.
pub(super) fn realm_of(matrix: Pair, previous: u64) -> u64 {
    let binding = [wire::STORAGE_BINDING, 1, 0, 0, 0, 0, 0, 0];
    let health = wire::route([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0], 1).unwrap();
    let reply = call(matrix, binding);
    if reply[0] == 0 {
        reply[2]
    } else if previous != 0 && call(matrix, health)[0] != abi::PEER_CLOSED {
        previous
    } else {
        0
    }
}

/// Restore the direct child's realm after provider replacement, then ask the
/// user matrix to restore its own remembered child bindings.
pub(crate) fn regrant(session: &mut admission::Session) {
    let matrix = session.matrix;
    if session.user == 0 {
        if session.login != Login::Picker {
            return;
        }
        let users = match resolve(matrix, b"Users") {
            Ok(users) => users,
            Err(code) => {
                session.login.recover(matrix, session.extent, code);
                return;
            }
        };
        owner(matrix, [wire::STORAGE_GRANT, 0, 0, users, 0, 0, 0, 0]);
        return;
    }
    // The transient picker has been collected; do not restore its authority.
    owner(
        matrix,
        [wire::STORAGE_GRANT, 1, 0, session.user, 0, 0, 0, 0],
    );
    owner(
        matrix,
        wire::route([wire::RESTORE_STORAGE, 0, 0, 0, 0, 0, 0, 0], 1).unwrap(),
    );
}
