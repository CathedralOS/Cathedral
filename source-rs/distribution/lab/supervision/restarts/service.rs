use cathedral_user_runtime::{
    server::{Server, SharedSlot},
    task::Launch,
};
use core::sync::atomic::{AtomicU64, Ordering};

static STARTS: AtomicU64 = AtomicU64::new(0);

pub fn run(round: u64) -> u64 {
    assert_eq!(STARTS.fetch_add(1, Ordering::Relaxed), 0);
    assert!(Launch::bootstrap().is_err());
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut client = Server::open_clients(0).unwrap();
    // SAFETY: Fresh process image per incarnation, with one server owning its slot.
    client.enable_shared_control(unsafe { &mut *slot }).unwrap();
    for _ in 0..4 {
        let request = client.next_request().unwrap();
        assert_eq!(request.len, 64);
        assert_eq!(
            u64::from_le_bytes(request.bytes[..8].try_into().unwrap()),
            round
        );
        client
            .reply(&request, &request.bytes[..request.len])
            .unwrap();
    }
    let request = client.next_request().unwrap();
    assert_eq!(&request.bytes[..request.len], b"crash");
    // SAFETY: Deliberate contained user fault; supervisor must observe the kernel report.
    unsafe {
        core::arch::asm!("ud2", options(noreturn));
    }
}
