//! Pointer-driven app content. Root cursor rendering and authority stay in the provider.
use super::*;
use cathedral_contracts::pointer as p;
use cathedral_user_runtime::time;
use core::fmt::Write;
pub fn run(
    role: u64,
    lease: u64,
    control: (Handle, Handle),
    connection: (Handle, Handle),
    link: Link,
) -> u64 {
    let mut held = [0; 2];
    let mut revision = 1;
    let mut pending = false;
    loop {
        let mut bytes = [0; 64];
        match control.0.receive_until(&mut bytes, time::now().unwrap()) {
            Ok(_) => control.1.send(&display::encode([0; 6])).unwrap(),
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            Err(_) => return 0,
        }
        if !pending {
            connection
                .0
                .send(&display::encode([p::WAIT, 0, 0, 0, 0, 0]))
                .unwrap();
            pending = true;
        }
        match cathedral_user_runtime::ipc::wait_two(
            control.0,
            connection.1,
            time::after(500).unwrap(),
        ) {
            Ok(1) => (),
            Ok(_) => continue,
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => continue,
            Err(_) => return 0,
        }
        let mut packet = [0; 64];
        let len = connection.1.receive(&mut packet).unwrap();
        pending = false;
        let event = p::Event::decode(display::decode(&packet[..len]).unwrap()).unwrap();
        if event.kind == p::NONE {
            cathedral_user_runtime::yield_now();
            continue;
        }
        let id = event.pointer as usize;
        if event.kind == p::CANCEL {
            held[id] = 0;
        } else {
            if held[id] == 0 && event.buttons != 0 {
                // A release or cancellation can race this request. Denial is normal.
                let _ = exchange(connection, [p::CAPTURE, event.pointer, 1, 0, 0, 0]);
            }
            held[id] = event.buttons;
        }
        if event.buttons != 0 && event.kind != p::CANCEL {
            revision += 1;
            let snapshot = scene::interactive(
                role,
                false,
                Some((event.x.clamp(0, 244) as u64, event.y.clamp(0, 188) as u64)),
            );
            let mut page = Private::allocate(1).unwrap();
            page.bytes_mut()[..snapshot.len()].copy_from_slice(&snapshot);
            let sealed = page.seal(link).unwrap();
            assert_eq!(
                exchange(connection, [w::SUBMIT, revision, sealed.handle(), 0, 0, 0])[0],
                0
            );
            sealed.release().unwrap();
        }
        report(role, event);
        if role == 1 && event.pointer == 0 {
            if event.buttons & 3 == 3 {
                // SAFETY: Deliberate lab fault while captured, contained by the kernel.
                unsafe {
                    core::arch::asm!("ud2", options(noreturn));
                }
            }
            if event.buttons & 4 != 0 {
                cathedral_user_runtime::write(b"Cathedral pointer: app 1 flooding\n").unwrap();
                loop {
                    let _ = connection
                        .0
                        .send(&display::encode([w::BACKGROUND, 0, 0, 0, 0, 0]));
                }
            }
        }
        // Right-button interaction in app two drives its separate leased pointer.
        // The lease token conveys no permission over the physical source.
        if role == 2 && event.pointer == 0 && event.kind != p::CANCEL && lease != 0 {
            assert_eq!(
                exchange(
                    connection,
                    [
                        p::DRIVE,
                        lease,
                        event.x.max(0) as u64 + 40,
                        event.y.max(0) as u64 + 40,
                        u64::from(event.buttons & 2 != 0),
                        0
                    ]
                )[0],
                0
            );
        }
    }
}
fn report(role: u64, event: p::Event) {
    struct Line {
        bytes: [u8; 160],
        len: usize,
    }
    impl Write for Line {
        fn write_str(&mut self, text: &str) -> core::fmt::Result {
            let end = self.len + text.len();
            if end > self.bytes.len() {
                return Err(core::fmt::Error);
            }
            self.bytes[self.len..end].copy_from_slice(text.as_bytes());
            self.len = end;
            Ok(())
        }
    }
    let mut line = Line {
        bytes: [0; 160],
        len: 0,
    };
    writeln!(
        line,
        "Cathedral pointer: app={role} id={} kind={} x={} y={} buttons={}",
        event.pointer, event.kind, event.x, event.y, event.buttons
    )
    .unwrap();
    cathedral_user_runtime::write(&line.bytes[..line.len]).unwrap();
}
