//! Transitional grant handoff/funding adapter. Notification waits carry no payload.
use super::*;
impl Ipc {
    pub fn create_notification(&mut self, caller: usize) -> Result<[u64; 3], u64> {
        if !self.live(caller) {
            return Err(abi::BAD_HANDLE);
        }
        if self.charged_channels(caller) >= self.reservations[caller] {
            return Err(abi::NO_MEMORY);
        }
        self.notifications.create(caller)
    }
    pub fn send_notification(
        &mut self,
        caller: usize,
        ticket: u64,
        bytes: &[u8],
        right: u64,
    ) -> Result<(), u64> {
        let recipient = self.memory_receiver(caller, ticket, bytes.len())?;
        let escrow = self.notifications.reserve(caller, right, recipient)?;
        let index = self.check(caller, ticket, Right::Send).unwrap();
        let mut message = Queued {
            bytes: [0; abi::MAX_MESSAGE],
            len: bytes.len(),
            attachment: None,
            memory: None,
            notification: Some(escrow),
        };
        message.bytes[..bytes.len()].copy_from_slice(bytes);
        self.endpoints[index].as_mut().unwrap().message = Some(message);
        self.changed_receivers |= 1 << recipient;
        Ok(())
    }
    pub fn receive_notification(
        &mut self,
        caller: usize,
        ticket: u64,
        capacity: usize,
    ) -> Result<Option<Message>, u64> {
        self.receive_kind(caller, ticket, capacity, 3)
    }
    pub fn notification_budget(&self, caller: usize) -> [u64; 4] {
        [
            self.reservations[caller] as u64,
            self.notifications.charged(caller) as u64,
            abi::NOTIFY_HANDLES as u64,
            self.notifications.handles(caller) as u64,
        ]
    }
}
