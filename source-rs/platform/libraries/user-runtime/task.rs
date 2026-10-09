//! Bounded launch and observation mechanisms; applications choose restart policy.
use crate::{Error, abi, arch, ipc::Handle};

#[derive(Clone, Copy, Debug)]
pub struct Launch(u64);
#[derive(Clone, Copy, Debug)]
pub struct Child(u64);
#[derive(Clone, Copy, Debug)]
pub struct Port(u64);
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Returned(u64),
    Fault {
        vector: u64,
        error: u64,
        address: u64,
        instruction: u64,
    },
    Cancelled,
}
impl Launch {
    pub fn bootstrap() -> Result<Self, Error> {
        result(arch::call(abi::TASK_LAUNCH, 0, 0)).map(Self)
    }
    pub fn from_raw(bits: u64) -> Self {
        Self(bits)
    }
    pub fn raw(self) -> u64 {
        self.0
    }
    pub fn spawn(self, argument: u64) -> Result<Child, Error> {
        result(arch::call(abi::TASK_SPAWN, self.0, argument)).map(Child)
    }
}
impl Child {
    pub fn from_raw(bits: u64) -> Self {
        Self(bits)
    }
    pub fn raw(self) -> u64 {
        self.0
    }
    /// Returns only after address-space reclamation. Success consumes the outcome;
    /// copied handles cannot wait twice or observe a later child in the same slot.
    pub fn wait(self) -> Result<Outcome, Error> {
        self.collect(abi::TASK_WAIT, abi::EXIT_BYTES as u64)
    }
    /// An expired wait leaves the child alive and its eventual outcome collectible.
    pub fn wait_until(self, deadline: u64) -> Result<Outcome, Error> {
        self.collect(abi::TASK_WAIT_UNTIL, deadline)
    }
    /// Stop and reclaim this exact child before returning; does not consume its
    /// outcome. Cancelling an already-reaped child preserves its original status.
    pub fn cancel(self) -> Result<(), Error> {
        result(arch::call(abi::TASK_CANCEL, self.0, 0)).map(|_| ())
    }
    fn collect(self, number: u64, third: u64) -> Result<Outcome, Error> {
        let mut record = Record([0; abi::EXIT_BYTES]);
        result(arch::call3(
            number,
            self.0,
            record.0.as_mut_ptr() as u64,
            third,
        ))?;
        let mut words = [0; 5];
        for (word, bytes) in words.iter_mut().zip(record.0.chunks_exact(8)) {
            *word = u64::from_le_bytes(bytes.try_into().unwrap());
        }
        match words[0] {
            abi::EXIT_RETURNED => Ok(Outcome::Returned(words[1])),
            abi::EXIT_FAULT => Ok(Outcome::Fault {
                vector: words[1],
                error: words[2],
                address: words[3],
                instruction: words[4],
            }),
            abi::EXIT_CANCELLED => Ok(Outcome::Cancelled),
            _ => Err(Error(abi::IO_ERROR as i64)),
        }
    }
}
impl Port {
    pub fn bootstrap() -> Result<Self, Error> {
        result(arch::call(abi::TASK_PORT, 0, 0)).map(Self)
    }
    /// Explicitly accept this service instance's grants: request sender, reply receiver.
    pub fn connect(self) -> Result<(Handle, Handle), Error> {
        let first = result(arch::call(abi::TASK_CONNECT, self.0, 0))?;
        Ok((Handle::bootstrap(first)?, Handle::bootstrap(first + 1)?))
    }
}
#[repr(align(64))]
struct Record([u8; abi::EXIT_BYTES]);
fn result(value: u64) -> Result<u64, Error> {
    if (value as i64) < 0 {
        Err(Error(value as i64))
    } else {
        Ok(value)
    }
}
