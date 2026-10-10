//! Boot-stack page-object operations. Traps enqueue; this owner maps, seals and reclaims.
mod allocation;
#[cfg(feature = "memory-lab")]
pub(super) mod rollback;
use super::{Frames, Session, next_epoch, syscalls};
use crate::{regions::Regions, scheduler::TaskState};
use cathedral_arch::StackFrames;
use cathedral_contracts::{memory as wire, user as abi};
pub(super) struct Memory {
    pub model: Regions,
    backing: [[u64; wire::MAX_PAGES]; wire::MAX_REGIONS],
}
impl Memory {
    pub fn new(tasks: usize, grants: &[wire::Grant]) -> Result<alloc::boxed::Box<Self>, u64> {
        let model = Regions::new(tasks, grants)?;
        // Keep the bounded arena off the boot stack: debug builds otherwise copy
        // it through each session preparation/result frame. Allocation is fallible.
        // SAFETY: Correct nonzero layout, null checked before initialization; Box
        // takes unique custody of the fully initialized allocation exactly once.
        unsafe {
            let pointer = alloc::alloc::alloc(core::alloc::Layout::new::<Self>()).cast::<Self>();
            if pointer.is_null() {
                return Err(abi::NO_MEMORY);
            }
            pointer.write(Self {
                model,
                backing: [[0; wire::MAX_PAGES]; wire::MAX_REGIONS],
            });
            Ok(alloc::boxed::Box::from_raw(pointer))
        }
    }
    pub fn frames(&self) -> usize {
        self.backing
            .iter()
            .flatten()
            .filter(|&&frame| frame != 0)
            .count()
    }
    unsafe fn reclaim(&mut self, index: usize, frames: &mut Frames<'_>) {
        if !self.model.entries[index].live() {
            for frame in &mut self.backing[index] {
                if *frame != 0 {
                    // SAFETY: All mappings were removed or never published; core holds unique backing custody.
                    unsafe {
                        frames.release(*frame);
                    }
                    *frame = 0;
                }
            }
        }
    }
}
/// # Safety
/// Sole CPU, boot stack, kernel root active, IRQs off; every user root is inactive.
pub(super) unsafe fn process(session: &mut Session, frames: &mut Frames<'_>) {
    for slot in 0..session.tasks.len() {
        let Some(call) = session.tasks[slot].memory_call.take() else {
            continue;
        };
        // Admission budgets are distinct from runtime page budgets enforced by Regions.
        frames.remaining = usize::MAX;
        session.tasks[slot].report.memory.deferred_calls += 1;
        // SAFETY: Same machine and ownership obligations as process.
        let result = unsafe { dispatch(session, frames, slot, call) };
        syscalls::complete(session, slot, result);
        session.scheduler.unblock(slot);
    }
}
unsafe fn dispatch(
    session: &mut Session,
    frames: &mut Frames<'_>,
    slot: usize,
    [number, first, second]: [u64; 3],
) -> Result<u64, u64> {
    match number {
        // SAFETY: Caller provides exclusive inactive roots and frame custody.
        abi::MEMORY_ALLOCATE => unsafe {
            allocation::allocate(session, frames, slot, first as usize)
        },
        abi::MEMORY_SEAL => {
            let peer = session
                .links
                .iter()
                .find(|link| link.model.spec.client == slot && link.model.port(slot) == Ok(second))
                .map(|link| link.model.spec.service)
                .ok_or(abi::DENIED)?;
            if matches!(
                session.scheduler.states()[peer],
                TaskState::Vacant | TaskState::Exited
            ) {
                return Err(abi::PEER_CLOSED);
            }
            let index = session.memory.model.offer(slot, first, peer)?;
            let pages = session.memory.model.entries[index].pages;
            // SAFETY: No consumer has mapped yet; downgrade the inactive producer before returning the offer.
            unsafe {
                session.tasks[slot]
                    .space
                    .as_mut()
                    .unwrap()
                    .seal_region(index, pages);
            }
            session.tasks[slot].report.memory.sealed_pages += pages;
            Ok(0)
        }
        abi::MEMORY_MAP => {
            if second != 0 {
                let owner = session
                    .links
                    .iter()
                    .find(|link| {
                        link.model.spec.service == slot && link.model.port(slot) == Ok(second)
                    })
                    .map(|link| link.model.spec.client)
                    .ok_or(abi::DENIED)?;
                session.memory.model.check_owner(first, owner)?;
            }
            let index = session.memory.model.accept(slot, first)?;
            let pages = session.memory.model.entries[index].pages;
            // SAFETY: Sealed owner is read-only; backing is pinned until this accepted reader releases/dies.
            let result = unsafe {
                session.tasks[slot].space.as_mut().unwrap().map_region(
                    index,
                    &session.memory.backing[index][..pages],
                    false,
                )
            };
            if result.is_err() {
                session.memory.model.entries[index].accepted = false;
                return Err(abi::NO_MEMORY);
            }
            session.tasks[slot].report.memory.mapped_pages += pages;
            Ok(wire::address(index))
        }
        abi::MEMORY_RELEASE => {
            let index = session.memory.model.release(slot, first)?;
            let pages = session.memory.model.entries[index].pages;
            // SAFETY: Calling task surrendered this mapping; no other mapping is withdrawn.
            unsafe {
                session.tasks[slot]
                    .space
                    .as_mut()
                    .unwrap()
                    .unmap_region(index, pages);
                session.memory.reclaim(index, frames);
            }
            session.tasks[slot].report.memory.unmapped_pages += pages;
            Ok(0)
        }
        _ => Err(abi::UNKNOWN),
    }
}
/// # Safety
/// Retired task cannot resume; boot stack/kernel root/IRQs off. Remove its aliases before freeing backing.
pub(super) unsafe fn close(session: &mut Session, slot: usize, frames: &mut Frames<'_>) {
    for index in 0..wire::MAX_REGIONS {
        let region = session.memory.model.entries[index];
        if region.owner == Some(slot) || (region.peer == Some(slot) && region.accepted) {
            // SAFETY: Only the dead task's mappings disappear. Accepted live readers keep their physical lease.
            unsafe {
                session.tasks[slot]
                    .space
                    .as_mut()
                    .unwrap()
                    .unmap_region(index, region.pages);
            }
            session.tasks[slot].report.memory.unmapped_pages += region.pages;
        }
        session.memory.model.close(index, slot);
        // SAFETY: Model has no remaining live alias when reclamation proceeds.
        unsafe {
            session.memory.reclaim(index, frames);
        }
    }
}
