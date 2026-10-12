//! Distribution layout and recovery policy over the platform matrix-owned tree.
mod admission;
mod app;
mod probes;
use super::*;
pub use app::run as app;
use cathedral_contracts::input;
use cathedral_user_runtime::{ipc, task::Launch};
const BACKGROUND: u64 = 0x101827;
fn owner(pair: Pair, words: wire::Packet) {
    assert_eq!(call(pair, words), [0; 8]);
}
fn inner(pair: Pair, pressure: bool) {
    owner(
        pair,
        [
            wire::RUN,
            0,
            0,
            0,
            6,
            if pressure { 2 } else { 1 },
            if pressure { 7 } else { 3 },
            if pressure { 20000 } else { 0 },
        ],
    );
    owner(
        pair,
        wire::route([wire::RUN, 0, 1, 0, 0, 0, 0, 0], 0).unwrap(),
    );
    if !pressure {
        assert_eq!(
            call(
                pair,
                wire::route([wire::RUN, 1, 1, 0, 0, 0, 0, 0], 0).unwrap()
            ),
            wire::error(abi::NO_MEMORY)
        );
    }
    if pressure {
        owner(
            pair,
            wire::route([wire::RUN, 1, 2, 0, 0, 0, 0, 0], 0).unwrap(),
        );
    }
    owner(pair, [wire::SURFACE_LIMIT, 0, 8, 0, 0, 0, 0, 0]);
    owner(
        pair,
        [wire::PLACE, 0, 110, 140, 280, 220, wire::KEY_MASK, 0],
    );
    owner(
        pair,
        wire::route([wire::SURFACE_LIMIT, 0, 4, 0, 0, 0, 0, 0], 0).unwrap(),
    );
    owner(
        pair,
        wire::route(
            [wire::PLACE, 0, 24, 40, 200, 130, 1 << (input::ENTER - 1), 0],
            0,
        )
        .unwrap(),
    );
}
fn pixel(pair: Pair, x: u64, y: u64, expected: u64) {
    let deadline = time::after(300).unwrap();
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
pub fn run() -> u64 {
    let session = admission::start();
    let matrix = session.matrix;
    let pressure = session.pressure;
    owner(
        matrix,
        [
            wire::BIND,
            session.extent[0],
            session.extent[1],
            wire::KEY_MASK,
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
    inner(matrix, pressure);
    owner(matrix, [wire::RUN, 1, 1, 1, 0, 0, 0, 0]);
    owner(matrix, [wire::SURFACE_LIMIT, 1, 8, 0, 0, 0, 0, 0]);
    owner(
        matrix,
        [wire::PLACE, 1, 500, 180, 220, 180, wire::KEY_MASK, 0],
    );
    pixel(matrix, 140, 190, app::LEFT);
    pixel(matrix, 510, 190, app::RIGHT);
    pixel(matrix, 334, 310, BACKGROUND); // Oversized leaf draw was clipped twice.
    for _ in 0..3 {
        owner(matrix, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
        pixel(matrix, 140, 190, BACKGROUND);
        pixel(matrix, 510, 190, app::RIGHT);
        inner(matrix, pressure);
        pixel(matrix, 140, 190, app::LEFT);
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
    probes::exercise(matrix);
    if pressure {
        cathedral_user_runtime::write(b"Cathedral matrix desktop: capped spinner active\n")
            .unwrap();
    }
    session.start_input();
    cathedral_user_runtime::write(b"Cathedral matrix desktop: ready\n").unwrap();
    loop {
        matrix.1.receive(&mut [0; 64]).unwrap();
    }
}
