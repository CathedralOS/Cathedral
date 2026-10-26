//! Device denial, failed display admission and independent observer progress.
use cathedral_kernel_contracts::framebuffer;
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::time;
use cathedral_user_runtime::{
    Error,
    connection::Client,
    ipc::Handle,
    server::{Server, SharedSlot},
    task::{Launch, Outcome},
};
pub fn owner() -> u64 {
    let launch = Launch::bootstrap().unwrap();
    for mode in 0..4 {
        let result = launch.spawn(mode).unwrap().wait().unwrap();
        match (mode, result) {
            (0, Outcome::Returned(0)) => {}
            (
                1,
                Outcome::Fault {
                    vector: 14,
                    error: 0x15,
                    address: framebuffer::USER_ADDRESS,
                    ..
                },
            ) => {}
            (
                2 | 3,
                Outcome::Fault {
                    vector: 14,
                    error: 4,
                    ..
                },
            ) => {}
            _ => panic!("unexpected framebuffer probe outcome"),
        }
    }
    done();
    0
}
pub fn failed_admission() -> u64 {
    let launch = Launch::bootstrap().unwrap();
    for _ in 0..2 {
        assert_eq!(launch.spawn(0).unwrap_err(), Error(abi::NO_MEMORY as i64));
    }
    done();
    0
}
pub fn peer() -> u64 {
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut server = Server::open_clients(0).unwrap();
    // SAFETY: One entry in this process, exclusively borrowed by the server.
    server.enable_shared_control(unsafe { &mut *slot }).unwrap();
    let request = server.next_request().unwrap();
    assert_eq!(&request.bytes[..request.len], b"done");
    server.reply(&request, b"done").unwrap();
    0
}

fn done() {
    let mut peer = Client::connect(
        (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap()),
        time::after(300).unwrap(),
    )
    .unwrap();
    let (reply, len) = peer.call_retained(b"done").unwrap();
    assert_eq!(&reply[..len], b"done");
    peer.retire();
}

pub fn observer() -> u64 {
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut server = Server::open_clients(0).unwrap();
    // SAFETY: One entry in this process, exclusively borrowed by the server.
    server.enable_shared_control(unsafe { &mut *slot }).unwrap();
    for _ in 0..2 {
        let request = server.next_request().unwrap();
        assert_eq!(request.len, 8);
        let deadline = u64::from_le_bytes(request.bytes[..8].try_into().unwrap());
        let start = time::now().unwrap();
        assert!(!time::reached(start, deadline));
        server.reply(&request, &start.to_le_bytes()).unwrap();
        let mut samples = 0u64;
        while !time::reached(time::now().unwrap(), deadline) {
            samples += 1;
        }
        assert!(samples > 1);
        let request = server.next_request().unwrap();
        assert_eq!(&request.bytes[..request.len], b"finish");
        server.reply(&request, &samples.to_le_bytes()).unwrap();
    }
    0
}

pub fn unmapped() -> ! {
    // SAFETY: Deliberately probe an ungranted virtual device address.
    // Assembly makes no Rust reference; the expected user fault is contained.
    unsafe {
        core::arch::asm!("mov rax, [rax]", "ud2", in("rax") framebuffer::USER_ADDRESS, options(noreturn));
    }
}
