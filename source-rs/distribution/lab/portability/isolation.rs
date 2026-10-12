//! Deliberate EL0 violations must retire only their own process.
use cathedral_user_runtime::{memory::Private, time};
pub fn survive(pattern: u8) -> u64 {
    let mut region = Private::allocate(2).unwrap();
    assert!(region.bytes_mut().iter().all(|b| *b == 0));
    region.bytes_mut().fill(pattern);
    let deadline = time::after(5).unwrap();
    while !time::reached(time::now().unwrap(), deadline) {
        #[cfg(target_arch = "aarch64")]
        {
            let observed: u32;
            // SAFETY: Bounded unprivileged loop; a caller-saved SIMD register retains
            // process-specific data across real timer interrupts and peer switches.
            unsafe {
                core::arch::asm!("dup v31.16b, {pattern:w}", "2:", "subs {count}, {count}, #1", "b.ne 2b", "umov {observed:w}, v31.b[15]",
                pattern=in(reg) u32::from(pattern),count=inout(reg) 10_000_000u64=>_,observed=lateout(reg) observed,out("v31") _,options(nostack));
            }
            assert_eq!(observed, u32::from(pattern));
        }
        #[cfg(target_arch = "x86_64")]
        for _ in 0..100_000 {
            core::hint::spin_loop();
        }
    }
    assert!(region.bytes_mut().iter().all(|b| *b == pattern));
    drop(region);
    0
}
pub fn fault(role: u64, address: u64) -> u64 {
    // SAFETY: Intentionally invalid accesses in an isolated disposable process.
    // Kernel assertions require the architectural fault, never successful return.
    unsafe {
        match role {
            2 => {
                core::ptr::read_volatile(address as *const u64);
            }
            4 => {
                core::ptr::write_volatile(address as *mut u64, 0);
            }
            5 => {
                let function: unsafe extern "C" fn() = core::mem::transmute(address);
                function();
            }
            _ => unreachable!(),
        }
    }
    253
}
