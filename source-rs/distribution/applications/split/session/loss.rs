//! Ask descendants asynchronously; silence and malformed replies mean keep open.
//! A timed-out request stays owned by its connection; it is never a fresh answer.
use super::*;
use cathedral_user_runtime::server::Request;
pub struct Survey {
    pub request: Option<Request>,
    pub waiting: u8,
    pub deadline: u64,
    clean: bool,
}
impl Survey {
    pub fn start(request: Option<Request>, children: &mut Children) -> Result<Self, Error> {
        let mut s = Self {
            request,
            waiting: 0,
            deadline: time::after(100)?,
            clean: true,
        };
        for slot in 0..2 {
            if children.slots[slot].is_some() {
                if children.uncertain[slot]
                    || children.controls[slot]
                        .as_mut()
                        .is_none_or(|control| control.ask().is_err())
                {
                    s.clean = false;
                } else {
                    s.waiting |= 1 << slot;
                }
            }
        }
        Ok(s)
    }
    pub fn progress(&mut self, children: &mut Children) -> Result<Option<bool>, Error> {
        let now = time::now()?;
        for slot in 0..2 {
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
        Ok((self.waiting == 0).then_some(self.clean))
    }
}
