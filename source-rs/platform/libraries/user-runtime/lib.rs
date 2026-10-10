#![no_std]

//! Minimal Rust-lab userspace support. No allocator, kernel imports or OS policy.
//! Link x86_64.S at _start; the application's entry! macro supplies its Rust main.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("Build the user runtime for x86_64-unknown-none");

mod arch;
pub mod display;
pub mod ipc;
pub mod keyboard;
pub mod link;
pub mod server;
pub mod task;
pub mod time;
use cathedral_contracts::user as abi;

#[derive(Debug, PartialEq, Eq)]
pub struct Error(pub i64);

/// Split arbitrary slices at both ABI-size and page boundaries.
pub fn write(mut bytes: &[u8]) -> Result<(), Error> {
    while !bytes.is_empty() {
        let address = bytes.as_ptr() as u64;
        let length = bytes
            .len()
            .min(abi::MAX_WRITE)
            .min(4096 - (address as usize & 4095));
        let result = arch::call(abi::WRITE, address, length as u64);
        if (result as i64) < 0 {
            return Err(Error(result as i64));
        }
        if result != length as u64 {
            return Err(Error(abi::IO_ERROR as i64));
        }
        bytes = &bytes[length..];
    }
    Ok(())
}

pub fn yield_now() {
    arch::call(abi::YIELD, 0, 0);
}
pub fn exit(status: u64) -> ! {
    arch::call(abi::EXIT, status, 0);
    // If a broken kernel resumes exit, keep trapping instead of returning into
    // an absent caller. Correct kernels never reach this instruction.
    arch::stop()
}

#[macro_export]
macro_rules! entry {
    ($main:path) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn cathedral_user_main(first: u64, second: u64) -> ! {
            let main: fn(u64, u64) -> u64 = $main;
            $crate::exit(main(first, second))
        }
        #[panic_handler]
        fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
            $crate::exit(255)
        }
    };
}
