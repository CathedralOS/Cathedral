//! Boot-selected negative tests, never part of the display request protocol.
use cathedral_contracts::{display, user as abi};
fn raw(number: u64, address: u64, length: u64) -> u64 {
    let result;
    // SAFETY: Hostile integer arguments enter only the checked syscall boundary.
    unsafe {
        core::arch::asm!("int 0x80", inlateout("rax") number => result,
        in("rdi") address, in("rsi") length, in("rdx") 0,
        options(nostack, preserves_flags));
    }
    result
}
pub fn run(mode: u64) -> u64 {
    let info = cathedral_user_runtime::display::mapping().unwrap();
    match mode {
        0 => {
            for address in [
                0,
                u64::MAX,
                b"readonly".as_ptr() as u64,
                display::USER_ADDRESS,
            ] {
                assert_eq!(raw(abi::DISPLAY_INFO, address, 48), abi::BAD_ADDRESS);
            }
            assert_eq!(raw(abi::DISPLAY_INFO, 0, 47), abi::INVALID_ARGUMENT);
            assert_eq!(raw(abi::WRITE, display::USER_ADDRESS, 4), abi::BAD_ADDRESS);
            0
        }
        1 => {
            // SAFETY: Deliberate NX probe; no Rust callable pointer is fabricated.
            unsafe {
                core::arch::asm!("jmp rax", in("rax") display::USER_ADDRESS, options(noreturn));
            }
        }
        2 | 3 => {
            let address = if mode == 2 {
                display::USER_ADDRESS - 4096
            } else {
                display::USER_ADDRESS + info[1]
            };
            // SAFETY: Deliberate guard-page access, expected to fault in userspace.
            unsafe {
                core::arch::asm!("mov rax, [rax]", "ud2", in("rax") address, options(noreturn));
            }
        }
        _ => 254,
    }
}
