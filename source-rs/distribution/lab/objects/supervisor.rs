use cathedral_platform_contracts::storage as wire;
use cathedral_user_runtime::{
    connection::Client,
    task::{Launch, Outcome, Port},
    time, write, yield_now,
};
pub fn run() -> u64 {
    let provider = Launch::at(0).unwrap().spawn(0).unwrap();
    let deadline = time::after(300).unwrap();
    let mut control = Client::connect(Port::at(0).unwrap().connect().unwrap(), deadline).unwrap();
    let (reply, n) = control.call(wire::HEALTH, deadline).unwrap();
    assert_eq!(&reply[..n], b"ready");
    write(b"Cathedral objects: shared owner control ready\n").unwrap();
    let first = Launch::at(1).unwrap().spawn(0).unwrap().wait().unwrap();
    let second = Launch::at(2).unwrap().spawn(1).unwrap().wait().unwrap();
    assert_eq!(first, second);
    let first_marker = if cfg!(feature = "retention") { 9 } else { 1 };
    let final_marker = if cfg!(feature = "retention") { 26 } else { 2 };
    if cfg!(feature = "retention") && first == Outcome::Returned(25) {
        write(b"Cathedral objects: collected revision 25\n").unwrap();
    } else if first == Outcome::Returned(first_marker) {
        #[cfg(feature = "retention")]
        write(b"Cathedral objects: pinned revision 9\n").unwrap();
        #[cfg(not(feature = "retention"))]
        write(b"Cathedral objects: acknowledged revision 1\n").unwrap();
        // Host kills QEMU here, with the provider's committed database still open.
    } else {
        assert_eq!(first, Outcome::Returned(final_marker));
        let (reply, n) = control
            .call_retained(&wire::Request::new(wire::CLOSE, 0, 0, 0, 0).encode())
            .unwrap();
        assert_eq!(n, 64);
        assert_eq!(reply, [0; 64]);
        assert_eq!(provider.wait().unwrap(), Outcome::Returned(0));
        control.retire();
        let used = cathedral_user_runtime::resources::budget().unwrap().used;
        assert_eq!(
            [used.private_pages, used.reader_pages, used.channels],
            [0; 3]
        );
        let notifications = cathedral_user_runtime::notification::budget().unwrap();
        assert_eq!([notifications[1], notifications[3]], [0; 2]);
        write(b"Cathedral objects: shared owner resources reclaimed\n").unwrap();
        write(b"Cathedral objects: recovered content and private roots passed\n").unwrap();
    }
    loop {
        yield_now();
    }
}
