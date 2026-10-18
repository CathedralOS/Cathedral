//! Install programs into a formatted image as objects of root 0, with exactly
//! the records the guest store would write: chunks and manifest shared by
//! digest, a head, one event per revision, an edge from the parent, the next
//! number and the quota ledger. Programs are numbered in the order given, so a
//! profile's program list is its catalogue. Seeding an image that already holds
//! the programs republishes only those whose bytes changed, as a new revision;
//! the guest collects the superseded history before its next write.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use redb::{ReadableTable, Table};
use std::collections::BTreeMap;

const ROOT: u8 = 0;
const HEAD_BYTES: usize = 56;

pub struct Program {
    pub parent: u64,
    pub bytes: Vec<u8>,
}

pub(super) fn key(kind: u8, suffix: &[u8]) -> Vec<u8> {
    let mut key = vec![ROOT, kind];
    key.extend_from_slice(suffix);
    key
}
pub(super) fn head(revision: u64, manifest: &[u8], len: usize, parent: u64) -> Vec<u8> {
    let mut head = revision.to_le_bytes().to_vec();
    head.extend_from_slice(manifest);
    head.extend_from_slice(&(len as u64).to_le_bytes());
    head.extend_from_slice(&parent.to_le_bytes());
    head
}
pub(super) fn event(object: u64, revision: u64) -> Vec<u8> {
    let mut suffix = object.to_be_bytes().to_vec();
    suffix.extend_from_slice(&revision.to_be_bytes());
    key(b'e', &suffix)
}
fn edge(parent: u64, child: u64) -> Vec<u8> {
    let mut suffix = parent.to_be_bytes().to_vec();
    suffix.extend_from_slice(&child.to_be_bytes());
    key(b't', &suffix)
}
fn word(table: &Table<&[u8], &[u8]>, key: &[u8]) -> Result<Option<u64>> {
    Ok(match table.get(key)? {
        None => None,
        Some(value) => Some(u64::from_le_bytes(value.value().try_into()?)),
    })
}
/// Insert a record and charge it, unless an identical one is already retained.
pub(super) fn retain(table: &mut Table<&[u8], &[u8]>, charged: &mut u64, key: &[u8], value: &[u8]) -> Result<()> {
    if let Some(existing) = table.get(key)? {
        if existing.value() != value {
            return Err("retained record differs from the program's".into());
        }
        return Ok(());
    }
    *charged += (key.len() + value.len()) as u64;
    table.insert(key, value)?;
    Ok(())
}
/// Programs become objects 1..=n in order; each parent is zero or an earlier object.
pub fn seed(path: &Path, offset: u64, bytes: u64, programs: &[Program]) -> Result<()> {
    let db = open(Region::new(Arc::new(Image::open(path)?), offset, bytes)?)?;
    let mut tx = db.begin_write()?;
    tx.set_durability(redb::Durability::Immediate)?;
    let mut installed = 0;
    let mut updated = 0;
    {
        let mut table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))?;
        let quota = key(b'q', &[]);
        let next = key(b'n', &[]);
        let mut charged = word(&table, &quota)?.unwrap_or(2 + 8);
        let mut number = word(&table, &next)?.unwrap_or(1);
        let mut store = Store::new(Policy {
            chunk_bytes: 4096,
            inline_limit: 128,
            layout: Layout::Flat,
        });
        for (index, program) in programs.iter().enumerate() {
            let object = index as u64 + 1;
            if program.parent >= object {
                return Err("a program's parent must be an earlier program or the root".into());
            }
            let version = store.put(&program.bytes);
            let records = store.records(&version);
            let manifest = records[b"head".as_slice()].clone();
            let current = table.get(key(b'h', &object.to_be_bytes()).as_slice())?.map(|v| v.value().to_vec());
            let revision = match &current {
                None => {
                    if number != object {
                        return Err(format!("object {object} is missing but numbers run to {number}").into());
                    }
                    number += 1;
                    1
                }
                Some(current) => {
                    if current.len() != HEAD_BYTES {
                        return Err("retained head has an unexpected shape".into());
                    }
                    if u64::from_le_bytes(current[48..].try_into()?) != program.parent {
                        return Err(format!("program {object} is installed under another parent").into());
                    }
                    if current[8..40] == manifest[..] {
                        continue; // Same bytes: nothing to republish.
                    }
                    if !is_program(&table, &current[8..40])? {
                        return Err(format!(
                            "object {object} holds something other than a program; refusing to overwrite incompatible desktop data. Move this image and its .programs sidecar (if present) to backup paths, then relaunch to create a fresh desktop"
                        )
                        .into());
                    }
                    u64::from_le_bytes(current[..8].try_into()?) + 1
                }
            };
            for (k, v) in &records {
                if k != b"head" {
                    retain(&mut table, &mut charged, &key(k[0], &k[1..]), v)?;
                    if k[0] == b'c' {
                        retain(&mut table, &mut charged, &key(b'k', &k[1..]), &[])?;
                    }
                }
            }
            let head = head(revision, &manifest, program.bytes.len(), program.parent);
            retain(&mut table, &mut charged, &event(object, revision), &head)?;
            let head_key = key(b'h', &object.to_be_bytes());
            if current.is_none() {
                charged += (head_key.len() + head.len()) as u64;
                retain(&mut table, &mut charged, &edge(program.parent, object), &[])?;
                installed += 1;
            } else {
                updated += 1;
            }
            table.insert(head_key.as_slice(), head.as_slice())?;
        }
        if word(&table, &next)?.is_none() {
            charged += (next.len() + 8) as u64;
        }
        table.insert(next.as_slice(), number.to_le_bytes().as_slice())?;
        table.insert(quota.as_slice(), charged.to_le_bytes().as_slice())?;
    }
    tx.commit()?;
    println!("SEEDED {installed} new and {updated} updated program objects in root {ROOT}");
    Ok(())
}
/// Whether a retained object's content begins like an ELF, read through its
/// manifest: inline bytes, or the first chunk. Installation only ever
/// replaces programs.
fn is_program(table: &Table<&[u8], &[u8]>, manifest: &[u8]) -> Result<bool> {
    let record = table.get(key(b'm', manifest).as_slice())?.ok_or("retained head names a missing manifest")?;
    let encoded = record.value().to_vec();
    drop(record);
    let head = match encoded.first() {
        Some(0) => encoded.get(17..21).map(<[u8]>::to_vec),
        Some(1) => {
            let digest = encoded.get(17..49).ok_or("manifest shorter than one chunk digest")?;
            table.get(key(b'c', digest).as_slice())?.map(|chunk| chunk.value()[..4.min(chunk.value().len())].to_vec())
        }
        _ => None,
    };
    Ok(head.as_deref() == Some(b"ELF".as_slice()))
}
/// The records a seeded set of programs leaves behind on a fresh root, for oracles.
pub fn expected(programs: &[Program]) -> BTreeMap<Vec<u8>, Vec<u8>> {
    let mut store = Store::new(Policy {
        chunk_bytes: 4096,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    let mut expected = BTreeMap::new();
    for (index, program) in programs.iter().enumerate() {
        let object = index as u64 + 1;
        let version = store.put(&program.bytes);
        let records = store.records(&version);
        let manifest = records[b"head".as_slice()].clone();
        for (k, v) in records {
            if k != b"head" {
                if k[0] == b'c' {
                    expected.insert(key(b'k', &k[1..]), Vec::new());
                }
                expected.insert(key(k[0], &k[1..]), v);
            }
        }
        let head = head(1, &manifest, program.bytes.len(), program.parent);
        expected.insert(key(b'h', &object.to_be_bytes()), head.clone());
        expected.insert(event(object, 1), head);
        expected.insert(edge(program.parent, object), Vec::new());
    }
    expected
}
pub fn load(args: &[String]) -> Result<Vec<Program>> {
    if !args.len().is_multiple_of(2) {
        return Err("programs are PARENT FILE pairs".into());
    }
    args.chunks(2)
        .map(|pair| {
            Ok(Program {
                parent: pair[0].parse()?,
                bytes: std::fs::read(&pair[1])?,
            })
        })
        .collect()
}
