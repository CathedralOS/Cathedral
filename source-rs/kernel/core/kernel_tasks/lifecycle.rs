//! Admit with frame/heap rollback, then reap retired tasks on the boot stack.
use super::{
    SpawnError,
    execution::task_entry,
    session::{Session, Task},
    stacks,
};
use crate::{
    memory::FrameAllocator,
    scheduler::{TaskId, TaskState},
};
use alloc::{alloc::alloc, boxed::Box};
use cathedral_arch::Context;
use core::alloc::Layout;

pub(super) unsafe fn initial(
    session: &mut Session,
    entries: &[fn()],
    frames: &mut FrameAllocator,
) -> Result<(), SpawnError> {
    for &entry in entries {
        // SAFETY: Before callback registration; runtime caller owns machine setup.
        if let Err(error) = unsafe { session.spawn(entry) } {
            for slot in 0..session.tasks.len() {
                if session.tasks[slot].take().is_some() {
                    // SAFETY: These tasks have never run; all context users are gone.
                    unsafe {
                        stacks::release(frames, slot);
                    }
                }
            }
            assert_eq!(frames.allocated(), session.baseline);
            return Err(error);
        }
    }
    Ok(())
}

impl Task {
    fn allocate(context: Context, entry: fn()) -> Result<Box<Self>, SpawnError> {
        // SAFETY: Matching global allocator/layout; initialize the complete value
        // before transferring sole ownership to Box. A null allocation is ordinary
        // admission failure, not the infallible allocator's terminal OOM path.
        unsafe {
            let pointer = alloc(Layout::new::<Self>()).cast::<Self>();
            if pointer.is_null() {
                return Err(SpawnError::OutOfHeap);
            }
            pointer.write(Self {
                context,
                entry,
                reply: None,
            });
            Ok(Box::from_raw(pointer))
        }
    }
}
impl Session {
    pub(super) unsafe fn spawn(&mut self, entry: fn()) -> Result<TaskId, SpawnError> {
        let slot = self.scheduler.vacancy().ok_or(SpawnError::TaskLimit)?;
        // SAFETY: Borrowed exclusively for run(), with IRQs off on the boot stack.
        let frames = unsafe { &mut *self.frames };
        let budget = self
            .config
            .frame_limit
            .saturating_sub(frames.allocated() - self.baseline);
        // SAFETY: Vacant slot, unique physical custody and exclusive mapper access.
        let stack = unsafe { stacks::allocate(frames, slot, budget)? };
        // SAFETY: Fresh guarded stack remains live until the task has switched out.
        let context = unsafe { Context::new(stack, task_entry, slot) };
        self.tasks[slot] = match Task::allocate(context, entry) {
            Ok(task) => Some(task),
            Err(error) => {
                // SAFETY: Admission failed; this stack was never entered.
                unsafe {
                    stacks::release(frames, slot);
                }
                return Err(error);
            }
        };
        let id = self.scheduler.admit(slot);
        self.stats.spawned += 1;
        Ok(id)
    }

    pub(super) unsafe fn service(&mut self) {
        for slot in 0..self.tasks.len() {
            if self.scheduler.states()[slot] == TaskState::Exited {
                drop(self.tasks[slot].take());
                // SAFETY: Running on boot stack; no context can resume this task.
                unsafe {
                    stacks::release(&mut *self.frames, slot);
                }
                self.scheduler.reap(slot);
            }
        }
        if let Some((caller, entry)) = self.service.take() {
            // SAFETY: Serialized admission on boot stack, never inside the IRQ.
            let result = unsafe { self.spawn(entry) };
            self.tasks[caller].as_mut().unwrap().reply = Some(result);
        }
    }
}
