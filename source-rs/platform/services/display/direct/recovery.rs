//! Test-only provider failures; absent from the ordinary executable.
use cathedral_user_runtime::server::{Request, Server};

pub fn startup(mode: u64, generation: u64) {
    if generation == 0 && mode != 0 {
        fail(mode as u8, None);
    }
}
pub fn request(server: &mut Server, request: &Request) {
    if let [0xf0, mode @ 1..=3] = &request.bytes[..request.len] {
        server.reply(request, b"armed").unwrap();
        fail(*mode, Some(server));
    }
}
fn fail(mode: u8, mut server: Option<&mut Server>) -> ! {
    match mode {
        1 => {
            // SAFETY: Deliberate userspace fault; only a recovery-lab build calls this.
            unsafe {
                core::arch::asm!("ud2", options(noreturn));
            }
        }
        3 => loop {
            server.as_mut().unwrap().next_request().unwrap();
        },
        _ => loop {
            core::hint::spin_loop();
        },
    }
}
