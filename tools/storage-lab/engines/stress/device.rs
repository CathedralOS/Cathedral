//! Deterministic storage model: reads see volatile bytes; a successful sync makes
//! all preceding mutations durable. Power cuts freeze the device before cleanup.
use redb::StorageBackend;
use std::{
    io,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug)]
pub enum Cut {
    Before,
    Partial,
    After,
}

#[derive(Clone, Debug)]
pub struct Operation {
    pub kind: &'static str,
    pub offset: usize,
    pub bytes: usize,
}

#[derive(Debug)]
struct State {
    volatile: Vec<u8>,
    durable: Vec<u8>,
    operations: Vec<Operation>,
    fault: Option<(usize, Cut)>,
    dead: bool,
    limit: usize,
    denied: usize,
}

#[derive(Clone, Debug)]
pub struct Device(Arc<Mutex<State>>);

impl Device {
    pub fn new(image: Vec<u8>) -> Self {
        Self(Arc::new(Mutex::new(State {
            volatile: image.clone(),
            durable: image,
            operations: Vec::new(),
            fault: None,
            dead: false,
            limit: usize::MAX,
            denied: 0,
        })))
    }
    pub fn arm(&self, operation: usize, cut: Cut) {
        let mut s = self.0.lock().unwrap();
        s.operations.clear();
        s.fault = Some((operation, cut));
    }
    pub fn operations(&self) -> Vec<Operation> {
        self.0.lock().unwrap().operations.clone()
    }
    pub fn reset(&self) {
        self.0.lock().unwrap().operations.clear();
    }
    pub fn limit(&self, bytes: usize) {
        self.0.lock().unwrap().limit = bytes;
    }
    pub fn denied(&self) -> usize {
        self.0.lock().unwrap().denied
    }
    pub fn extent(&self) -> usize {
        self.0.lock().unwrap().volatile.len()
    }
    pub fn crashed(&self) -> bool {
        self.0.lock().unwrap().dead
    }
    pub fn crash(&self) -> Vec<u8> {
        let mut s = self.0.lock().unwrap();
        s.dead = true;
        s.durable.clone()
    }
    fn mutate(&self, operation: Operation, data: &[u8]) -> io::Result<()> {
        let mut s = self.0.lock().unwrap();
        if s.dead {
            return Err(io::Error::other("device powered off"));
        }
        s.operations.push(operation.clone());
        let cut = s
            .fault
            .filter(|(n, _)| *n == s.operations.len())
            .map(|(_, c)| c);
        if matches!(cut, Some(Cut::Before)) {
            s.dead = true;
            return Err(io::Error::other("cut before operation"));
        }
        let end = operation
            .offset
            .checked_add(operation.bytes)
            .ok_or_else(|| io::Error::other("offset overflow"))?;
        if operation.kind != "sync" && end > s.limit {
            s.denied += 1;
            return Err(io::Error::other("device capacity exhausted"));
        }
        match operation.kind {
            "resize" => s.volatile.resize(end, 0),
            "write" => {
                if end > s.volatile.len() {
                    return Err(io::Error::other("write beyond extent"));
                }
                s.volatile[operation.offset..end].copy_from_slice(data);
            }
            "sync" => {}
            _ => unreachable!(),
        }
        match cut {
            None => {
                if operation.kind == "sync" {
                    s.durable = s.volatile.clone();
                }
                Ok(())
            }
            Some(cut) => {
                // Unflushed writes may reach media independently. A partial write
                // persists a byte prefix; a partial sync persists a file prefix.
                // Earlier successful barriers are always preserved.
                let count = if matches!(cut, Cut::Partial) {
                    operation.bytes / 2
                } else {
                    operation.bytes
                };
                match operation.kind {
                    "write" => {
                        if s.durable.len() < operation.offset + count {
                            s.durable.resize(operation.offset + count, 0);
                        }
                        s.durable[operation.offset..operation.offset + count]
                            .copy_from_slice(&data[..count]);
                    }
                    "resize" if matches!(cut, Cut::After) => s.durable.resize(end, 0),
                    "resize" => {}
                    "sync" => {
                        let count = if matches!(cut, Cut::Partial) {
                            s.volatile.len() / 2
                        } else {
                            s.volatile.len()
                        };
                        let bytes = s.volatile[..count].to_vec();
                        if s.durable.len() < count {
                            s.durable.resize(count, 0);
                        }
                        s.durable[..count].copy_from_slice(&bytes);
                        if matches!(cut, Cut::After) {
                            s.durable.truncate(count);
                        }
                    }
                    _ => unreachable!(),
                }
                s.dead = true;
                Err(io::Error::other("cut during/after operation"))
            }
        }
    }
}

impl StorageBackend for Device {
    fn len(&self) -> io::Result<u64> {
        Ok(self.0.lock().unwrap().volatile.len() as u64)
    }
    fn read(&self, offset: u64, out: &mut [u8]) -> io::Result<()> {
        let s = self.0.lock().unwrap();
        if s.dead {
            return Err(io::Error::other("device powered off"));
        }
        let start = usize::try_from(offset).map_err(io::Error::other)?;
        let end = start
            .checked_add(out.len())
            .ok_or_else(|| io::Error::other("offset overflow"))?;
        out.copy_from_slice(
            s.volatile
                .get(start..end)
                .ok_or_else(|| io::Error::other("read beyond extent"))?,
        );
        Ok(())
    }
    fn set_len(&self, len: u64) -> io::Result<()> {
        self.mutate(
            Operation {
                kind: "resize",
                offset: 0,
                bytes: usize::try_from(len).map_err(io::Error::other)?,
            },
            &[],
        )
    }
    fn write(&self, offset: u64, data: &[u8]) -> io::Result<()> {
        self.mutate(
            Operation {
                kind: "write",
                offset: usize::try_from(offset).map_err(io::Error::other)?,
                bytes: data.len(),
            },
            data,
        )
    }
    fn sync_data(&self) -> io::Result<()> {
        self.mutate(
            Operation {
                kind: "sync",
                offset: 0,
                bytes: 0,
            },
            &[],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_sync_may_persist_none_a_prefix_or_all() {
        for (cut, expected) in [
            (Cut::Before, vec![0; 8]),
            (Cut::Partial, vec![1, 1, 1, 1, 0, 0, 0, 0]),
            (Cut::After, vec![1; 8]),
        ] {
            let device = Device::new(vec![0; 8]);
            device.write(0, &[1; 8]).unwrap();
            device.arm(1, cut);
            assert!(device.sync_data().is_err());
            assert_eq!(device.crash(), expected);
        }
    }
    #[test]
    fn successful_barriers_survive_later_partial_writes_and_cleanup() {
        let device = Device::new(vec![0; 8]);
        device.write(0, &[1; 8]).unwrap();
        device.sync_data().unwrap();
        device.arm(1, Cut::Partial);
        assert!(device.write(2, &[2; 4]).is_err());
        assert!(device.sync_data().is_err());
        assert_eq!(device.crash(), vec![1, 1, 2, 2, 1, 1, 1, 1]);
    }
    #[test]
    fn unflushed_bytes_disappear_and_capacity_refusal_does_not_mutate() {
        let device = Device::new(vec![0; 4]);
        device.limit(4);
        assert!(device.set_len(5).is_err());
        assert_eq!(device.extent(), 4);
        device.write(0, &[1; 4]).unwrap();
        assert_eq!(device.crash(), vec![0; 4]);
    }
}
