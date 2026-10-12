//! Admit device adapters, stage code authority and hand a funded pool to the root.
use super::*;
use cathedral_contracts::{composition as c, display, pointer};
use cathedral_user_runtime::task::{self, Executable, Port};
pub(super) struct Session {
    pub matrix: Pair,
    pub pressure: bool,
    pub extent: [u64; 2],
    source: Pair,
}
fn display_call(
    pair: Pair,
    task: cathedral_user_runtime::task::Child,
    words: [u64; 6],
) -> [u64; 6] {
    pair.0.send(&display::encode(words)).unwrap();
    let mut bytes = [0; 64];
    let len = pair
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap_or_else(|error| {
            panic!(
                "display request {words:?}: {error:?}; outcome {:?}",
                task.wait_until(time::now().unwrap())
            )
        });
    display::decode(&bytes[..len]).unwrap()
}
pub(super) fn start() -> Session {
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
        display_call(
            display,
            display_task,
            [c::BACKGROUND, BACKGROUND, 0, 0, 0, 0]
        )[0],
        0
    );
    assert_eq!(
        display_call(display, display_task, [pointer::ENABLE, 1, 0, 0, 0, 0])[0],
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
    let ready = matrix
        .0
        .send(&wire::encode([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]))
        .and_then(|()| {
            matrix
                .1
                .receive_until(&mut [0; 64], time::after(300).unwrap())
        });
    if let Err(error) = ready {
        panic!(
            "root matrix startup {error:?}; outcome {:?}",
            _root.wait_until(time::now().unwrap())
        );
    }
    Session {
        matrix,
        pressure,
        extent: [info[1], info[2]],
        source: (source_send, source_recv),
    }
}
impl Session {
    pub fn start_input(self) {
        for op in [wire::SPAWN, wire::READ_PIXEL, wire::BIND] {
            assert_eq!(
                call(self.source, [op, 0, 0, 0, 0, 0, 0, 0]),
                wire::error(abi::DENIED)
            );
        }
        let _input = Launch::at(1)
            .unwrap()
            .spawn_with(
                self.extent[0] | (self.extent[1] << 32),
                &[self.source.0, self.source.1],
            )
            .unwrap();
        Port::at(1)
            .unwrap()
            .connect()
            .unwrap()
            .1
            .receive(&mut [0; 64])
            .unwrap();
    }
}
