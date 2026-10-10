//! Raw ABI probes intentionally bypass runtime ownership to test kernel enforcement.
use cathedral_contracts::user as abi;
use cathedral_user_runtime::link::Link;
use cathedral_user_runtime::{Error, ipc::Handle, memory::raw};
pub const FILL: u64 = 100;
pub const CHECK: u64 = 101;
pub const FAULT: u64 = 102;
pub const COPYOUT: u64 = 103;
pub const CHECK_HEAD: u64 = 104;
pub fn run() -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    loop {
        let mut bytes = [0; 64];
        let length = input.receive(&mut bytes).unwrap();
        assert_eq!(length, 24);
        let mut words = [0; 3];
        for (word, input) in words.iter_mut().zip(bytes[..length].chunks_exact(8)) {
            *word = u64::from_le_bytes(input.try_into().unwrap());
        }
        let [op, first, second] = words;
        let result = dispatch(op, first, second).unwrap_or_else(|Error(code)| code as u64);
        output.send(&result.to_le_bytes()).unwrap();
    }
}
fn dispatch(op: u64, first: u64, second: u64) -> Result<u64, Error> {
    match op {
        abi::MEMORY_ALLOCATE => raw::allocate(first as usize),
        abi::MEMORY_ADDRESS => raw::address(first),
        abi::MEMORY_PAGES => raw::pages(first).map(|n| n as u64),
        abi::MEMORY_MAP => raw::accept(first),
        abi::MEMORY_SEAL => {
            let link = if second == 0 {
                Link::at(0)?.raw()
            } else {
                second
            };
            // SAFETY: Raw probe holds no Rust references to the allocation.
            unsafe { raw::seal(first, link) }.map(|_| 0)
        }
        // SAFETY: Probe never retains Rust borrows across a syscall.
        abi::MEMORY_RELEASE => unsafe { raw::release(first) }.map(|_| 0),
        FILL | CHECK | CHECK_HEAD => {
            let address = raw::address(first)?;
            let length = if op == CHECK_HEAD {
                64
            } else {
                raw::pages(first)? * 4096
            };
            for index in 0..length {
                // SAFETY: Controller uses FILL only on an owned writable region;
                // CHECK uses its live owned/accepted mapping. No references retained.
                unsafe {
                    let pointer = (address as *mut u8).add(index);
                    if op == FILL {
                        pointer.write_volatile(second as u8);
                    } else {
                        assert_eq!(pointer.read_volatile(), second as u8);
                    }
                }
            }
            Ok(0)
        }
        FAULT => {
            // SAFETY: Deliberately forbidden ring-3 accesses. Each must fault and
            // terminate only this disposable probe; no Rust reference is created.
            unsafe {
                match second {
                    1 => core::arch::asm!("mov byte ptr [{0}], 1", in(reg) first),
                    2 => core::arch::asm!("call {0}", in(reg) first),
                    _ => core::arch::asm!("mov al, byte ptr [{0}]", in(reg) first, out("al") _),
                }
            }
            panic!()
        }
        COPYOUT => {
            let result;
            // SAFETY: Intentionally invalid syscall destination without forming
            // a mutable reference. Kernel must reject before waiting or copying.
            unsafe {
                core::arch::asm!("int 0x80", inlateout("rax") abi::IPC_RECEIVE => result,
                    in("rdi") Handle::bootstrap(0)?.raw(), in("rsi") first, in("rdx") 64u64,
                    options(nostack, preserves_flags));
            }
            Ok(result)
        }
        _ => Err(Error(abi::UNKNOWN as i64)),
    }
}
