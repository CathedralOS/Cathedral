//! Admit device adapters, stage code authority and hand a funded pool to the root.
use super::*;
use cathedral_platform_contracts::display;
use cathedral_platform_contracts::display::composition as c;
use cathedral_platform_contracts::display::pointer;
use cathedral_user_runtime::task::{self, Executable, Port};
pub(super) struct Session {
    #[cfg(feature = "database-probes")]
    pub(super) database_fresh: bool,
    pub matrix: Pair,
    #[cfg(feature = "shell")]
    pub(super) notes_open: bool,
    pub pressure: bool,
    pub extent: [u64; 2],
    pub(super) source: Pair,
    pub(super) storage: task::Child,
    pub(super) storage_control: Pair,
    pub(super) storage_watch: persistence::Watchdog,
    pub(super) display_task: task::Child,
    pub(super) root: task::Child,
    pub(super) input: Option<task::Child>,
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
    let (storage, storage_control) = {
        let storage = Launch::at(3).unwrap().spawn(0).unwrap();
        let control = Port::at(3).unwrap().connect().unwrap();
        control
            .0
            .send(cathedral_platform_contracts::storage::HEALTH)
            .unwrap();
        assert_eq!(control.1.receive(&mut [0; 64]).unwrap(), 5);
        (storage, control)
    };
    #[cfg(feature = "notes-probes")]
    {
        // Quiet post-commit gate: the ordinary watchdog must discover the stall.
        // On cold reboot this stays armed, but the restore-only proof performs no save.
        storage_control
            .0
            .send(&[
                0xf5,
                2,
                cathedral_platform_contracts::storage::PUBLISH as u8,
                0,
            ])
            .unwrap();
        let mut bytes = [0; 64];
        let n = storage_control.1.receive(&mut bytes).unwrap();
        assert_eq!(&bytes[..n], b"armed");
    }
    #[cfg(feature = "database-probes")]
    let database_fresh = database_recovery::arm_startup(storage_control);
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
    let labels: &[(u64, u64, &[u8])] = if cfg!(feature = "shell") {
        &[
            (40, 50, b"CATHEDRAL".as_slice()),
            (40, 110, b"PROGRAMS"),
            (330, 110, b"WORKSPACE"),
            (40, 330, b"CLICK A PROGRAM"),
            (40, 362, b"THEN CLICK APP"),
            (40, 420, b"NOTES: F1 SAVE"),
            (40, 452, b"F2 LOAD F3 COPY"),
            (40, 484, b"F4 NEXT NOTE"),
            (360, 420, b"COLORS: ENTER"),
            (360, 452, b"CHANGES COLOR"),
            (360, 484, b"NO REBOOT NEEDED"),
        ]
    } else if cfg!(feature = "notes") {
        &[
            (80, 60, b"PRIVATE NOTES"),
            (134, 140, b"NESTED NOTE"),
            (500, 140, b"SIBLING NOTE"),
            (80, 400, b"F1 SAVE OR CHECK"),
            (80, 432, b"F2 RELOAD"),
            (80, 464, b"F3 COPY F4 NEXT"),
            (500, 400, b"GREEN SAVED"),
            (500, 432, b"AMBER DRAFT"),
            (500, 464, b"RED SAVE ERROR"),
        ]
    } else {
        &[
            (80, 60, b"MATRIX SESSION".as_slice()),
            (134, 140, b"NESTED APP"),
            (500, 140, b"SIBLING APP"),
            (80, 400, b"CLICK THEN ENTER"),
            (80, 440, b"RED: UNCONFIRMED"),
            (80, 472, b"ENTER TO RELOAD"),
        ]
    };
    for &(x, y, text) in labels {
        display
            .0
            .send(&display::encode_text(x, y, 2, 0xffffff, text).unwrap())
            .unwrap();
        display.1.receive(&mut [0; 64]).unwrap();
    }
    let (source_send, source_input) = ipc::channel().unwrap();
    let (source_output, source_recv) = ipc::channel().unwrap();
    let capacity = task::available_slots().unwrap();
    let pressure = !cfg!(feature = "shell") && Executable::at(2).is_ok();
    assert_eq!(capacity, 4);
    let root = Launch::at(2).unwrap();
    // Descendant reservations return to the surviving distribution owner on root
    // death. Re-fund the replacement explicitly; restarting never mints budgets.
    cathedral_user_runtime::resources::assign(
        root,
        cathedral_user_runtime::resources::Limits {
            private_pages: if cfg!(feature = "notes") { 128 } else { 16 },
            reader_pages: if cfg!(feature = "notes") { 128 } else { 8 },
            channels: 20,
        },
    )
    .unwrap();
    root.assign_slots(capacity).unwrap();
    root.assign_memory((capacity + 1) * abi::MAX_ADMISSION_FRAMES as u64)
        .unwrap();
    for index in 0..if pressure || cfg!(feature = "shell") {
        3
    } else {
        2
    } {
        root.allow_executable(Executable::at(index).unwrap())
            .unwrap();
    }
    let _root = root
        .spawn_with(0, &[display.0, display.1, source_input, source_output])
        .unwrap();
    crate::ROOT_TASK.store(_root.raw(), core::sync::atomic::Ordering::Relaxed);
    assert_eq!(task::available_slots().unwrap(), 0);
    assert!(Launch::at(3).is_ok());
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
        #[cfg(feature = "database-probes")]
        database_fresh,
        matrix,
        #[cfg(feature = "shell")]
        notes_open: false,
        pressure,
        extent: [info[1], info[2]],
        source: (source_send, source_recv),
        storage,
        storage_control,
        storage_watch: persistence::Watchdog::default(),
        display_task,
        root: _root,
        input: None,
    }
}
impl Session {
    pub fn start_input(&mut self) {
        assert!(self.input.is_none());
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
        self.input = Some(_input);
        Port::at(1)
            .unwrap()
            .connect()
            .unwrap()
            .1
            .receive(&mut [0; 64])
            .unwrap();
    }
    pub(super) fn stop(self) {
        for child in [
            self.input,
            Some(self.root),
            Some(self.display_task),
            Some(self.storage),
        ]
        .into_iter()
        .flatten()
        {
            match child.wait_until(time::now().unwrap()) {
                Ok(_) => (),
                Err(Error(code)) if code == abi::BAD_HANDLE as i64 => (), // Probe already collected it.
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => {
                    child.cancel().unwrap();
                    child.wait().unwrap();
                }
                Err(error) => panic!("desktop shutdown {error:?}"),
            }
        }
        if self.input.is_none() {
            self.source.0.close().unwrap();
            self.source.1.close().unwrap();
        }
        // Descendant creator death returns dynamic reservations to this owner.
        let mut index = 0;
        while let Ok(launch) = Launch::at(index) {
            match launch.release() {
                Ok(()) => (),
                Err(Error(code)) if code == abi::DENIED as i64 || code == abi::BUSY as i64 => {
                    index += 1
                }
                Err(error) => panic!("desktop reservation release {error:?}"),
            }
        }
    }
}
