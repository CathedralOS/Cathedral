// Buffered kernel-phase and wait-to-selection observations. Reports are emitted
// only after execution; phases overlap and include host descheduling.
// The memory lab enables collection; ordinary builds erase the clocks/storage.
#[cfg(feature = "memory-lab")]
use phases::{PHASES, Sample};

pub(in crate::user_tasks) struct Timing {
    // Allocate once at session preparation, never on a measured path or stack.
    #[cfg(feature = "memory-lab")]
    samples: alloc::vec::Vec<Sample>,
    #[cfg(feature = "memory-lab")]
    waits: waits::Observations,
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
    #[cfg(feature = "memory-lab")]
    pub(in crate::user_tasks) fn count(&self, phase: Phase) -> u64 {
        self.samples[phase as usize].count
    }
    pub(in crate::user_tasks) fn try_new() -> Result<Self, alloc::collections::TryReserveError> {
        #[cfg(feature = "memory-lab")]
        {
            let mut samples = alloc::vec::Vec::new();
            samples.try_reserve_exact(PHASES.len())?;
            samples.resize(PHASES.len(), Sample::default());
            Ok(Self {
                samples,
                waits: waits::Observations::try_new(cathedral_kernel_contracts::user::MAX_TASKS)?,
            })
        }
        #[cfg(not(feature = "memory-lab"))]
        Ok(Self {})
    }

    #[cfg(feature = "memory-lab")]
    pub(in crate::user_tasks) fn wait_ready(
        &mut self,
        slot: usize,
        kind: super::waits::Kind,
        deadline: Option<u64>,
        now: u64,
    ) {
        self.waits
            .complete(slot, kind, deadline, now, cathedral_arch::cpu_time_us());
    }

    #[cfg(feature = "memory-lab")]
    pub(in crate::user_tasks) fn wait_selected(&mut self, next: Option<usize>) {
        if let Some(slot) = next.filter(|&slot| self.waits.pending(slot)) {
            self.waits.select(slot, cathedral_arch::cpu_time_us());
        }
    }

    #[cfg(feature = "memory-lab")]
    pub(in crate::user_tasks) fn cancel_wait(&mut self, slot: usize) {
        self.waits.cancel(slot);
    }

    #[inline]
    pub(in crate::user_tasks) fn record(&mut self, phase: Phase, start: u64) {
        #[cfg(feature = "memory-lab")]
        {
            let elapsed = cathedral_arch::cpu_time_us().saturating_sub(start);
            self.record_elapsed(phase, elapsed);
        }
        #[cfg(not(feature = "memory-lab"))]
        let _ = (phase, start);
    }

    #[cfg(feature = "memory-lab")]
    pub(in crate::user_tasks) fn record_elapsed(&mut self, phase: Phase, elapsed: u64) {
        let sample = &mut self.samples[phase as usize];
        sample.count = sample.count.saturating_add(1);
        sample.total_us = sample.total_us.saturating_add(elapsed);
        sample.max_us = sample.max_us.max(elapsed);
        sample.over_ms = sample.over_ms.saturating_add(u64::from(elapsed >= 1_000));
    }

    /// Only after the session has stopped: no diagnostic I/O in the samples.
    pub(in crate::user_tasks) fn report(&self, output: fn(&[u8]) -> bool) {
        #[cfg(feature = "memory-lab")]
        {
            use core::fmt::Write;
            self.waits.report(output);
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
struct Line<const N: usize> {
    bytes: [u8; N],
    len: usize,
}
#[cfg(feature = "memory-lab")]
impl<const N: usize> core::fmt::Write for Line<N> {
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
