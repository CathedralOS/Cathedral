#![no_std]
#![no_main]
extern crate alloc;
mod transaction;
use cathedral_user_runtime::{
    heap::{self, Heap},
    memory::raw,
    resources, write,
};
#[global_allocator]
static HEAP: Heap = Heap::empty();
cathedral_user_runtime::entry!(main);
fn main(_: u64, mode: u64) -> u64 {
    assert_eq!(resources::budget().unwrap().used.private_pages, 0);
    heap::initialize(&HEAP, 64).unwrap();
    assert_eq!(resources::budget().unwrap().used.private_pages, 64);
    assert!(raw::allocate(1).is_err());
    let mut oversized = alloc::vec::Vec::<u8>::new();
    assert!(oversized.try_reserve_exact(512 * 1024).is_err());
    let mut probe = alloc::vec![0u8; 16 * 1024];
    assert!(probe.iter().all(|b| *b == 0));
    probe.fill(0xa7);
    if mode == 1 {
        core::mem::forget(probe);
        cathedral_user_runtime::exit(9);
    }
    if mode == 2 {
        // SAFETY: Deliberate contained guest fault; the worker owns no address zero.
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("mov byte ptr [rax], 1", in("rax") 0usize, options(nostack));
        }
        // SAFETY: Deliberate contained guest fault at an unmapped address.
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("strb wzr, [x0]", in("x0") 0usize, options(nostack));
        }
        unreachable!();
    }
    drop(probe);
    transaction::run();
    assert_eq!(HEAP.stats().live, 0, "engine leaked heap allocations");
    write(b"Cathedral database: worker heap returned to zero\n").unwrap();
    0
}
