//! Fatal probes run in separate QEMU boots. Match the access kind and exact
//! address, so an unrelated crash or invalid instruction cannot pass as NX.
use cathedral_arch as arch;
use core::{
    arch::asm,
    sync::atomic::{AtomicU64, Ordering},
};
static KIND: AtomicU64 = AtomicU64::new(0);
static ADDRESS: AtomicU64 = AtomicU64::new(0);
static READ_ONLY: [u8; 32] = *b"Cathedral immutable kernel bytes";

pub fn expect_guard(address: u64) {
    expect(1, address);
}
fn expect(kind: u64, address: u64) {
    ADDRESS.store(address, Ordering::Release);
    KIND.store(kind, Ordering::Release);
}
pub fn probe(layout: &arch::BootLayout) {
    let (kind, address) = if cfg!(feature = "fault-text-write") {
        (2, probe as *const () as u64)
    } else if cfg!(feature = "fault-rodata-write") {
        (2, READ_ONLY.as_ptr() as u64)
    } else if cfg!(feature = "fault-heap-execute") {
        (3, layout.heap_start)
    } else if cfg!(feature = "fault-stack-execute") {
        (3, layout.stack_bottom)
    } else if cfg!(feature = "fault-heap-alias-execute") {
        (
            3,
            // SAFETY: Before enabling IRQs; owned kernel root is active.
            unsafe { layout.physical_address(layout.heap_start) }.expect("heap backing"),
        )
    } else if cfg!(feature = "fault-stack-alias-execute") {
        (
            3,
            // SAFETY: Before enabling IRQs; owned kernel root is active.
            unsafe { layout.physical_address(layout.stack_bottom) }.expect("stack backing"),
        )
    } else {
        return;
    };
    expect(kind, address);
    // SAFETY: Deliberate hardware access after installing terminal fault diagnostics.
    // Assembly avoids creating a Rust mutable reference to code/immutable storage,
    // or a Rust function pointer to data. Each probe must fault before returning.
    unsafe {
        if kind == 2 {
            #[cfg(target_arch = "x86_64")]
            asm!("mov byte ptr [{address}], 0", address=in(reg) address, options(nostack));
            #[cfg(target_arch = "aarch64")]
            asm!("strb wzr, [{address}]", address=in(reg) address, options(nostack));
        } else {
            #[cfg(target_arch = "x86_64")]
            asm!("jmp {address}", address=in(reg) address, options(noreturn));
            #[cfg(target_arch = "aarch64")]
            asm!("br {address}", address=in(reg) address, options(noreturn));
        }
    }
    panic!("kernel protection probe returned");
}
pub fn matches(fault: &arch::Fault) -> bool {
    let kind = KIND.load(Ordering::Acquire);
    let address = ADDRESS.load(Ordering::Acquire);
    if kind == 0 || fault.address != address || (kind == 3 && fault.instruction != address) {
        return false;
    }
    #[cfg(target_arch = "x86_64")]
    {
        let error = match kind {
            1 => 2,
            2 => 3,
            3 => 17,
            _ => return false,
        };
        fault.vector == 14 && fault.error & 0x1f == error
    }
    #[cfg(target_arch = "aarch64")]
    {
        let status = fault.error & 0x3f;
        match kind {
            1 => fault.vector == 0x25 && fault.error & (1 << 6) != 0 && status == 7,
            2 => fault.vector == 0x25 && fault.error & (1 << 6) != 0 && status == 15,
            3 => fault.vector == 0x21 && matches!(status, 14 | 15),
            _ => false,
        }
    }
}
