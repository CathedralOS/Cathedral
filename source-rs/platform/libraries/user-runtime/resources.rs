//! Inspect this account or subdivide unused capacity through an owned launch grant.
use crate::{Error, abi, arch, task::Launch};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub private_pages: u64,
    pub reader_pages: u64,
    pub channels: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    pub limits: Limits,
    pub used: Limits,
}
#[repr(align(64))]
struct Record([u64; 6]);
pub fn budget() -> Result<Budget, Error> {
    let mut record = Record([0; 6]);
    result(arch::call(
        abi::RESOURCE_BUDGET,
        record.0.as_mut_ptr() as u64,
        0,
    ))?;
    let [
        private_pages,
        reader_pages,
        channels,
        pages,
        readers,
        used_channels,
    ] = record.0;
    Ok(Budget {
        limits: Limits {
            private_pages,
            reader_pages,
            channels,
        },
        used: Limits {
            private_pages: pages,
            reader_pages: readers,
            channels: used_channels,
        },
    })
}
pub fn assign(child: Launch, limits: Limits) -> Result<(), Error> {
    let record = Record([
        limits.private_pages,
        limits.reader_pages,
        limits.channels,
        0,
        0,
        0,
    ]);
    result(arch::call3(
        abi::RESOURCE_ASSIGN,
        child.raw(),
        record.0.as_ptr() as u64,
        0,
    ))
}
fn result(value: u64) -> Result<(), Error> {
    if (value as i64) < 0 {
        Err(Error(value as i64))
    } else {
        Ok(())
    }
}

/// Only this account's reservation and live charges, including orphaned backing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalBudget {
    pub reserved: u64,
    pub charged: u64,
}
pub fn physical_budget() -> Result<PhysicalBudget, Error> {
    let mut record = Record([0; 6]);
    result(arch::call(
        abi::PHYSICAL_BUDGET,
        record.0.as_mut_ptr() as u64,
        0,
    ))?;
    Ok(PhysicalBudget {
        reserved: record.0[0],
        charged: record.0[1],
    })
}
