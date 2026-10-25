//! Join one private frame snapshot and one typed grant. Neither half
//! alone authorizes dispatch; mismatches fail closed and Drop releases retained custody.
use super::*;
pub(crate) struct Message {
    pub id: u64,
    pub bytes: [u8; 64],
    pub len: usize,
    pub object: Option<Object>,
    pub endpoint: Option<Endpoint>,
    pub notification: Option<NotificationGrant>,
}
struct Frame {
    id: u64,
    bytes: [u8; 64],
    len: usize,
    attachment: wire::Attachment,
}
enum Grant {
    Memory(Object),
    Endpoint(Endpoint),
    Notification(NotificationGrant),
}
impl Grant {
    fn kind(&self) -> wire::Attachment {
        match self {
            Self::Memory(_) => wire::Attachment::Memory,
            Self::Endpoint(_) => wire::Attachment::Endpoint,
            Self::Notification(_) => wire::Attachment::Notification,
        }
    }
}
#[derive(Default)]
pub(super) struct Incoming {
    frame: Option<Frame>,
    grant: Option<(u64, Grant)>,
    pub deadline: Option<u64>,
}
impl Incoming {
    pub fn validate_replies(&self, pending: &[u64]) -> Result<(), Error> {
        if self
            .grant
            .as_ref()
            .is_some_and(|(id, _)| !pending.contains(id))
            || self
                .frame
                .as_ref()
                .is_some_and(|frame| !pending.contains(&frame.id))
        {
            return Err(Error(abi::IO_ERROR as i64));
        }
        Ok(())
    }
    pub fn transfer(&mut self, input: Handle) -> Result<(), Error> {
        let mut bytes = [0; 64];
        let received = match input.try_receive_memory(&mut bytes) {
            Ok((n, object)) => Ok((n, object.map(Grant::Memory))),
            Err(Error(c)) if c == abi::TOO_SMALL as i64 => input
                .try_receive_with(&mut bytes)
                .map(|(n, endpoint)| (n, endpoint.map(|h| Grant::Endpoint(Endpoint::received(h)))))
                .or_else(|error| {
                    if error.0 != abi::TOO_SMALL as i64 {
                        return Err(error);
                    }
                    N::try_accept(input, &mut bytes).map(|(n, notification)| {
                        (
                            n,
                            notification
                                .map(|h| Grant::Notification(NotificationGrant::received(h))),
                        )
                    })
                }),
            Err(error) => Err(error),
        };
        match received {
            Ok((n, grant)) => {
                let grant = grant.ok_or(Error(abi::IO_ERROR as i64))?;
                let id = wire::grant_id(&bytes[..n]).ok_or(Error(abi::IO_ERROR as i64))?;
                if self.grant.is_some() {
                    return Err(Error(abi::IO_ERROR as i64));
                }
                self.grant = Some((id, grant));
                self.arm()?;
                Ok(())
            }
            Err(Error(c)) if c == abi::WOULD_BLOCK as i64 || c == abi::TIMED_OUT as i64 => Ok(()),
            Err(e) => Err(e),
        }
    }
    fn arm(&mut self) -> Result<(), Error> {
        if self.deadline.is_none() {
            self.deadline = Some(time::after(300)?);
        }
        Ok(())
    }
    pub fn receive(&mut self, mailbox: &mut Mailbox) -> Result<Option<Message>, Error> {
        if self.frame.is_none() {
            let mut frame = [0; wire::FRAME_BYTES];
            if let Some(n) = mailbox.try_receive(&mut frame)? {
                let (id, payload, attachment) =
                    wire::decode(&frame[..n]).ok_or(Error(abi::IO_ERROR as i64))?;
                let mut bytes = [0; 64];
                bytes[..payload.len()].copy_from_slice(payload);
                self.frame = Some(Frame {
                    id,
                    bytes,
                    len: payload.len(),
                    attachment,
                });
                self.arm()?;
            }
        }
        if let Some(frame) = &self.frame {
            if self
                .grant
                .as_ref()
                .is_some_and(|(id, grant)| grant.kind() != frame.attachment || *id != frame.id)
            {
                return Err(Error(abi::IO_ERROR as i64));
            }
            if frame.attachment == wire::Attachment::None || self.grant.is_some() {
                let frame = self.frame.take().unwrap();
                let (object, endpoint, notification) = match self.grant.take() {
                    None => (None, None, None),
                    Some((_, Grant::Memory(object))) => (Some(object), None, None),
                    Some((_, Grant::Endpoint(endpoint))) => (None, Some(endpoint), None),
                    Some((_, Grant::Notification(notification))) => {
                        (None, None, Some(notification))
                    }
                };
                self.deadline = None;
                return Ok(Some(Message {
                    id: frame.id,
                    bytes: frame.bytes,
                    len: frame.len,
                    object,
                    endpoint,
                    notification,
                }));
            }
        }
        if let Some(deadline) = self.deadline
            && time::reached(time::now()?, deadline)
        {
            return Err(Error(abi::TIMED_OUT as i64));
        }
        Ok(None)
    }
}
