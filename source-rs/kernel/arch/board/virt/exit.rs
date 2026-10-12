//! QEMU virt semihosted lab exit.
use core::arch::asm;
/// # Safety
/// Only the lab harness enables semihosting; no guest-provided arguments reach it.
pub unsafe fn qemu_exit(status: u32) -> ! {
    let block = [0x20026u64, if status == 0x10 { 0 } else { 1 }];
    // SAFETY: QEMU SYS_EXIT_EXTENDED, with a live two-word host exit block.
    unsafe {
        asm!("hlt #0xf000", in("x0") 0x20u64, in("x1") block.as_ptr(), options(nostack));
        crate::halt_forever()
    }
}
