#![no_std]
//! Cathedral's initial scene, shared by ordinary startup and its display fixture.
use cathedral_contracts::{display as wire, user as abi};
use cathedral_user_runtime::{Error, ipc::Handle};

/// A drawing session; ordinary init supplies one deadline for the whole redraw.
/// Smoke fixtures may use the unbounded constructor to test unrelated mechanisms.
pub struct Client {
    send: Handle,
    receive: Handle,
    deadline: Option<u64>,
}
impl Client {
    pub fn text(&self, x: u64, y: u64, scale: u64, color: u64, text: &[u8]) -> Result<(), Error> {
        for (index, chunk) in text.chunks(16).enumerate() {
            let x = (index as u64)
                .checked_mul(16 * 6)
                .and_then(|n| n.checked_mul(scale))
                .and_then(|n| x.checked_add(n))
                .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
            let request = wire::encode_text(x, y, scale, color, chunk)
                .ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
            self.request(&request)?;
        }
        Ok(())
    }
    pub fn new(send: Handle, receive: Handle) -> Self {
        Self {
            send,
            receive,
            deadline: None,
        }
    }
    pub fn until(send: Handle, receive: Handle, deadline: u64) -> Self {
        Self {
            send,
            receive,
            deadline: Some(deadline),
        }
    }
    pub fn dimensions(&self) -> Result<(u64, u64), Error> {
        let reply = self.call([wire::INFO, 0, 0, 0, 0, 0])?;
        Ok((reply[1], reply[2]))
    }
    pub fn draw(&self, w: u64, h: u64) -> Result<(), Error> {
        if !(640..=8192).contains(&w) || !(480..=8192).contains(&h) {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        self.call([wire::CLEAR, 0, 0, 0, 0, 0x101827])?;
        for (x, y, width, height, color) in [
            (w / 16, h / 12, w * 7 / 8, h / 96, 0x59d9cc),
            (w / 16, h * 5 / 24, w / 4, h / 2, 0xe96f6f),
            (w * 6 / 16, h * 5 / 24, w / 4, h / 2, 0x79c99e),
            (w * 11 / 16, h * 5 / 24, w / 4, h / 2, 0x779bea),
        ] {
            self.call([wire::RECT, x, y, width, height, color])?;
        }
        Ok(())
    }
    pub fn draw_interactive(&self, w: u64, h: u64, selected: u8, active: u8) -> Result<(), Error> {
        self.draw(w, h)?;
        for index in 0..3 {
            let x = w * (1 + 5 * index) / 16;
            let y = h * 5 / 24;
            if active & (1 << index) != 0 {
                self.call([wire::RECT, x + 16, y + h / 4 - 8, w / 4 - 32, 16, 0x101827])?;
            }
            if u64::from(selected) == index {
                for (rx, ry, rw, rh) in [
                    (x - 4, y - 4, w / 4 + 8, 4),
                    (x - 4, y + h / 2, w / 4 + 8, 4),
                    (x - 4, y, 4, h / 2),
                    (x + w / 4, y, 4, h / 2),
                ] {
                    self.call([wire::RECT, rx, ry, rw, rh, 0xe8edf4])?;
                }
            }
        }
        Ok(())
    }
    fn call(&self, words: [u64; 6]) -> Result<[u64; 6], Error> {
        self.request(&wire::encode(words))
    }
    fn request(&self, request: &[u8]) -> Result<[u64; 6], Error> {
        self.send.send(request)?;
        let mut bytes = [0; 64];
        let length = match self.deadline {
            Some(deadline) => self.receive.receive_until(&mut bytes, deadline)?,
            None => self.receive.receive(&mut bytes)?,
        };
        let reply = wire::decode(&bytes[..length]).ok_or(Error(abi::IO_ERROR as i64))?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        Ok(reply)
    }
}

pub fn dimensions(send: Handle, receive: Handle) -> Result<(u64, u64), Error> {
    Client::new(send, receive).dimensions()
}
pub fn draw(send: Handle, receive: Handle, w: u64, h: u64) -> Result<(), Error> {
    Client::new(send, receive).draw(w, h)
}
