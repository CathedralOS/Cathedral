//! Independently pending input hints. The owner signals; the child only waits.
//! Copied wakes remain explicit adapters for positional bring-up compositions.
use crate::{
    Error, abi,
    event::Interest,
    ipc::Handle,
    notification::{Notification, Outcome},
    time,
};
#[derive(Clone, Copy)]
pub enum Wake {
    Copied(Handle),
    Notification(Notification),
}
impl Wake {
    pub fn interest(self) -> Interest {
        match self {
            Self::Copied(h) => Interest::Ipc(h),
            Self::Notification(n) => Interest::Notification(n),
        }
    }
    /// Consume before polling state. A later signal stays pending across replies;
    /// readiness alone never consumes a hint or proves an input event exists.
    pub fn take(self) -> Result<bool, Error> {
        match self {
            Self::Copied(h) => match h.receive_until(&mut [0; 64], time::now()?) {
                Ok(_) => Ok(true),
                Err(Error(c)) if c == abi::TIMED_OUT as i64 => Ok(false),
                Err(error) => Err(error),
            },
            Self::Notification(n) => match n.wait(Some(time::now()?))? {
                Outcome::Signaled => Ok(true),
                Outcome::TimedOut => Ok(false),
                Outcome::Revoked => Err(Error(abi::REVOKED as i64)),
                _ => Err(Error(abi::PEER_CLOSED as i64)),
            },
        }
    }
}
/// Owner-retained signal and close authority, closed during child collection.
#[derive(Clone, Copy)]
pub enum WakeSource {
    Copied(Handle),
    Notification {
        signal: Notification,
        control: Notification,
    },
}
impl WakeSource {
    pub(crate) fn create() -> Result<(Self, Notification), Error> {
        let [wait, signal, control] = Notification::create()?;
        Ok((Self::Notification { signal, control }, wait))
    }
    pub fn signal(self) -> Result<(), Error> {
        match self {
            Self::Copied(h) => h.send(&[]),
            Self::Notification { signal, .. } => signal.signal(false),
        }
    }
    pub fn close(self) -> Result<(), Error> {
        match self {
            Self::Copied(h) => h.close(),
            Self::Notification { signal, control } => {
                control.close_object()?;
                signal.release()?;
                control.release()
            }
        }
    }
}
