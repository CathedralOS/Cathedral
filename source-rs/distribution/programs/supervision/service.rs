use cathedral_user_runtime::{ipc::Handle, task::Launch};
use core::sync::atomic::{AtomicU64, Ordering};

static STARTS: AtomicU64 = AtomicU64::new(0);

pub fn run(round: u64) -> u64 {
    assert_eq!(STARTS.fetch_add(1, Ordering::Relaxed), 0);
    assert!(Launch::bootstrap().is_err());
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    let mut message = [0; 64];
    for _ in 0..4 {
        assert_eq!(input.receive(&mut message).unwrap(), 64);
        assert_eq!(u64::from_le_bytes(message[..8].try_into().unwrap()), round);
        output.send(&message).unwrap();
    }
    assert_eq!(input.receive(&mut message).unwrap(), 5);
    assert_eq!(&message[..5], b"crash");
    // SAFETY: Deliberate contained user fault; supervisor must observe the kernel report.
    unsafe {
        core::arch::asm!("ud2", options(noreturn));
    }
}
