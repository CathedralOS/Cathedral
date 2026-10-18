//! Test-only gates let the QEMU driver resize a view during surface creation
//! and bitmap publication. Normal Notes contains neither gates nor delays.
use cathedral_user_runtime::{time, write, yield_now};
use core::sync::atomic::{AtomicU8, Ordering};

pub fn gate(stage: u8) {
    static SEEN: AtomicU8 = AtomicU8::new(0);
    if SEEN.fetch_or(stage, Ordering::Relaxed) & stage != 0 {
        return;
    }
    write(if stage == 1 {
        b"Cathedral Notes: render gate create\n"
    } else {
        b"Cathedral Notes: render gate image\n"
    })
    .unwrap();
    // 2000 ticks are nominally 20 seconds on the lab's 100-Hz clock. The test
    // observes the gate marker before issuing a real settings request.
    let deadline = time::after(2000).unwrap();
    while !time::reached(time::now().unwrap(), deadline) {
        yield_now();
    }
}
