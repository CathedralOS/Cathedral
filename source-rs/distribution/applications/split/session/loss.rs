//! Ask descendants asynchronously; silence and malformed replies mean keep open.
//! A timed-out uncorrelated control stream stays uncertain until that child exits.
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
            if let Some(child) = &children.slots[slot] {
                if children.uncertain[slot]
                    || child
                        .control
                        .0
                        .send(&wire::encode([wire::LOSS, 0, 0, 0, 0, 0, 0, 0]))
                        .is_err()
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
            let Some(child) = &children.slots[slot] else {
                self.waiting &= !(1 << slot);
                continue;
            };
            let mut bytes = [0; 64];
            match child.control.1.receive_until(&mut bytes, now) {
                Ok(len) => {
                    self.clean &= wire::decode(&bytes[..len]) == Some([0; 8]);
                    self.waiting &= !(1 << slot);
                }
                Err(Error(code))
                    if code == abi::TIMED_OUT as i64 && !time::reached(now, self.deadline) => {}
                Err(_) => {
                    self.clean = false;
                    children.uncertain[slot] = true;
                    self.waiting &= !(1 << slot);
                }
            }
        }
        Ok((self.waiting == 0).then_some(self.clean))
    }
}
