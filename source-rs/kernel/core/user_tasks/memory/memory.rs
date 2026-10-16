// Boot-stack page-object operations. Traps enqueue; this owner maps, seals and reclaims.
use super::{Frames, Session, syscalls};
use cathedral_kernel_contracts::memory as wire;
use cathedral_kernel_contracts::user as abi;
use regions::Regions;
pub(super) struct Memory {
    pub model: Regions,
    pub funding: crate::user_tasks::resources::physical::Funding,
    backing: [alloc::vec::Vec<u64>; wire::MAX_REGIONS],
}
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
            (&raw mut (*pointer).backing).write(core::array::from_fn(|_| alloc::vec::Vec::new()));
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
}
/// # Safety
/// Sole CPU, boot stack, kernel root active, IRQs off; every user root is inactive.
pub(super) unsafe fn process(session: &mut Session, frames: &mut Frames<'_>) {
    // SAFETY: Queue discards never published a mapping; same boot-stack custody.
    unsafe {
        objects::retire_escrow(session, frames);
    }
    for slot in 0..session.tasks.len() {
        let Some(call) = session.tasks[slot].memory_call.take() else {
            continue;
        };
        session.scheduler.account_cpu(cathedral_arch::cpu_time_us());
        // Admission budgets are distinct from runtime page budgets enforced by Regions.
        frames.remaining = usize::MAX;
        session.tasks[slot].report.memory.deferred_calls += 1;
        // SAFETY: Same machine and ownership obligations as process.
        let result = unsafe { dispatch(session, frames, slot, call) };
        syscalls::complete(session, slot, result);
        session.scheduler.unblock(slot);
        session
            .scheduler
            .charge_cpu(slot, cathedral_arch::cpu_time_us());
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
        // SAFETY: Caller provides exclusive inactive roots and frame custody.
        abi::MEMORY_SEAL => unsafe { sharing::seal(session, slot, first, second) },
        // SAFETY: Caller provides exclusive inactive roots and frame custody.
        abi::MEMORY_MAP => unsafe { sharing::map(session, slot, first, second) },
        // SAFETY: Caller provides exclusive inactive roots and frame custody.
        abi::MEMORY_RELEASE => unsafe { retirement::release(session, frames, slot, first) },
        // SAFETY: Exclusive inactive roots and retained backing, as for the legacy operations.
        abi::MEMORY_FREEZE => unsafe { objects::freeze(session, slot, first) },
        // SAFETY: Exclusive inactive roots and retained backing.
        abi::MEMORY_MAP_OBJECT => unsafe { objects::map(session, slot, first) },
        // SAFETY: Caller withdraws only its own mapping.
        abi::MEMORY_UNMAP_OBJECT => unsafe { objects::unmap(session, slot, first) },
        _ => Err(abi::UNKNOWN),
    }
}
