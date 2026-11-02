//! Test-only stalled display. Reserved corner positions arm the next image,
//! commit, rectangle, pointer or refused commit; ordinary builds have no such controls.
use super::*;
use cathedral_user_runtime::time;
#[derive(Default)]
pub(super) struct Gate {
    armed: u64,
    refuse: bool,
    held: Option<(Event, u64)>,
    abort: Option<u64>,
}
impl Gate {
    pub fn next(&mut self, server: &mut Server) -> Result<Event, Error> {
        loop {
            if self
                .held
                .as_ref()
                .is_some_and(|(_, until)| time::reached(time::now().unwrap(), *until))
            {
                let (event, _) = self.held.take().unwrap();
                if self.refuse {
                    let Event::Request(request) = event else {
                        unreachable!()
                    };
                    let words = display::decode(&request.bytes[..request.len]).unwrap();
                    server.reply(&request, &display::encode([abi::DENIED, 0, 0, 0, 0, 0]))?;
                    self.abort = Some(words[1]);
                    self.refuse = false;
                    time::storage_sample("display-refused", &[]);
                    continue;
                }
                time::storage_sample("display-released", &[]);
                return Ok(event);
            }
            let event = server.next_event_for(&[], self.held.as_ref().map(|(_, until)| *until))?;
            let Some(event) = event else { continue };
            let request = match &event {
                Event::Request(r) | Event::Memory(r, _) => r,
                _ => return Ok(event),
            };
            if !request.control {
                return Ok(event);
            }
            let Some(words) = display::decode(&request.bytes[..request.len]) else {
                return Ok(event);
            };
            if words[0] == wire::MATRIX_ABORT && self.abort == Some(words[1]) {
                self.abort = None;
                time::storage_sample("display-aborted", &[]);
            }
            if words[0] == wire::MATRIX_POINTER
                && words[2] == 3
                && (3..=7).contains(&words[1])
                && self.held.is_none()
            {
                self.armed = match words[1] {
                    3 => wire::MATRIX_IMAGE,
                    5 => wire::MATRIX_RECT,
                    7 => wire::MATRIX_POINTER,
                    _ => wire::MATRIX_COMMIT,
                };
                self.refuse = words[1] == 6;
                time::storage_sample("display-armed", &[self.armed]);
            } else if self.armed != 0 && words[0] & 0xffff == self.armed && self.held.is_none() {
                self.armed = 0;
                self.held = Some((event, time::after(500)?));
                time::storage_sample("display-held", &[]);
                continue;
            }
            return Ok(event);
        }
    }
}
