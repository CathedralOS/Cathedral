//! Buffered observations of uninterrupted kernel phases. Samples include nested
//! phases and host descheduling; they are not additive or hardware latency bounds.
//! The memory lab enables collection; ordinary builds erase the clocks/storage.
#[derive(Clone, Copy, Debug)]
pub(in crate::user_tasks) enum Phase {
    Trap,
    Syscall,
    InterruptWork,
    Deferred,
    Withdraw,
    Retire,
    Abandon,
    Collect,
    Conservation,
    Admission,
    DecodeRoot,
    ImageBatch,
    RegionTables,
    Framebuffer,
    Rollback,
    Escrow,
    References,
    Allocate,
    Seal,
    Map,
    Release,
    Freeze,
    MapObject,
    UnmapObject,
    OtherMemory,
}

#[cfg(feature = "memory-lab")]
const PHASES: [Phase; 25] = [
    Phase::Trap,
    Phase::Syscall,
    Phase::InterruptWork,
    Phase::Deferred,
    Phase::Withdraw,
    Phase::Retire,
    Phase::Abandon,
    Phase::Collect,
    Phase::Conservation,
    Phase::Admission,
    Phase::DecodeRoot,
    Phase::ImageBatch,
    Phase::RegionTables,
    Phase::Framebuffer,
    Phase::Rollback,
    Phase::Escrow,
    Phase::References,
    Phase::Allocate,
    Phase::Seal,
    Phase::Map,
    Phase::Release,
    Phase::Freeze,
    Phase::MapObject,
    Phase::UnmapObject,
    Phase::OtherMemory,
];

#[cfg(feature = "memory-lab")]
#[derive(Clone, Copy, Default)]
struct Sample {
    count: u64,
    total_us: u64,
    max_us: u64,
    over_ms: u64,
}

#[derive(Default)]
pub(in crate::user_tasks) struct Timing {
    // Allocate once at session preparation, never on a measured path or stack.
    #[cfg(feature = "memory-lab")]
    samples: alloc::boxed::Box<[Sample; PHASES.len()]>,
}

#[inline]
pub(in crate::user_tasks) fn start() -> u64 {
    #[cfg(feature = "memory-lab")]
    {
        cathedral_arch::cpu_time_us()
    }
    #[cfg(not(feature = "memory-lab"))]
    {
        0
    }
}

impl Timing {
    #[inline]
    pub(in crate::user_tasks) fn record(&mut self, phase: Phase, start: u64) {
        #[cfg(feature = "memory-lab")]
        {
            let elapsed = cathedral_arch::cpu_time_us().saturating_sub(start);
            let sample = &mut self.samples[phase as usize];
            sample.count = sample.count.saturating_add(1);
            sample.total_us = sample.total_us.saturating_add(elapsed);
            sample.max_us = sample.max_us.max(elapsed);
            sample.over_ms = sample.over_ms.saturating_add(u64::from(elapsed >= 1_000));
        }
        #[cfg(not(feature = "memory-lab"))]
        let _ = (phase, start);
    }

    /// Only after the session has stopped: no diagnostic I/O in the samples.
    pub(in crate::user_tasks) fn report(&self, output: fn(&[u8]) -> bool) {
        #[cfg(feature = "memory-lab")]
        {
            use core::fmt::Write;
            for phase in PHASES {
                let sample = self.samples[phase as usize];
                if sample.count == 0 {
                    continue;
                }
                let mut line = Line {
                    bytes: [0; 192],
                    len: 0,
                };
                writeln!(
                    line,
                    "KERNEL_PHASE phase={phase:?} count={} total_us={} max_us={} over_1ms={}",
                    sample.count, sample.total_us, sample.max_us, sample.over_ms
                )
                .unwrap();
                output(&line.bytes[..line.len]);
            }
        }
        #[cfg(not(feature = "memory-lab"))]
        let _ = output;
    }
}

#[cfg(feature = "memory-lab")]
struct Line {
    bytes: [u8; 192],
    len: usize,
}
#[cfg(feature = "memory-lab")]
impl core::fmt::Write for Line {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let end = self.len + text.len();
        self.bytes
            .get_mut(self.len..end)
            .ok_or(core::fmt::Error)?
            .copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}
