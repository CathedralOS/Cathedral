//! Bounded root-local records and validation of a whole transaction before disk effects.
use cathedral_contracts::{
    storage::{self as wire, Record},
    user as abi,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    pub present: bool,
    pub length: usize,
    pub bytes: [u8; wire::CAPACITY],
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Root {
    pub generation: u64,
    pub entries: [Entry; wire::OBJECTS],
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Catalog {
    pub sequence: u64,
    pub roots: [Root; wire::ROOTS],
}
#[derive(Clone, Copy, Debug)]
pub enum Change {
    Create(u64, Entry),
    Replace(u64, Entry),
    Delete(u64),
}
impl Change {
    pub fn value(op: u64, object: u64, bytes: &[u8]) -> Result<Self, u64> {
        index(object)?;
        if bytes.len() > wire::CAPACITY {
            return Err(abi::INVALID_ARGUMENT);
        }
        let mut entry = Entry {
            present: true,
            length: bytes.len(),
            ..Entry::default()
        };
        entry.bytes[..bytes.len()].copy_from_slice(bytes);
        match op {
            wire::CREATE | wire::STAGE_CREATE => Ok(Self::Create(object, entry)),
            wire::REPLACE | wire::STAGE_REPLACE => Ok(Self::Replace(object, entry)),
            wire::DELETE | wire::STAGE_DELETE if bytes.is_empty() => Ok(Self::Delete(object)),
            _ => Err(abi::INVALID_ARGUMENT),
        }
    }
    pub fn object(self) -> u64 {
        match self {
            Self::Create(id, _) | Self::Replace(id, _) | Self::Delete(id) => id,
        }
    }
}
pub fn index(object: u64) -> Result<usize, u64> {
    if !(1..=wire::OBJECTS as u64).contains(&object) {
        return Err(abi::DENIED);
    }
    Ok(object as usize - 1)
}
impl Root {
    pub fn read(&self, object: u64) -> Result<Record, u64> {
        let entry = self.entries[index(object)?];
        if !entry.present {
            return Err(wire::NOT_FOUND);
        }
        Ok(Record {
            generation: self.generation,
            length: entry.length,
            bytes: entry.bytes,
        })
    }
    pub fn list(&self) -> Record {
        let mut record = Record {
            generation: self.generation,
            ..Record::default()
        };
        for (index, entry) in self.entries.iter().enumerate() {
            if entry.present {
                record.bytes[record.length..record.length + 8]
                    .copy_from_slice(&(index as u64 + 1).to_le_bytes());
                record.length += 8;
            }
        }
        record
    }
    pub fn changed(&self, expected: u64, changes: &[Change]) -> Result<Self, u64> {
        if self.generation != expected {
            return Err(abi::WOULD_BLOCK);
        }
        if changes.is_empty() || changes.len() > wire::CHANGES {
            return Err(abi::INVALID_ARGUMENT);
        }
        let mut next = *self;
        for (position, &change) in changes.iter().enumerate() {
            let object = change.object();
            if changes[..position].iter().any(|old| old.object() == object) {
                return Err(abi::INVALID_ARGUMENT);
            }
            let entry = &mut next.entries[index(object)?];
            match change {
                Change::Create(_, value) => {
                    if !value.present {
                        return Err(abi::INVALID_ARGUMENT);
                    }
                    if entry.present {
                        return Err(wire::EXISTS);
                    }
                    *entry = value;
                }
                Change::Replace(_, value) => {
                    if !value.present {
                        return Err(abi::INVALID_ARGUMENT);
                    }
                    if !entry.present {
                        return Err(wire::NOT_FOUND);
                    }
                    *entry = value;
                }
                Change::Delete(_) => {
                    if !entry.present {
                        return Err(wire::NOT_FOUND);
                    }
                    *entry = Entry::default();
                }
            }
            if entry.length > wire::CAPACITY
                || (!entry.present && entry.length != 0)
                || entry.bytes[entry.length..].iter().any(|&byte| byte != 0)
            {
                return Err(abi::INVALID_ARGUMENT);
            }
        }
        next.generation = expected.checked_add(1).ok_or(abi::IO_ERROR)?;
        Ok(next)
    }
}
