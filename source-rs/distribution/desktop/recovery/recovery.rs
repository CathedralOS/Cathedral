//! Present the failure, observe local input, report a user-requested retry to
//! desktop/session/shell/login.rs over the owner control link, or to the local
//! matrix's explicit launcher-retry grant. HEALTH consumes a sign-in request;
//! silence or process death never requests retry.
use cathedral_desktop_matrix_contracts::matrix as wire;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::{Error, ipc::Handle, task::Environment, time};
pub(crate) type Pair = (Handle, Handle);

pub(crate) fn call(pair: Pair, packet: wire::Packet) -> wire::Packet {
    pair.0.send(&wire::encode(packet)).unwrap();
    let mut bytes = [0; 64];
    let n = pair
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    wire::decode(&bytes[..n]).unwrap()
}

pub(crate) fn run(environment: Environment, argument: u64) -> u64 {
    let launcher = argument == crate::protocol::LAUNCHER;
    let mut failed = argument != crate::protocol::STARTING;
    let connections =
        cathedral_user_runtime::matrix::OptionalConnections::load(&environment).unwrap();
    let parent = connections.interaction;
    if launcher {
        assert!(connections.storage.is_none());
        // Template admission includes its own code; no other images or child
        // capacity are lent, so even that grant cannot reserve a process.
        assert!(cathedral_user_runtime::task::Executable::at(1).is_err());
        assert_eq!(cathedral_user_runtime::task::available_slots().unwrap(), 0);
        for packet in [
            [wire::MANAGED_TASK, 0, 0, 0, 0, 0, 0, 0],
            [wire::APPLY_LAYOUT, 0, 24, 192, 16, 0, 0, 0],
            [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0],
        ] {
            assert_eq!(call(parent, packet), wire::error(abi::DENIED));
        }
    }
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let mut epoch = 0;
    let mut pointers = cathedral_matrix::reception::Pointers::default();
    let mut retry = false;
    let mut painted = None;
    let mut surface = 0;
    loop {
        let snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
        let view = wire::poll_view(snapshot);
        let event = wire::poll_event(snapshot);
        let clicked = pointers.update(event).is_some_and(|b| b.left_pressed());
        if view[0] == 0 {
            if epoch != view[1] {
                let reply = call(parent, [wire::SURFACE_CREATE, view[1], 0, 0, 0, 0, 0, 0]);
                assert_eq!(reply[0], 0);
                surface = reply[1];
                epoch = view[1];
                painted = None;
            }
            if painted != Some(failed) {
                crate::screen::paint(parent, view, surface, failed, launcher);
                painted = Some(failed);
                cathedral_user_runtime::write(if launcher {
                    b"Cathedral desktop: launcher recovery visible\n"
                } else if failed {
                    b"Cathedral desktop: sign-in recovery visible\n"
                } else {
                    b"Cathedral desktop: storage starting visible\n"
                })
                .unwrap();
            }
            #[cfg(feature = "fallback-probes")]
            if launcher && event[2] == wire::KEY && event[4] == input::PRESS as u64 {
                match event[3] as u8 {
                    input::F7 => return 0,
                    input::F8 => panic!("deliberate launcher fallback failure"),
                    _ => (),
                }
            }
            match event[2] {
                wire::POINTER => {
                    retry |= failed
                        && clicked
                        && crate::screen::button(view, event[3], event[4], launcher);
                }
                wire::KEY => {
                    retry |=
                        failed && event[3] == input::ENTER as u64 && event[4] == input::PRESS as u64
                }
                _ => (),
            }
        }
        if launcher && core::mem::take(&mut retry) {
            // The acknowledgement precedes our retirement. Waiting for the
            // owner preserves its phase ordering without sending another RPC
            // through an endpoint it is about to close.
            if call(parent, [wire::RETRY_LAUNCHER, 0, 0, 0, 0, 0, 0, 0])[0] == 0 {
                await_retirement(control);
            }
        }
        if snapshot[5] & wire::MORE != 0 {
            continue;
        }
        match cathedral_user_runtime::event::wait(
            &[
                connections.wake.interest(),
                cathedral_user_runtime::event::Interest::Ipc(control.0),
            ],
            time::after(100).unwrap(),
        ) {
            Ok(0) => {
                connections.wake.take().unwrap();
            }
            Ok(_) => {
                let mut bytes = [0; 64];
                let n = control.0.receive(&mut bytes).unwrap();
                let request = wire::decode(&bytes[..n]).unwrap();
                let reply = match request[0] {
                    crate::protocol::STATUS if request[1] <= crate::protocol::FAILED => {
                        failed = request[1] != crate::protocol::STARTING;
                        retry = false;
                        // Owner revokes queued input before STATUS. The next
                        // binding/cancellation packet resets receiver holds;
                        // residual physical holds are suppressed by the router.
                        [0; 8]
                    }
                    wire::HEALTH => {
                        let requested = core::mem::take(&mut retry);
                        [0, u64::from(requested), 0, 0, 0, 0, 0, 0]
                    }
                    wire::LOSS => [0; 8],
                    _ => wire::error(abi::DENIED),
                };
                control.1.send(&wire::encode(reply)).unwrap();
            }
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(error) => panic!("recovery wait: {error:?}"),
        }
    }
}
fn await_retirement(control: Pair) -> ! {
    loop {
        let mut bytes = [0; 64];
        let n = control.0.receive(&mut bytes).unwrap();
        let reply = match wire::decode(&bytes[..n]) {
            Some([wire::HEALTH | wire::LOSS, 0, 0, 0, 0, 0, 0, 0]) => [0; 8],
            _ => wire::error(abi::DENIED),
        };
        control.1.send(&wire::encode(reply)).unwrap();
    }
}
