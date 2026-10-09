#![no_std]
#![no_main]

//! Each instance checks fresh data/BSS, retains private state across yields,
//! writes through the shared runtime and returns its own supplied exit status.
use cathedral_user_runtime as runtime;
use core::sync::atomic::{AtomicU64, Ordering};

runtime::entry!(main);
static DATA: AtomicU64 = AtomicU64::new(0x1234_5678);
static BSS: [AtomicU64; 1024] = [const { AtomicU64::new(0) }; 1024];
#[repr(align(4096))]
struct Message([u8; 8192]);
static MESSAGE: Message = Message([b'.'; 8192]);

fn main(status: u64, _: u64) -> u64 {
    assert_eq!(DATA.load(Ordering::Relaxed), 0x1234_5678);
    for value in &BSS {
        assert_eq!(value.load(Ordering::Relaxed), 0);
    }
    DATA.store(status, Ordering::Relaxed);
    for (index, value) in BSS.iter().enumerate() {
        value.store(status ^ index as u64, Ordering::Relaxed);
    }
    for _ in 0..16 {
        runtime::yield_now();
    }
    assert_eq!(DATA.load(Ordering::Relaxed), status);
    for (index, value) in BSS.iter().enumerate() {
        assert_eq!(value.load(Ordering::Relaxed), status ^ index as u64);
    }
    // One slice crosses both a page boundary and the kernel's 256-byte call cap:
    // 6 + 256 + 38 bytes. No kernel pointer or unsafe code appears in the app.
    runtime::write(&MESSAGE.0[4090..4390]).unwrap();
    runtime::write(b"hello from a separately compiled Rust program\n").unwrap();
    status
}
