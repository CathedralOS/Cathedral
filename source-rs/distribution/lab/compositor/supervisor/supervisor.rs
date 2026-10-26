// Lab sequencing and recovery policy. Readback checks the device, not the scene model.
use super::exchange;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display;
use cathedral_platform_contracts::display::composition as w;
use cathedral_user_runtime::{
    connection::Client,
    task::{Child, Launch, Outcome, Port},
    time,
};
fn connect(index: u64) -> Client {
    Client::connect(
        Port::at(index).unwrap().connect().unwrap(),
        time::after(300).unwrap(),
    )
    .unwrap()
}
fn call(pair: &mut Client, op: u64, arg: u64) -> [u64; 6] {
    exchange(pair, [op, arg, 0, 0, 0, 0])
}
fn place(display: &mut Client, client: u64, x: u64, y: u64) {
    assert_eq!(exchange(display, [w::PLACE, client, x, y, 256, 200])[0], 0);
}
fn pixel(display: &mut Client, x: u64, y: u64, color: u64) {
    assert_eq!(exchange(display, [w::PIXEL, x, y, 0, 0, 0])[1], color);
}
fn gone(display: &mut Client, client: u64) {
    let deadline = time::after(300).unwrap();
    while call(display, w::STATUS, client)[3] != 0 {
        assert!(!time::reached(time::now().unwrap(), deadline));
    }
}
fn marker(text: &[u8]) {
    cathedral_user_runtime::write(text).unwrap();
}
struct Session {
    provider: Child,
    left: Child,
    right: Child,
    display: Client,
    a: Client,
    b: Client,
}
impl Session {
    fn start() -> Self {
        let display_launch = Launch::at(0).unwrap();
        let left_launch = Launch::at(1).unwrap();
        let right_launch = Launch::at(2).unwrap();
        let provider = display_launch.spawn(0).unwrap();
        let left = left_launch.spawn(0).unwrap();
        let right = right_launch.spawn(0).unwrap();
        let display_pair = Port::at(0)
            .unwrap()
            .connect()
            .unwrap_or_else(|e| panic!("display connect {e:?}: {:?}", provider.wait()));
        let display = Client::connect(display_pair, time::after(300).unwrap()).unwrap();
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
    if let Ok(child) = Launch::at(4)
        && !cfg!(feature = "session-probes")
    {
        assert_eq!(call(&mut session.a, 1, 1)[0], 0);
        assert_eq!(call(&mut session.b, 1, 1)[0], 0);
        marker(b"Cathedral session: interactive startup; destructive probes disabled\n");
        return session.desktop(child);
    }
    session.drawing_checks();
    session.client_recovery();
    session.provider_recovery();
    marker(b"Cathedral compositor: ready\n");
    if let Ok(child) = Launch::at(4) {
        return session.desktop(child);
    }
    if let Ok(source) = Launch::at(3) {
        return session.pointers(source);
    }
    // Preserve the final scene and children for QMP or interactive viewing.
    loop {
        crate::transport::idle(100);
    }
}
