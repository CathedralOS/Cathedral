//! Present the failure, observe local input, report a user-requested retry to
//! desktop/session/shell/login.rs over the owner control link. HEALTH word 1
//! consumes the pending request; silence or process death never requests retry.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{input, matrix as wire};
use cathedral_user_runtime::{
    Error,
    ipc::{self, Handle},
    task::Environment,
    time,
};
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

pub(crate) fn run(environment: Environment, mut failed: bool) -> u64 {
    let connections =
        cathedral_user_runtime::matrix::OptionalConnections::load(&environment).unwrap();
    let parent = connections.interaction;
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let mut epoch = 0;
    let mut pressed = false;
    let mut retry = false;
    let mut painted = None;
    let mut surface = 0;
    loop {
        let snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
        let view = wire::poll_view(snapshot);
        if view[0] == 0 {
            if epoch != view[1] {
                let reply = call(parent, [wire::SURFACE_CREATE, view[1], 0, 0, 0, 0, 0, 0]);
                assert_eq!(reply[0], 0);
                surface = reply[1];
                epoch = view[1];
                painted = None;
            }
            if painted != Some(failed) {
                crate::screen::paint(parent, view, surface, failed);
                painted = Some(failed);
                cathedral_user_runtime::write(if failed {
                    b"Cathedral desktop: sign-in recovery visible\n"
                } else {
                    b"Cathedral desktop: storage starting visible\n"
                })
                .unwrap();
            }
            let event = wire::poll_event(snapshot);
            match event[2] {
                // Cancellation clears the old lease, not the physical button.
                // Keep the last observed button state across presentation
                // phases: a held press cannot become a new retry, and an
                // already released button needs only one fresh click.
                wire::CANCEL => (),
                wire::POINTER => {
                    let down = event[5] & 1 != 0;
                    retry |= failed
                        && down
                        && !pressed
                        && crate::screen::button(view, event[3], event[4]);
                    pressed = down;
                }
                wire::KEY => {
                    retry |=
                        failed && event[3] == input::ENTER as u64 && event[4] == input::PRESS as u64
                }
                _ => (),
            }
        }
        if snapshot[5] & wire::MORE != 0 {
            continue;
        }
        match ipc::wait_many(&[connections.wake, control.0], time::after(100).unwrap()) {
            Ok(0) => {
                connections.wake.receive(&mut [0; 64]).unwrap();
            }
            Ok(_) => {
                let mut bytes = [0; 64];
                let n = control.0.receive(&mut bytes).unwrap();
                let request = wire::decode(&bytes[..n]).unwrap();
                let reply = match request[0] {
                    crate::protocol::STATUS if request[1] <= crate::protocol::FAILED => {
                        failed = request[1] != crate::protocol::STARTING;
                        retry = false;
                        // Owner revokes queued input before STATUS. Preserve
                        // button state until a new local pointer report arrives.
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
