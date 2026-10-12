//! Distribution layout and recovery policy over the platform matrix-owned tree.
mod app;
mod probes;
use super::*;
pub use app::run as app;
use cathedral_contracts::{composition as c, display, input, pointer};
use cathedral_user_runtime::{
    ipc,
    task::{self, Executable, Launch, Port},
};
const BACKGROUND: u64 = 0x101827;
fn display_call(pair: Pair, words: [u64; 6]) -> [u64; 6] {
    pair.0.send(&display::encode(words)).unwrap();
    let mut bytes = [0; 64];
    let len = pair
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    display::decode(&bytes[..len]).unwrap()
}
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
    owner(
        pair,
        [wire::PLACE, 0, 110, 140, 280, 220, wire::KEY_MASK, 0],
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
    assert_eq!(
        Launch::at(0).unwrap().executable().unwrap_err(),
        Error(abi::DENIED as i64)
    );
    let (display_send, display_input) = ipc::channel().unwrap();
    let (display_output, display_recv) = ipc::channel().unwrap();
    let display_task = Launch::at(0)
        .unwrap()
        .spawn_with(0, &[display_input, display_output])
        .unwrap();
    let display = (display_send, display_recv);
    display
        .0
        .send(&display::encode([display::INFO, 0, 0, 0, 0, 0]))
        .unwrap_or_else(|error| {
            panic!(
                "display entry {error:?}; outcome {:?}",
                display_task.wait_until(time::now().unwrap())
            )
        });
    let mut info_bytes = [0; 64];
    let info_len = display
        .1
        .receive_until(&mut info_bytes, time::after(300).unwrap())
        .unwrap_or_else(|error| {
            panic!(
                "display startup {error:?}; outcome {:?}",
                display_task.wait_until(time::now().unwrap())
            )
        });
    let info = display::decode(&info_bytes[..info_len]).unwrap();
    assert_eq!(info[0], 0);
    assert_eq!(
        display_call(display, [c::BACKGROUND, BACKGROUND, 0, 0, 0, 0])[0],
        0
    );
    assert_eq!(
        display_call(display, [pointer::ENABLE, 1, 0, 0, 0, 0])[0],
        0
    );
    for (x, y, text) in [
        (80, 60, b"MATRIX SESSION".as_slice()),
        (134, 140, b"NESTED APP"),
        (500, 140, b"SIBLING APP"),
        (80, 400, b"CLICK THEN ENTER"),
    ] {
        display
            .0
            .send(&display::encode_text(x, y, 2, 0xffffff, text).unwrap())
            .unwrap();
        display.1.receive(&mut [0; 64]).unwrap();
    }
    let (source_send, source_input) = ipc::channel().unwrap();
    let (source_output, source_recv) = ipc::channel().unwrap();
    let capacity = task::available_slots().unwrap();
    let pressure = capacity == 4;
    assert_eq!(capacity, if pressure { 4 } else { 3 });
    let root = Launch::at(2).unwrap();
    root.assign_slots(capacity).unwrap();
    for index in 0..if pressure { 3 } else { 2 } {
        root.allow_executable(Executable::at(index).unwrap())
            .unwrap();
    }
    let _root = root
        .spawn_with(0, &[display.0, display.1, source_input, source_output])
        .unwrap();
    assert_eq!(task::available_slots().unwrap(), 0);
    assert!(Launch::at(3).is_err());
    let matrix = Port::at(2).unwrap().connect().unwrap();
    owner(
        matrix,
        [wire::BIND, info[1], info[2], wire::KEY_MASK, 0, 0, 0, 0],
    );
    // A bad delegated executable fails after reservation; rollback restores funding.
    assert_eq!(
        call(matrix, [wire::RUN, 0, 0, 0, 6, 1, 64, 0]),
        wire::error(abi::BAD_HANDLE)
    );
    inner(matrix, pressure);
    owner(matrix, [wire::RUN, 1, 1, 1, 0, 0, 0, 0]);
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
    for op in [wire::SPAWN, wire::READ_PIXEL, wire::BIND] {
        assert_eq!(
            call((source_send, source_recv), [op, 0, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::DENIED)
        );
    }
    let _input = Launch::at(1)
        .unwrap()
        .spawn_with(info[1] | (info[2] << 32), &[source_send, source_recv])
        .unwrap();
    Port::at(1)
        .unwrap()
        .connect()
        .unwrap()
        .1
        .receive(&mut [0; 64])
        .unwrap();
    cathedral_user_runtime::write(b"Cathedral matrix desktop: ready\n").unwrap();
    loop {
        matrix.1.receive(&mut [0; 64]).unwrap();
    }
}
