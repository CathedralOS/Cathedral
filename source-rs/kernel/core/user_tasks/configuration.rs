//! Boot-selected executable authority and resource envelopes; userspace subdivides them.
use crate::ipc::EndpointSpec;
pub struct Config<'a> {
    /// Code-only grants supplied to the sole initial process, with no device rights.
    pub executables: &'a [Program<'a>],
    /// Unbound process accounts owned initially by the sole initial process.
    pub dynamic_tasks: usize,
    /// Per boot/launch account, including dormant children. Empty selects fixed defaults.
    pub channel_limits: &'a [usize],
    /// Per-account CPU burst microseconds, refilled at limit/100_000 of one CPU.
    /// Empty or zero adds no local cap; ancestor caps still apply.
    pub cpu_limits: &'a [u64],
    pub memory: &'a [cathedral_kernel_contracts::memory::Grant],
    pub frame_limit: usize,
    pub links: &'a [crate::ipc::link::LinkSpec],
    pub endpoints: &'a [EndpointSpec],
    pub supervision: &'a [Supervision<'a>],
    pub clock_readers: &'a [usize],
}

pub struct Supervision<'a> {
    /// Exclusive board-approved non-DMA register/interrupt grant.
    pub device: Option<cathedral_arch::board::DeviceId>,
    /// Exclusive secondary ISA ATA controller; no DMA or arbitrary ports.
    pub disk: bool,
    /// Exclusive PC bootstrap byte channel; configuration/decoding stay in userspace.
    pub keyboard: bool,
    /// Exclusive boot-approved display aperture, granted only to the child.
    pub framebuffer: Option<cathedral_kernel_contracts::framebuffer::Framebuffer>,
    pub owner: usize,
    pub peer: usize,
    /// Boot selects executable and first entry argument; spawn supplies the second.
    pub program: Program<'a>,
    /// Physical admission budget per spawn, including deterministic failure probes.
    pub frame_limit: usize,
}

#[derive(Clone, Copy)]
pub enum Executable<'a> {
    Probe(&'a [u8]),
    Elf(&'a [u8]),
}
pub struct Program<'a> {
    pub executable: Executable<'a>,
    /// Initial architecture entry arguments; ordinary data, not capabilities.
    pub arguments: [u64; 2],
}

#[derive(Debug)]
pub enum Error {
    InvalidCpuBudget,
    InvalidCount,
    InvalidEndpoints,
    OutOfHeap,
    Executable(super::admission::elf::Error),
    Memory(cathedral_arch::MemoryError),
}
