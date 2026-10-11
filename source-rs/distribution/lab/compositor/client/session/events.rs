//! App interpretation of already-authorized local events. No global focus discovery.
use super::*;
use core::fmt::Write;
impl App {
    pub(super) fn event(&mut self, w: [u64; 6]) -> Result<(), Error> {
        if w[1] == s::KEY_STREAM {
            if w[2] == s::FOCUS_LOST {
                self.held = [0; 2];
                return Ok(());
            }
            if w[2] != s::KEY_EVENT {
                return Err(Error(abi::INVALID_ARGUMENT as i64));
            }
            if w[4] == u64::from(input::PRESS) {
                match w[3] as u8 {
                    input::LEFT | input::UP => self.selected = 0,
                    input::RIGHT | input::DOWN => self.selected = 1,
                    input::ENTER => self.activate(),
                    _ => (),
                }
                self.publish()?;
            }
            self.report("key", w[3], w[4])?;
            return Ok(());
        }
        let event = p::Event::decode(w).ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
        let id = event.pointer as usize;
        if event.kind == p::CANCEL {
            self.held[id] = 0;
            self.report("cancel", event.pointer, 0)?;
            return Ok(());
        }
        let pressed = self.held[id] == 0 && event.buttons != 0;
        if pressed {
            let _ = self.rpc([p::CAPTURE, event.pointer, 1, 0, 0, 0])?;
        }
        self.held[id] = event.buttons;
        if pressed && event.buttons & 1 != 0 && (32..56).contains(&event.y) {
            self.selected = if event.x < 96 { 0 } else { 1 };
            self.activate();
            self.publish()?;
        } else if event.buttons != 0 {
            self.marker = Some((event.x.clamp(0, 244) as u64, event.y.clamp(0, 188) as u64));
            self.publish()?;
        }
        self.report("pointer", event.pointer, u64::from(event.buttons))?;
        if self.role == 1 && event.pointer == 0 && event.buttons & 3 == 3 {
            // SAFETY: Lab fault injection in this isolated application only.
            unsafe {
                core::arch::asm!("ud2", options(noreturn));
            }
        }
        if self.role == 2 && event.pointer == 0 && self.lease != 0 {
            self.rpc([
                p::DRIVE,
                self.lease,
                event.x.max(0) as u64 + 40,
                event.y.max(0) as u64 + 40,
                u64::from(event.buttons & 2 != 0),
                0,
            ])?;
        }
        Ok(())
    }
    fn activate(&mut self) {
        if self.selected == 0 {
            self.value = (self.value + 1) % 1000;
        } else {
            self.value = 0;
        }
    }
    fn report(&self, kind: &str, a: u64, b: u64) -> Result<(), Error> {
        struct Line {
            bytes: [u8; 192],
            len: usize,
        }
        impl Write for Line {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let end = self.len + s.len();
                if end > self.bytes.len() {
                    return Err(core::fmt::Error);
                }
                self.bytes[self.len..end].copy_from_slice(s.as_bytes());
                self.len = end;
                Ok(())
            }
        }
        let mut line = Line {
            bytes: [0; 192],
            len: 0,
        };
        writeln!(
            line,
            "Cathedral session: app={} {kind} a={a} b={b} value={} selected={}",
            self.role, self.value, self.selected
        )
        .unwrap();
        cathedral_user_runtime::write(&line.bytes[..line.len])
    }
}
