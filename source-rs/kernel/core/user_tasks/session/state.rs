//! Owned session state, bounded frame source and non-reused grant epochs.
use super::Report;
use super::syscalls::{self, readiness, tasks};
use crate::{
    ipc::{Ipc, MAX_EPOCH},
    scheduler::Scheduler,
    supervision::Supervisor,
};
use alloc::vec::Vec;
use cathedral_arch::{Context, UserSpace};
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};
pub(super) struct Task {
    pub(super) memory_call: Option<[u64; 3]>,
    pub(super) context: Context,
    pub(super) space: Option<UserSpace>,
    pub(super) report: Report,
    pub(super) receive: Option<syscalls::Receive>,
    pub(super) wait: Option<tasks::Wait>,
    pub(super) notify: Option<readiness::Wait>,
    pub(super) keyboard_wait: bool,
    pub(super) keyboard_deadline: Option<u64>,
}
pub(super) struct Session {
    pub(super) pool: crate::admission_pool::Pool,
    pub(super) executables: crate::admission_pool::Executables,
    pub(super) names: crate::names::Names,
    pub(super) memory: alloc::boxed::Box<super::memory::Memory>,
    pub(super) tasks: Vec<Task>,
    pub(super) scheduler: Scheduler,
    pub(super) boot: Context,
    pub(super) output: fn(&[u8]) -> bool,
    pub(super) completed: usize,
    pub(super) ipc: Ipc,
    pub(super) launches: Vec<LaunchState>,
    pub(super) links: Vec<LinkState>,
    pub(super) keyboard: crate::byte_queue::ByteQueue,
    pub(super) frame_baseline: usize,
    pub(super) clock: crate::deadline::Clock,
}
pub(super) static ACTIVE: AtomicPtr<Session> = AtomicPtr::new(core::ptr::null_mut());

pub(super) static EPOCH: AtomicU64 = AtomicU64::new(1);

pub(super) struct LaunchState {
    pub(super) images: [Option<usize>; cathedral_contracts::user::MAX_LAUNCHES],
    pub(super) template: Option<usize>,
    pub(super) dynamic: bool,
    pub(super) frame_limit: usize,
    pub(super) cpu_origin: u64,
    pub(super) delegated: [u64; cathedral_contracts::user::MAX_LAUNCHES],
    pub(super) delegated_count: usize,
    pub(super) initial: [u64; cathedral_contracts::user::MAX_INITIAL_ENDPOINTS],
    pub(super) initial_count: usize,
    pub(super) disk: bool,
    pub(super) model: Supervisor,
    pub(super) endpoint_base: usize,
    pub(super) framebuffer: Option<cathedral_contracts::display::Framebuffer>,
    pub(super) keyboard: bool,
}
pub(super) struct LinkState {
    pub(super) model: crate::link::Link,
    pub(super) endpoint_base: usize,
}

pub(super) fn next_epoch() -> Option<u64> {
    EPOCH
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            (value <= MAX_EPOCH).then_some(value + 1)
        })
        .ok()
}
