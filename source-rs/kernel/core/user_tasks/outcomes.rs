//! Outcomes and accounting returned after a complete user-task session.
use cathedral_arch as arch;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Returned(u64),
    Fault(arch::UserFault),
    Cancelled,
}
#[derive(Default, Debug)]
pub struct Report {
    pub memory: MemoryReport,
    pub exit: Option<Exit>,
    pub preemptions: usize,
    pub yields: usize,
    pub writes: usize,
    pub rejected: usize,
    pub completion_order: usize,
    pub frames: usize,
    pub receives_blocked: usize,
    pub readiness_blocked: usize,
    pub keyboard_reads_blocked: usize,
    pub ipc_sent: usize,
    pub ipc_received: usize,
    pub spawned: usize,
    pub reaped: usize,
    pub waits_blocked: usize,
    pub cancelled: usize,
    pub wait_timeouts: usize,
    pub cancelled_ready: usize,
    pub cancelled_blocked: usize,
}
/// Runtime page-object work only; excludes image/stack/table admission and CR3
/// switches. Counts successful leaf mapping/permission edits, not TLB misses.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct MemoryReport {
    pub deferred_calls: usize,
    pub allocated_pages: usize,
    pub mapped_pages: usize,
    pub sealed_pages: usize,
    pub unmapped_pages: usize,
}
