//! Omega selects child-control operations; the host retains resources and exact replies.
use cathedral_desktop_matrix_contracts::matrix as wire;
use cathedral_kernel_contracts::{memory, user as abi};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Unhandled,
    Deny,
    Busy,
    Invalid,
    Survey,
    Stored,
    Forward,
    Images,
    Fallback,
    Management,
    Configure,
    Restore,
    Imports,
    Pages,
    Memory,
    Policy,
    Placement,
    Spawn,
    Run,
    Stop,
}
#[derive(Default)]
pub struct Facts {
    pub owner: bool,
    pub loss: bool,
    pub launching: bool,
    pub storing: bool,
    pub occupied: u64,
    pub unavailable: u64,
    pub pending: u64,
    pub standing: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Decision {
    pub operation: Operation,
    pub slot: usize,
    pub deadline: u64,
}
#[inline(never)]
pub fn decide(words: wire::Packet, children: usize, f: Facts) -> Decision {
    let mut p = [0; 128];
    p[..16].copy_from_slice(&[
        15,
        4,
        f.owner.into(),
        f.loss.into(),
        f.launching.into(),
        f.storing.into(),
        f.occupied,
        f.unavailable,
        f.pending,
        f.standing,
        children as u64,
        abi::MAX_LAUNCHES as u64,
        memory::MAX_PRIVATE_PAGES as u64,
        memory::MAX_READER_PAGES as u64,
        cathedral_platform_contracts::imports::SUPPORTED,
        wire::MAX_SURFACES as u64,
    ]);
    p[16..24].copy_from_slice(&words);
    p[24] = wire::CONFIGURE_DEADLINE_TICKS;
    use Operation::*;
    let operation = match super::native::decide(&mut p) {
        0 => Unhandled,
        1 => Deny,
        2 => Busy,
        3 => Invalid,
        4 => Survey,
        5 => Stored,
        6 => Forward,
        7 => Images,
        8 => Fallback,
        9 => Management,
        10 => Configure,
        11 => Restore,
        12 => Imports,
        13 => Pages,
        14 => Memory,
        15 => Policy,
        16 => Placement,
        17 => Spawn,
        18 => Run,
        19 => Stop,
        _ => unreachable!("Omega child control operation"),
    };
    Decision {
        operation,
        slot: p[25] as usize,
        deadline: p[26],
    }
}
