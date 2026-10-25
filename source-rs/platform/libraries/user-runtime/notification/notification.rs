// Kernel wake authority, not message delivery. Publish/acquire application data
// in its own protocol; a wake only asks the receiver to check that protocol.
use crate::{Error, abi, arch};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Notification(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Signaled,
    TimedOut,
    Closed,
    Revoked,
    Cancelled,
    /// Preserve the committed phase if a newer kernel adds an outcome.
    Unknown(u64),
}
#[repr(align(128))]
struct Record([u8; abi::IPC_PACKET_BYTES]);
impl Notification {
    /// Three separate rights: wait, signal, control. Each consumes a local handle.
    /// One object consumes the same funded transport budget used by channels.
    pub fn create() -> Result<[Self; 3], Error> {
        let mut record = Record([0; abi::IPC_PACKET_BYTES]);
        result(arch::call3(
            abi::NOTIFY_CREATE,
            record.0.as_mut_ptr() as u64,
            0,
            0,
        ))?;
        Ok(core::array::from_fn(|i| {
            Self(word(&record.0[i * 8..i * 8 + 8]))
        }))
    }
    /// Names are process-local, typed by this ABI. Bytes alone never install a grant.
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
    pub fn raw(self) -> u64 {
        self.0
    }
    /// Drop this grant, including all copied Rust aliases, without closing peers' object.
    pub fn release(self) -> Result<(), Error> {
        result(arch::call3(abi::NOTIFY_DROP, self.0, 0, 0)).map(|_| ())
    }
    pub fn signal(self, all: bool) -> Result<(), Error> {
        result(arch::call3(abi::NOTIFY_SIGNAL, self.0, u64::from(all), 0)).map(|_| ())
    }
    pub fn wait(self, deadline: Option<u64>) -> Result<Outcome, Error> {
        outcome(arch::call3(
            abi::NOTIFY_WAIT,
            self.0,
            deadline.unwrap_or(0),
            u64::from(deadline.is_some()),
        ))
    }
    /// Err means no signal committed. Ok(outcome) means it did, even when the
    /// subsequent wait timed out, was cancelled, closed or lost its authority.
    pub fn signal_and_wait(self, wait: Self, deadline: Option<u64>) -> Result<Outcome, Error> {
        let mut record = Record([0; abi::IPC_PACKET_BYTES]);
        for (value, bytes) in [
            self.0,
            wait.0,
            deadline.unwrap_or(0),
            u64::from(deadline.is_some()),
        ]
        .into_iter()
        .zip(record.0.chunks_exact_mut(8))
        {
            bytes.copy_from_slice(&value.to_le_bytes());
        }
        outcome(arch::call3(
            abi::NOTIFY_SIGNAL_WAIT,
            record.0.as_ptr() as u64,
            0,
            0,
        ))
    }
    pub fn close_object(self) -> Result<(), Error> {
        self.control(abi::NOTIFY_CLOSE)
    }
    /// Irreversibly withdraw all wait grants; signal/control rights stay independent.
    pub fn withdraw_waits(self) -> Result<(), Error> {
        self.control(abi::NOTIFY_WITHDRAW_WAITS)
    }
    /// Cancel currently registered waits. Future waits and pending signals survive.
    pub fn cancel_waits(self) -> Result<(), Error> {
        self.control(abi::NOTIFY_CANCEL_WAITS)
    }
    fn control(self, action: u64) -> Result<(), Error> {
        result(arch::call3(abi::NOTIFY_CONTROL, self.0, action, 0)).map(|_| ())
    }
}
/// Shared transport reservation, charged notifications, local handle limit/use.
pub fn budget() -> Result<[u64; 4], Error> {
    let mut record = Record([0; abi::IPC_PACKET_BYTES]);
    result(arch::call3(
        abi::NOTIFY_BUDGET,
        record.0.as_mut_ptr() as u64,
        0,
        0,
    ))?;
    Ok(core::array::from_fn(|i| word(&record.0[i * 8..i * 8 + 8])))
}
fn word(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().unwrap())
}
fn result(value: u64) -> Result<u64, Error> {
    if (value as i64) < 0 {
        Err(Error(value as i64))
    } else {
        Ok(value)
    }
}
fn outcome(value: u64) -> Result<Outcome, Error> {
    match result(value)? {
        abi::NOTIFY_SIGNALED => Ok(Outcome::Signaled),
        abi::NOTIFY_TIMED_OUT => Ok(Outcome::TimedOut),
        abi::NOTIFY_CLOSED => Ok(Outcome::Closed),
        abi::NOTIFY_REVOKED => Ok(Outcome::Revoked),
        abi::NOTIFY_CANCELLED => Ok(Outcome::Cancelled),
        other => Ok(Outcome::Unknown(other)),
    }
}
