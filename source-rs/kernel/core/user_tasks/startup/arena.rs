//! Fallible session storage, initialized in place so debug startup fits the boot stack.
use super::{Config, Error, Session, composition::Grants};
use crate::scheduler::Scheduler;
use crate::user_tasks::{admission::pool, memory::Memory};
use alloc::{boxed::Box, vec::Vec};
use cathedral_arch::Context;

pub(super) fn allocate(
    count: usize,
    config: &Config<'_>,
    grants: Grants,
    baseline: usize,
    output: fn(&[u8]) -> bool,
) -> Result<Box<Session>, Error> {
    let mut scheduler = Scheduler::try_new(count).map_err(|_| Error::OutOfHeap)?;
    if !scheduler.configure_cpu(config.cpu_limits) {
        return Err(Error::InvalidCpuBudget);
    }
    for launch in &grants.launches {
        scheduler.set_parent(launch.model.child, launch.model.owner);
    }
    let memory = Memory::new(count, config.dynamic_tasks, config.memory).map_err(|code| {
        if code == cathedral_kernel_contracts::user::NO_MEMORY {
            Error::OutOfHeap
        } else {
            Error::InvalidEndpoints
        }
    })?;
    let executables = pool::Executables::new(count).map_err(|_| Error::OutOfHeap)?;
    let timing =
        crate::user_tasks::execution::timing::Timing::try_new().map_err(|_| Error::OutOfHeap)?;
    let waits = crate::user_tasks::execution::waits::Registry::try_new(count)
        .map_err(|_| Error::OutOfHeap)?;
    let child_launch = core::array::from_fn(|slot| {
        grants
            .launches
            .iter()
            .position(|launch| launch.model.child == slot)
    });
    let device_routes = core::array::from_fn(|slot| {
        grants
            .launches
            .iter()
            .find(|launch| launch.model.child == slot)
            .and_then(|launch| launch.device)
    });
    let keyboard_owner = grants
        .launches
        .iter()
        .find(|launch| launch.keyboard)
        .map(|launch| launch.model.child);

    // Finish every fallible operation before raw allocation. A refusal drops
    // grants, memory and executables normally. Once allocated, initialize every
    // field without early returns, allocation or publication to trap handlers.
    // SAFETY: Nonzero correctly aligned layout, checked for null. Field writes
    // never reference uninitialized storage; Box assumes unique ownership only
    // after all fields are initialized. No whole Session temporary uses the stack.
    unsafe {
        let pointer = alloc::alloc::alloc(core::alloc::Layout::new::<Session>()).cast::<Session>();
        if pointer.is_null() {
            return Err(Error::OutOfHeap);
        }
        (&raw mut (*pointer).timing).write(timing);
        (&raw mut (*pointer).cancel_waiters).write(0);
        (&raw mut (*pointer).deferred).write(Default::default());
        (&raw mut (*pointer).child_launch).write(child_launch);
        (&raw mut (*pointer).waits).write(waits);
        (&raw mut (*pointer).device_routes).write(device_routes);
        (&raw mut (*pointer).keyboard_owner).write(keyboard_owner);
        (&raw mut (*pointer).devices).write(Default::default());
        (&raw mut (*pointer).pool).write(pool::Pool::new(count - config.dynamic_tasks, count));
        (&raw mut (*pointer).executables).write(executables);
        (&raw mut (*pointer).names).write(grants.names);
        (&raw mut (*pointer).memory).write(memory);
        (&raw mut (*pointer).tasks).write(Vec::new());
        (&raw mut (*pointer).scheduler).write(scheduler);
        (&raw mut (*pointer).boot).write(Context::default());
        (&raw mut (*pointer).output).write(output);
        (&raw mut (*pointer).completed).write(0);
        (&raw mut (*pointer).ipc).write(grants.ipc);
        (&raw mut (*pointer).launches).write(grants.launches);
        (&raw mut (*pointer).links).write(grants.links);
        (&raw mut (*pointer).keyboard)
            .write(crate::user_tasks::syscalls::devices::keyboard::queue::ByteQueue::new());
        (&raw mut (*pointer).frame_baseline).write(baseline);
        (&raw mut (*pointer).clock).write(grants.clock);
        Ok(Box::from_raw(pointer))
    }
}
