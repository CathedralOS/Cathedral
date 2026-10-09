#![no_std]
#![no_main]
//! Cathedral startup policy: launch the approved display provider and own the scene.
use cathedral_user_runtime::{
    Error,
    task::{Launch, Port},
    write,
};
cathedral_user_runtime::entry!(main);

fn main(_: u64, _: u64) -> u64 {
    match start() {
        Ok(()) => 0,
        Err(_) => {
            write(b"Cathedral: startup failed\n").ok();
            1
        }
    }
}
fn start() -> Result<(), Error> {
    let launch = Launch::bootstrap()?;
    let port = Port::bootstrap()?;
    // A small, explicit distribution policy. Healthy service and init stay
    // blocked without polling; repeated failures eventually stop this session.
    for generation in 0..3 {
        let child = launch.spawn(generation)?;
        let drawing = port.connect().and_then(|(send, receive)| {
            cathedral_boot_scene::dimensions(send, receive)
                .and_then(|(w, h)| cathedral_boot_scene::draw(send, receive, w, h))
        });
        if drawing.is_err() {
            child.cancel()?;
        } else {
            write(b"Cathedral: startup ready\n")?;
        }
        child.wait()?;
        if generation < 2 {
            write(b"Cathedral: restarting display provider\n")?;
        }
    }
    Err(Error(cathedral_contracts::user::IO_ERROR as i64))
}
