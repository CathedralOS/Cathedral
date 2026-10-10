//! Lab sequencing and recovery policy. Readback checks the device, not the scene model.
use super::exchange;
use cathedral_contracts::{composition as w, display, user as abi};
use cathedral_user_runtime::{
    ipc::Handle,
    task::{Child, Launch, Outcome, Port},
    time,
};
type Pair = (Handle, Handle);
fn connect(index: u64) -> Pair {
    Port::at(index).unwrap().connect().unwrap()
}
fn call(pair: Pair, op: u64, arg: u64) -> [u64; 6] {
    exchange(pair, [op, arg, 0, 0, 0, 0])
}
fn place(display: Pair, client: u64, x: u64, y: u64) {
    assert_eq!(exchange(display, [w::PLACE, client, x, y, 256, 200])[0], 0);
}
fn pixel(display: Pair, x: u64, y: u64, color: u64) {
    assert_eq!(exchange(display, [w::PIXEL, x, y, 0, 0, 0])[1], color);
}
fn gone(display: Pair, client: u64) {
    let deadline = time::after(300).unwrap();
    while call(display, w::STATUS, client)[3] != 0 {
        assert!(!time::reached(time::now().unwrap(), deadline));
    }
}
fn marker(text: &[u8]) {
    cathedral_user_runtime::write(text).unwrap();
}
mod checks;
mod recovery;
struct Session {
    provider: Child,
    left: Child,
    right: Child,
    display: Pair,
    a: Pair,
    b: Pair,
}
impl Session {
    fn start() -> Self {
        let display_launch = Launch::at(0).unwrap();
        let left_launch = Launch::at(1).unwrap();
        let right_launch = Launch::at(2).unwrap();
        let provider = display_launch.spawn(0).unwrap();
        let left = left_launch.spawn(0).unwrap();
        let right = right_launch.spawn(0).unwrap();
        let display = Port::at(0)
            .unwrap()
            .connect()
            .unwrap_or_else(|e| panic!("display connect {e:?}: {:?}", provider.wait()));
        let a = connect(1);
        let b = connect(2);
        Self {
            provider,
            left,
            right,
            display,
            a,
            b,
        }
    }
}
pub fn run() -> u64 {
    let mut session = Session::start();
    session.drawing_checks();
    session.client_recovery();
    session.provider_recovery();
    marker(b"Cathedral compositor: ready\n");
    // Preserve the final scene and children for QMP or interactive viewing.
    loop {
        let _ = session
            .display
            .1
            .receive_until(&mut [0; 64], time::after(100).unwrap());
    }
}
