//! Ask descendants asynchronously; silence and malformed replies mean keep open.
//! A timed-out request stays owned by its connection; it is never a fresh answer.
use super::*;
use cathedral_user_runtime::server::Request;
pub struct Survey {
    pub request: Option<Request>,
    pub waiting: u8,
    pub deadline: u64,
    unasked: u8,
    clean: bool,
}
impl Survey {
    pub fn start(request: Option<Request>, children: &mut Children) -> Result<Self, Error> {
        let mut s = Self {
            request,
            waiting: 0,
            unasked: 0,
            deadline: time::after(100)?,
            clean: true,
        };
        for slot in 0..2 {
            if children.slots[slot].is_some() {
                if children.uncertain[slot] {
                    s.clean = false;
                    continue;
                }
                match children.controls[slot]
                    .as_mut()
                    .map(|control| control.ask())
                {
                    Some(Ok(true)) => s.waiting |= 1 << slot,
                    Some(Ok(false)) => s.unasked |= 1 << slot,
                    _ => s.clean = false,
                }
            }
        }
        if s.unasked != 0 {
            time::storage_sample("split-loss-awaiting-control", &[s.unasked as u64]);
        }
        Ok(s)
    }
    pub fn progress(&mut self, children: &mut Children) -> Result<Option<bool>, Error> {
        let now = time::now()?;
        for slot in 0..2 {
            if self.unasked & (1 << slot) != 0 {
                // Setup keeps yielding to the children's upstream work. An
                // expired unsent query must neither be published nor mark an
                // old reply outstanding; a later user action can retry safely.
                let result = if time::reached(now, self.deadline) {
                    None
                } else {
                    children.controls[slot]
                        .as_mut()
                        .map(|control| control.ask())
                };
                match result {
                    Some(Ok(false)) => continue,
                    Some(Ok(true)) => self.waiting |= 1 << slot,
                    _ => self.clean = false,
                }
                self.unasked &= !(1 << slot);
            }
            if self.waiting & (1 << slot) == 0 {
                continue;
            }
            let Some(control) = &mut children.controls[slot] else {
                self.waiting &= !(1 << slot);
                continue;
            };
            match control.answer() {
                Ok(Some(clean)) => {
                    self.clean &= clean;
                    self.waiting &= !(1 << slot);
                }
                Ok(None) if !time::reached(now, self.deadline) => (),
                Ok(None) | Err(_) => {
                    self.clean = false;
                    children.uncertain[slot] = true;
                    self.waiting &= !(1 << slot);
                }
            }
        }
        Ok((self.waiting | self.unasked == 0).then_some(self.clean))
    }
}
