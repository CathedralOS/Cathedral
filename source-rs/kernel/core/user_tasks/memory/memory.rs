// Boot-stack page-object operations. Traps enqueue; this owner maps, seals and reclaims.
use super::{Frames, Session, syscalls};
use crate::user_tasks::execution::timing::{self, Phase as TimedPhase};
use cathedral_kernel_contracts::memory as wire;
use cathedral_kernel_contracts::user as abi;
use regions::Regions;
pub(super) struct Memory {
    pub model: Regions,
    pub funding: crate::user_tasks::resources::physical::Funding,
    backing: [alloc::vec::Vec<u64>; wire::MAX_REGIONS],
    /// Kernel references to drop on the boot stack, where frames can be reclaimed.
    pending: [u16; wire::MAX_REGIONS],
    retirement_pending: bool,
    pending_regions: [u64; wire::MAX_REGIONS.div_ceil(64)],
    reference_cursor: usize,
    releases: [Option<usize>; abi::MAX_TASKS],
    escrow_slots: [u64; wire::MAX_READ_HANDLES.div_ceil(64)],
    escrow_object: Option<usize>,
    escrow_active: bool,
    allocations: [Option<allocation::Staging>; abi::MAX_TASKS],
}
// A whole subtree can surrender every executable reference before the boot
// loop settles them. The counter must cover all accounts, not just one task.
const _: () = assert!(abi::MAX_TASKS * abi::MAX_LAUNCHES <= u16::MAX as usize);
impl Memory {
    pub fn new(
        tasks: usize,
        dynamic: usize,
        grants: &[wire::Grant],
    ) -> Result<alloc::boxed::Box<Self>, u64> {
        let model = Regions::new(tasks, grants)?;
        // Bootstrap policy: each bound account starts with 288 frames plus its
        // private-page quota. Unbound accounts' 288-frame allowances belong to root.
        let mut limits = [0; abi::MAX_TASKS];
        for (account, limit) in limits.iter_mut().enumerate().take(tasks - dynamic) {
            *limit = abi::MAX_ADMISSION_FRAMES + model.limits(account)[0];
        }
        limits[0] += dynamic * abi::MAX_ADMISSION_FRAMES;
        let funding = crate::user_tasks::resources::physical::Funding::new(limits);
        // Keep the bounded arena off the boot stack: debug builds otherwise copy
        // it through each session preparation/result frame. Allocation is fallible.
        // SAFETY: Correct nonzero layout, null checked before initialization; Box
        // takes unique custody of the fully initialized allocation exactly once.
        unsafe {
            let pointer = alloc::alloc::alloc(core::alloc::Layout::new::<Self>()).cast::<Self>();
            if pointer.is_null() {
                return Err(abi::NO_MEMORY);
            }
            (&raw mut (*pointer).model).write(model);
            (&raw mut (*pointer).funding).write(funding);
            // Initialize each vector directly; a temporary array scales with
            // task capacity and would overflow the guarded boot stack.
            let backing = (&raw mut (*pointer).backing).cast::<alloc::vec::Vec<u64>>();
            for index in 0..wire::MAX_REGIONS {
                backing.add(index).write(alloc::vec::Vec::new());
            }
            (&raw mut (*pointer).pending).write([0; wire::MAX_REGIONS]);
            (&raw mut (*pointer).retirement_pending).write(false);
            (&raw mut (*pointer).pending_regions).write([0; wire::MAX_REGIONS.div_ceil(64)]);
            (&raw mut (*pointer).reference_cursor).write(0);
            (&raw mut (*pointer).releases).write([None; abi::MAX_TASKS]);
            (&raw mut (*pointer).escrow_slots).write([0; wire::MAX_READ_HANDLES.div_ceil(64)]);
            (&raw mut (*pointer).escrow_object).write(None);
            (&raw mut (*pointer).escrow_active).write(false);
            (&raw mut (*pointer).allocations).write([None; abi::MAX_TASKS]);
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
    /// Retained backing frames, one per page in order; caller checks object mode.
    pub fn frames_of(&self, region: usize) -> &[u64] {
        &self.backing[region][..self.model.entries[region].pages]
    }
    pub fn reserved_unbacked(&self) -> usize {
        self.allocations
            .iter()
            .flatten()
            .map(allocation::Staging::reserved)
            .sum()
    }
}
/// # Safety
/// Sole CPU, boot stack, kernel root active, IRQs off; every user root is inactive.
pub(super) unsafe fn process(session: &mut Session, frames: &mut Frames<'_>) {
    // SAFETY: Queue discards never published a mapping; same boot-stack custody.
    unsafe {
        if session.ipc.memory_retirement_pending() || session.memory.escrow_active {
            let start = timing::start();
            escrow::step(session, frames);
            session.timing.record(TimedPhase::Escrow, start);
        }
        if session.memory.retirement_pending {
            let start = timing::start();
            session.memory.settle(frames);
            session.timing.record(TimedPhase::References, start);
        }
    }
    use crate::user_tasks::execution::deferred::Work;
    if let Some(slot) = session.deferred.next(Work::Memory) {
        let call = session.tasks[slot]
            .memory_call
            .expect("queued memory call missing");
        session.scheduler.account_cpu(cathedral_arch::cpu_time_us());
        frames.remaining = usize::MAX;
        if session.memory.allocations[slot].is_none() && session.memory.releases[slot].is_none() {
            session.tasks[slot].report.memory.deferred_calls += 1;
        }
        let start = timing::start();
        // SAFETY: Same machine and ownership obligations as process.
        let result = unsafe {
            if call[0] == abi::MEMORY_ALLOCATE {
                allocation::step(session, frames, slot, call[1] as usize)
            } else if call[0] == abi::MEMORY_RELEASE {
                retirement::step(session, frames, slot, call[1])
            } else {
                Some(dispatch(session, slot, call))
            }
        };
        let phase = match call[0] {
            abi::MEMORY_ALLOCATE => TimedPhase::Allocate,
            abi::MEMORY_SEAL => TimedPhase::Seal,
            abi::MEMORY_MAP => TimedPhase::Map,
            abi::MEMORY_RELEASE => TimedPhase::Release,
            abi::MEMORY_FREEZE => TimedPhase::Freeze,
            abi::MEMORY_MAP_OBJECT => TimedPhase::MapObject,
            abi::MEMORY_UNMAP_OBJECT => TimedPhase::UnmapObject,
            _ => TimedPhase::OtherMemory,
        };
        session.timing.record(phase, start);
        if let Some(result) = result {
            session.tasks[slot].memory_call = None;
            syscalls::complete(session, slot, result);
            session.scheduler.unblock(slot);
        } else {
            session.deferred.insert(Work::Memory, slot);
        }
        session
            .scheduler
            .charge_cpu(slot, cathedral_arch::cpu_time_us());
    }
}
unsafe fn dispatch(
    session: &mut Session,
    slot: usize,
    [number, first, second]: [u64; 3],
) -> Result<u64, u64> {
    match number {
        abi::MEMORY_SHARE_MUTABLE => session.memory.model.share_mutable(slot, first).map(|_| 0),
        // SAFETY: Inactive roots and retained backing; only producer gets writes.
        abi::MEMORY_MAP_SHARED => unsafe { objects::map_kind(session, slot, first, true) },
        // SAFETY: Caller provides exclusive inactive roots and frame custody.
        abi::MEMORY_SEAL => unsafe { sharing::seal(session, slot, first, second) },
        // SAFETY: Caller provides exclusive inactive roots and frame custody.
        abi::MEMORY_MAP => unsafe { sharing::map(session, slot, first, second) },
        // SAFETY: Exclusive inactive roots and retained backing, as for the legacy operations.
        abi::MEMORY_FREEZE => unsafe { objects::freeze(session, slot, first) },
        // SAFETY: Exclusive inactive roots and retained backing.
        abi::MEMORY_MAP_OBJECT => unsafe { objects::map(session, slot, first) },
        // SAFETY: Caller withdraws only its own mapping.
        abi::MEMORY_UNMAP_OBJECT => unsafe { objects::unmap(session, slot, first) },
        _ => Err(abi::UNKNOWN),
    }
}
