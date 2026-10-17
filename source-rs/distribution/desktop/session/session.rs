// Distribution layout and recovery policy over the platform matrix-owned tree.
use super::*;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::{ipc, task::Launch};
const APP_PAGES: u64 = if cfg!(feature = "notes") { 32 } else { 2 };
const APP_READERS: u64 = if cfg!(feature = "notes") { 4 } else { 0 };
const MATRIX_PAGES: u64 = if cfg!(feature = "notes") { 40 } else { 4 };
const MATRIX_READERS: u64 = if cfg!(feature = "notes") { 40 } else { 4 };
/// Notes draws a base, a text bitmap and three note-bar rectangles; the color
/// probe draws a base and an accent plus its quota probes.
const APP_SURFACES: u64 = if cfg!(feature = "notes") { 6 } else { 4 };
const INNER_SURFACES: u64 = if cfg!(feature = "notes") { 10 } else { 8 };
const SIBLING_SURFACES: u64 = if cfg!(feature = "notes") { 6 } else { 8 };
const KEYS: u64 = if cfg!(feature = "notes") {
    wire::VALID_KEY_MASK
} else {
    wire::KEY_MASK
};
const BACKGROUND: u64 = 0x101827;
fn owner(pair: Pair, words: wire::Packet) {
    assert_eq!(call(pair, words), [0; 8], "owner request {words:?}");
}
fn launch_inner(pair: Pair, pressure: bool) {
    owner(
        pair,
        [wire::RUN_PAGES, 0, MATRIX_PAGES, MATRIX_READERS, 0, 0, 0, 0],
    );
    owner(
        pair,
        [
            wire::RUN,
            0,
            0,
            0,
            10,
            2,
            if pressure { 7 } else { 3 },
            if pressure { 20000 } else { 0 },
        ],
    );
}
fn inner(pair: Pair, pressure: bool) {
    launch_inner(pair, pressure);
    owner(
        pair,
        wire::route([wire::RUN_PAGES, 0, APP_PAGES, APP_READERS, 0, 0, 0, 0], 0).unwrap(),
    );
    owner(
        pair,
        wire::route([wire::RUN, 0, 1, 0, 0, 0, 0, 0], 0).unwrap(),
    );
    if pressure {
        owner(
            pair,
            wire::route([wire::RUN, 1, 2, 0, 0, 0, 0, 0], 0).unwrap(),
        );
    }
    owner(pair, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    owner(
        pair,
        wire::route([wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
    );
    owner(
        pair,
        [wire::SURFACE_LIMIT, 0, INNER_SURFACES, 0, 0, 0, 0, 0],
    );
    owner(pair, [wire::PLACE, 0, 110, 140, 280, 220, KEYS, 0]);
    owner(
        pair,
        wire::route([wire::SURFACE_LIMIT, 0, APP_SURFACES, 0, 0, 0, 0, 0], 0).unwrap(),
    );
    #[cfg(feature = "database-probes")]
    stalls_peer::start(pair, pressure);
    owner(
        pair,
        wire::route(
            [
                wire::PLACE,
                0,
                24,
                40,
                200,
                130,
                if cfg!(feature = "notes") {
                    KEYS
                } else {
                    1 << (input::ENTER - 1)
                },
                0,
            ],
            0,
        )
        .unwrap(),
    );
}
fn pixel(pair: Pair, x: u64, y: u64, expected: u64) {
    // Cold surface creation crosses multiple IPC hops under the 20% pressure cap.
    // This is a bounded correctness wait; the host measures input latency separately.
    let deadline = time::after(1000).unwrap();
    loop {
        let p = call(pair, [wire::READ_PIXEL, x, y, 0, 0, 0, 0, 0]);
        if p[0] == 0 && p[1] == expected {
            return;
        }
        assert!(
            !time::reached(time::now().unwrap(), deadline),
            "pixel {x},{y}: {p:?}"
        );
    }
}
fn bitmap(matrix: Pair) {
    pixel(matrix, 316, 292, 0xffcc44);
    pixel(matrix, 316, 299, 0x7755dd);
    pixel(matrix, 334, 292, BACKGROUND);
    pixel(matrix, 682, 292, 0xffcc44);
}
fn setup(probes: bool) -> admission::Session {
    let mut session = admission::start();
    let matrix = session.matrix;
    let pressure = session.pressure;
    owner(
        matrix,
        [
            wire::BIND,
            session.extent[0],
            session.extent[1],
            KEYS,
            0,
            0,
            0,
            0,
        ],
    );
    // A bad delegated executable fails after reservation; rollback restores funding.
    assert_eq!(
        call(matrix, [wire::RUN, 0, 0, 0, 6, 1, 64, 0]),
        wire::error(abi::BAD_HANDLE)
    );
    owner(
        matrix,
        [wire::RUN_PAGES, 1, APP_PAGES, APP_READERS, 0, 0, 0, 0],
    );
    owner(matrix, [wire::RUN, 1, 1, 1, 0, 0, 0, 0]);
    owner(matrix, [wire::STORAGE_GRANT, 1, 1, 0, 0, 0, 0, 0]);
    owner(
        matrix,
        [wire::SURFACE_LIMIT, 1, SIBLING_SURFACES, 0, 0, 0, 0, 0],
    );
    owner(matrix, [wire::PLACE, 1, 500, 180, 220, 180, KEYS, 0]);
    if probes && !recovery::sibling_saved(matrix) {
        funding::exercise(matrix, pressure);
    }
    inner(matrix, pressure);
    if probes {
        probes::startup(matrix, pressure);
        if !recovery::sibling_saved(matrix) {
            stalls::exercise(&mut session);
        }
    }
    if pressure {
        cathedral_user_runtime::write(b"Cathedral matrix desktop: capped spinner active\n")
            .unwrap();
    }
    session
}
pub fn run() -> u64 {
    if cfg!(feature = "shell") {
        #[cfg(feature = "shell")]
        return shell::run();
    }
    let mut session = setup(cfg!(feature = "desktop-probes"));
    if cfg!(feature = "desktop-probes") && !recovery::sibling_saved(session.matrix) {
        session = providers::exercise(session);
    }
    #[cfg(feature = "database-probes")]
    database_recovery::exercise(&mut session);
    session.start_input();
    cathedral_user_runtime::write(b"Cathedral matrix desktop: ready\n").unwrap();
    loop {
        match session
            .matrix
            .1
            .receive_until(&mut [0; 64], time::after(100).unwrap())
        {
            Ok(_) => (),
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(error) => panic!("desktop owner: {error:?}"),
        }
        persistence::supervise(&mut session);
    }
}
