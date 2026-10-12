//! QEMU PC lab exit device.
use core::arch::asm;
/// # Safety
/// Requires QEMU's isa-debug-exit device at port 0xf4 and I/O privilege.
pub unsafe fn qemu_exit(value: u32) -> ! {
    // SAFETY: The smoke-test runner supplies this exact emulated device.
    unsafe {
        asm!("out dx, eax", in("dx") 0xf4u16, in("eax") value, options(nomem, nostack));
    }
    // SAFETY: A missing exit device must fail by idling, never return to firmware.
    unsafe { crate::halt_forever() }
}
