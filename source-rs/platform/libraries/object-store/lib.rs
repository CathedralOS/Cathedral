#![no_std]
#![forbid(unsafe_code)]
//! One private object, two alternating body/commit pairs. No in-place live update.
use cathedral_contracts::{
    block::Device,
    storage::{CAPACITY, OBJECT, Record},
    user as abi,
};
mod format;
pub struct Store<D> {
    disk: D,
    record: Record,
    slot: u64,
    healthy: bool,
}
impl<D: Device> Store<D> {
    pub fn open(mut disk: D) -> Result<Self, u64> {
        if disk.sectors() < 4 {
            return Err(abi::IO_ERROR);
        }
        let mut chosen: Option<(u64, Record)> = None;
        let mut blank = true;
        for slot in 0..2 {
            let mut body = [0; 512];
            let mut commit = [0; 512];
            disk.read(slot * 2, &mut body)?;
            disk.read(slot * 2 + 1, &mut commit)?;
            blank &= body.iter().chain(commit.iter()).all(|&byte| byte == 0);
            if let Some(record) = format::decode(&body, &commit) {
                if let Some((_, old)) = chosen {
                    if old.generation == record.generation {
                        return Err(abi::IO_ERROR);
                    }
                    if old.generation > record.generation {
                        continue;
                    }
                }
                chosen = Some((slot, record));
            }
        }
        let (slot, record) = match chosen {
            Some(value) => value,
            None if blank => {
                let empty = Record::default();
                let (body, commit) = format::encode(empty);
                disk.write(2, &body)?;
                disk.flush()?;
                disk.write(3, &commit)?;
                disk.flush()?;
                (1, empty)
            }
            None => return Err(abi::IO_ERROR),
        };
        Ok(Self {
            disk,
            record,
            slot,
            healthy: true,
        })
    }
    pub fn needs_reopen(&self) -> bool {
        !self.healthy
    }
    pub fn read(&self, object: u64) -> Result<Record, u64> {
        if !self.healthy {
            Err(abi::IO_ERROR)
        } else if object == OBJECT {
            Ok(self.record)
        } else {
            Err(abi::DENIED)
        }
    }
    /// A reply may be lost after durable commit. Callers must read/reconcile before retry.
    /// Hook phases: before body, body durable, commit written, commit durable.
    pub fn replace(
        &mut self,
        object: u64,
        expected: u64,
        payload: &[u8],
        mut hook: impl FnMut(u8),
    ) -> Result<Record, u64> {
        if !self.healthy {
            return Err(abi::IO_ERROR);
        }
        if object != OBJECT {
            return Err(abi::DENIED);
        }
        if payload.len() > CAPACITY {
            return Err(abi::INVALID_ARGUMENT);
        }
        if expected != self.record.generation {
            return Err(abi::WOULD_BLOCK);
        }
        let mut record = Record {
            generation: expected.checked_add(1).ok_or(abi::IO_ERROR)?,
            length: payload.len(),
            bytes: [0; CAPACITY],
        };
        record.bytes[..payload.len()].copy_from_slice(payload);
        let (body, commit) = format::encode(record);
        let slot = 1 - self.slot;
        self.healthy = false;
        hook(1);
        self.disk.write(slot * 2, &body)?;
        self.disk.flush()?;
        hook(2);
        self.disk.write(slot * 2 + 1, &commit)?;
        hook(3);
        self.disk.flush()?;
        hook(4);
        self.slot = slot;
        self.record = record;
        self.healthy = true;
        Ok(record)
    }
}
#[cfg(test)]
mod tests;
