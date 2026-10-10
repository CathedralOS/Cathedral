//! Boot-selected executables and grants; userspace cannot widen this composition.
use super::Program;
use crate::ipc::EndpointSpec;
pub struct Config<'a> {
    pub frame_limit: usize,
    pub links: &'a [crate::link::LinkSpec],
    pub endpoints: &'a [EndpointSpec],
    pub supervision: &'a [Supervision<'a>],
    pub clock_readers: &'a [usize],
}

pub struct Supervision<'a> {
    /// Exclusive secondary ISA ATA controller; no DMA or arbitrary ports.
    pub disk: bool,
    /// Exclusive PC bootstrap byte channel; configuration/decoding stay in userspace.
    pub keyboard: bool,
    /// Exclusive boot-approved display aperture, granted only to the child.
    pub framebuffer: Option<cathedral_contracts::display::Framebuffer>,
    pub owner: usize,
    pub peer: usize,
    /// Boot selects executable and first entry argument; spawn supplies the second.
    pub program: Program<'a>,
    /// Physical admission budget per spawn, including deterministic failure probes.
    pub frame_limit: usize,
}
