//! Scan durable pairs, choose the newest complete catalog, initialize only blank media.
//! Sector buffers and initialization use separate frames to fit the bounded guest stack.
use super::{
    catalog::{Catalog, Entry},
    format,
};
use cathedral_contracts::{block::Device, user as abi};
pub(super) fn open(disk: &mut impl Device) -> Result<(u64, Catalog), u64> {
    if disk.sectors() < 4 {
        return Err(abi::IO_ERROR);
    }
    let mut chosen = None;
    let mut blank = true;
    for slot in 0..2 {
        scan(disk, slot, &mut chosen, &mut blank)?;
    }
    match chosen {
        Some(value) => Ok(value),
        None if blank => initialize(disk),
        None => Err(abi::IO_ERROR),
    }
}
fn scan(
    disk: &mut impl Device,
    slot: u64,
    chosen: &mut Option<(u64, Catalog)>,
    blank: &mut bool,
) -> Result<(), u64> {
    let mut body = [0; 512];
    let mut commit = [0; 512];
    disk.read(slot * 2, &mut body)?;
    disk.read(slot * 2 + 1, &mut commit)?;
    *blank &= body.iter().chain(commit.iter()).all(|&byte| byte == 0);
    if let Some(record) = format::decode(&body, &commit) {
        if let Some((_, old)) = chosen.as_ref() {
            if old.sequence == record.sequence {
                return Err(abi::IO_ERROR);
            }
            if old.sequence > record.sequence {
                return Ok(());
            }
        }
        *chosen = Some((slot, record));
    }
    Ok(())
}
fn initialize(disk: &mut impl Device) -> Result<(u64, Catalog), u64> {
    let mut empty = Catalog::default();
    empty.roots[0].entries[0] = Entry {
        present: true,
        ..Entry::default()
    };
    let mut body = [0; 512];
    let mut commit = [0; 512];
    format::encode(&empty, &mut body, &mut commit);
    disk.write(2, &body)?;
    disk.flush()?;
    disk.write(3, &commit)?;
    disk.flush()?;
    Ok((1, empty))
}
