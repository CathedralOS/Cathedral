// Caller-local register authority and one outstanding interrupt per device.
// No hardware here: syscalls/devices/leased.rs couples these transitions to the board.
use cathedral_contracts::user as abi;

#[derive(Default, Clone, Copy)]
struct Grant {
    name: u64,
    registers: u64,
    armed: bool,
    pending: bool,
    waiting: bool,
}

#[derive(Default)]
pub(crate) struct Devices {
    grants: [Grant; abi::MAX_TASKS],
}

impl Devices {
    pub fn install(&mut self, slot: usize, name: u64, registers: u64) {
        assert_ne!(name, 0);
        assert_eq!(self.grants[slot].name, 0);
        self.grants[slot] = Grant {
            name,
            registers,
            ..Grant::default()
        };
    }
    pub fn handle(&self, slot: usize, index: u64) -> Result<u64, u64> {
        let name = self.grants[slot].name;
        if index != 0 || name == 0 {
            Err(abi::BAD_HANDLE)
        } else {
            Ok(name)
        }
    }
    pub fn check(&self, slot: usize, name: u64) -> Result<(), u64> {
        if name == 0 || self.grants[slot].name != name {
            Err(abi::BAD_HANDLE)
        } else {
            Ok(())
        }
    }
    pub fn register(&self, slot: usize, name: u64, offset: u64) -> Result<(), u64> {
        self.check(slot, name)?;
        if offset >= self.grants[slot].registers {
            Err(abi::INVALID_ARGUMENT)
        } else {
            Ok(())
        }
    }
    pub fn arm(&mut self, slot: usize, name: u64) -> Result<(), u64> {
        self.check(slot, name)?;
        let grant = &mut self.grants[slot];
        if grant.armed || grant.pending {
            return Err(abi::BUSY);
        }
        grant.armed = true;
        Ok(())
    }
    /// True asks the caller to block. A delivered notification leaves the route masked.
    pub fn wait(&mut self, slot: usize, name: u64, block: bool) -> Result<bool, u64> {
        self.check(slot, name)?;
        let grant = &mut self.grants[slot];
        if grant.pending {
            grant.pending = false;
            return Ok(false);
        }
        if !block {
            return Err(abi::WOULD_BLOCK);
        }
        grant.waiting = true;
        Ok(true)
    }
    /// The board masked and acknowledged this IRQ already. Return a waiter to wake.
    pub fn interrupt(&mut self, slot: usize) -> bool {
        let grant = &mut self.grants[slot];
        if grant.name == 0 || !grant.armed {
            return false;
        }
        grant.armed = false;
        if grant.waiting {
            grant.waiting = false;
            true
        } else {
            grant.pending = true;
            false
        }
    }
    pub fn close(&mut self, slot: usize) -> bool {
        let live = self.grants[slot].name != 0;
        self.grants[slot] = Grant::default();
        live
    }
}
