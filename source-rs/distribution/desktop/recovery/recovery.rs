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

pub(crate) fn run(environment: Environment) -> u64 {
    let connections =
        cathedral_user_runtime::matrix::OptionalConnections::load(&environment).unwrap();
    let parent = connections.interaction;
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let mut epoch = 0;
    let mut pressed = false;
    let mut retry = false;
    loop {
        let snapshot = call(parent, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
        let view = wire::poll_view(snapshot);
        if view[0] == 0 {
            if epoch != view[1] {
                crate::screen::paint(parent, view);
                epoch = view[1];
                pressed = false;
                cathedral_user_runtime::write(b"Cathedral desktop: sign-in recovery visible\n")
                    .unwrap();
            }
            let event = wire::poll_event(snapshot);
            match event[2] {
                wire::CANCEL => pressed = false,
                wire::POINTER => {
                    let down = event[5] & 1 != 0;
                    retry |= down && !pressed && crate::screen::button(view, event[3], event[4]);
                    pressed = down;
                }
                wire::KEY => {
                    retry |= event[3] == input::ENTER as u64 && event[4] == input::PRESS as u64
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
