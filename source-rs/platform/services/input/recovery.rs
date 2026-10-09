//! Test-only provider failures; absent from the ordinary executable.
use cathedral_user_runtime::ipc::Handle;

pub fn startup(mode: u64, generation: u64) {
    if generation == 0 && mode != 0 {
        fail(mode as u8, None);
    }
}
pub fn request(bytes: &[u8], input: Handle, output: Handle) {
    if let [0xf0, mode @ 1..=3] = bytes {
        output.send(b"armed").unwrap();
        fail(*mode, Some(input));
    }
}
fn fail(mode: u8, input: Option<Handle>) -> ! {
    match mode {
        1 => {
            // SAFETY: Deliberate userspace fault; only a recovery-lab build calls this.
            unsafe {
                core::arch::asm!("ud2", options(noreturn));
            }
        }
        3 => loop {
            input.unwrap().receive(&mut [0; 64]).unwrap();
        },
        _ => loop {
            core::hint::spin_loop();
        },
    }
}
