//! Exercise the actual IPC and shared-page path; no display or task-control grant.
use super::{exchange, scene};
use cathedral_contracts::{composition as w, display, user as abi};
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    link::Link,
    memory::{Private, Sealed},
};
pub fn run(role: u64) -> u64 {
    assert_eq!(
        cathedral_user_runtime::display::mapping(),
        Err(Error(abi::DENIED as i64))
    );
    let control = (Handle::bootstrap(0).unwrap(), Handle::bootstrap(1).unwrap());
    let link = Link::at(0).unwrap();
    let mut pair = None;
    let mut offer: Option<Sealed> = None;
    loop {
        let mut bytes = [0; 64];
        let len = control.0.receive(&mut bytes).unwrap();
        let [op, arg, extra, _, _, _] = display::decode(&bytes[..len]).unwrap();
        if op == 9 {
            pair = None;
        }
        if pair.is_none() {
            pair = Some(link.connect().unwrap());
        }
        let connection = pair.unwrap();
        let reply = match op {
            1..=3 => {
                let snapshot = scene::snapshot(role, op == 2);
                let mut page = Private::allocate(1).unwrap();
                page.bytes_mut()[..snapshot.len()].copy_from_slice(&snapshot);
                let sealed = page.seal(link).unwrap();
                let handle = sealed.handle();
                let result = exchange(connection, [w::SUBMIT, arg, handle, 0, 0, 0]);
                sealed.release().unwrap();
                [result[0], handle, 0, 0, 0, 0]
            }
            4 => exchange(connection, [w::SUBMIT, arg, extra, 0, 0, 0]),
            5 => {
                let snapshot = scene::snapshot(role, false);
                let mut page = Private::allocate(1).unwrap();
                page.bytes_mut()[..snapshot.len()].copy_from_slice(&snapshot);
                let sealed = page.seal(link).unwrap();
                let handle = sealed.handle();
                offer = Some(sealed);
                [0, handle, 0, 0, 0, 0]
            }
            6 => {
                let sealed = offer.take().unwrap();
                let result = exchange(connection, [w::SUBMIT, arg, sealed.handle(), 0, 0, 0]);
                sealed.release().unwrap();
                result
            }
            7 => {
                control.1.send(&display::encode([0; 6])).unwrap();
                loop {
                    core::hint::spin_loop();
                }
            }
            8 => {
                // Queue enough requests to overflow the undrained reply queue before
                // announcing the hostile mode. The supervisor then tests peer progress.
                let packet = display::encode([w::BACKGROUND, 0, 0, 0, 0, 0]);
                let mut sent = 0;
                while sent < 16 {
                    match connection.0.send(&packet) {
                        Ok(()) => sent += 1,
                        Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => {
                            cathedral_user_runtime::yield_now()
                        }
                        Err(error) => panic!("flood setup: {error:?}"),
                    }
                }
                control.1.send(&display::encode([0; 6])).unwrap();
                loop {
                    let _ = connection.0.send(&packet);
                }
            }
            9 => [0; 6],
            10 => {
                // SAFETY: Explicit lab-only invalid-opcode fault; kernel contains this client.
                unsafe {
                    core::arch::asm!("ud2", options(noreturn));
                }
            }
            11 => exchange(connection, [w::PLACE, 1, 0, 0, 256, 256]),
            _ => panic!(),
        };
        control.1.send(&display::encode(reply)).unwrap();
    }
}
