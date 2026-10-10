//! Trap-side validation and queueing only. Page-table edits and allocation run on the boot stack.
use super::Session;
use crate::scheduler::Event;
use cathedral_contracts::{memory as wire, user as abi};
pub(super) fn syscall(
    session: &mut Session,
    slot: usize,
    [number, first, second, third]: [u64; 4],
    event: &mut Event,
) -> Result<u64, u64> {
    if third != 0 || (!matches!(number, abi::MEMORY_SEAL | abi::MEMORY_MAP) && second != 0) {
        return Err(abi::INVALID_ARGUMENT);
    }
    if number == abi::MEMORY_ADDRESS || number == abi::MEMORY_PAGES {
        let index = session.memory.model.access(slot, first)?;
        return Ok(if number == abi::MEMORY_ADDRESS {
            wire::address(index)
        } else {
            session.memory.model.entries[index].pages as u64
        });
    }
    if number == abi::MEMORY_ALLOCATE && !(1..=wire::MAX_PAGES as u64).contains(&first) {
        return Err(abi::INVALID_ARGUMENT);
    }
    assert!(session.tasks[slot].memory_call.is_none());
    session.tasks[slot].memory_call = Some([number, first, second]);
    *event = Event::Block;
    Ok(0)
}
