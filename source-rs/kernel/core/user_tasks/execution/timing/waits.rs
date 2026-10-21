//! Registered wait completion to first selection, including CPU-ceiling delays.
use crate::user_tasks::{execution::waits::Kind, time};
use alloc::{collections::TryReserveError, vec::Vec};

#[derive(Clone, Copy, Default)]
pub(super) struct Sample {
    pub(super) completed: u64,
    pub(super) selected: u64,
    pub(super) cancelled: u64,
    pub(super) total_ready_us: u64,
    pub(super) max_ready_us: u64,
    pub(super) due: u64,
    pub(super) total_late_ticks: u64,
    pub(super) max_late_ticks: u64,
}
#[derive(Clone, Copy)]
struct Ready {
    kind: Kind,
    since_us: u64,
}

pub(super) struct Observations {
    ready: Vec<Option<Ready>>,
    samples: Vec<[Sample; 3]>,
}
impl Observations {
    pub(super) fn try_new(tasks: usize) -> Result<Self, TryReserveError> {
        let mut ready = Vec::new();
        ready.try_reserve_exact(tasks)?;
        ready.resize(tasks, None);
        let mut samples = Vec::new();
        samples.try_reserve_exact(tasks)?;
        samples.resize(tasks, [Sample::default(); 3]);
        Ok(Self { ready, samples })
    }
    pub(super) fn complete(
        &mut self,
        slot: usize,
        kind: Kind,
        deadline: Option<u64>,
        now: u64,
        now_us: u64,
    ) {
        assert!(
            self.ready[slot].is_none(),
            "wait completed twice without selection or cancellation"
        );
        self.ready[slot] = Some(Ready {
            kind,
            since_us: now_us,
        });
        let sample = &mut self.samples[slot][kind as usize];
        sample.completed = sample.completed.saturating_add(1);
        if let Some(deadline) = deadline.filter(|&deadline| time::reached(now, deadline)) {
            let late = now.wrapping_sub(deadline);
            sample.due = sample.due.saturating_add(1);
            sample.total_late_ticks = sample.total_late_ticks.saturating_add(late);
            sample.max_late_ticks = sample.max_late_ticks.max(late);
        }
    }
    pub(super) fn pending(&self, slot: usize) -> bool {
        self.ready[slot].is_some()
    }
    pub(super) fn select(&mut self, slot: usize, now_us: u64) {
        if let Some(ready) = self.ready[slot].take() {
            let elapsed = now_us.saturating_sub(ready.since_us);
            let sample = &mut self.samples[slot][ready.kind as usize];
            sample.selected = sample.selected.saturating_add(1);
            sample.total_ready_us = sample.total_ready_us.saturating_add(elapsed);
            sample.max_ready_us = sample.max_ready_us.max(elapsed);
        }
    }
    pub(super) fn cancel(&mut self, slot: usize) {
        if let Some(ready) = self.ready[slot].take() {
            let sample = &mut self.samples[slot][ready.kind as usize];
            sample.cancelled = sample.cancelled.saturating_add(1);
        }
    }
    #[cfg(test)]
    pub(super) fn sample(&self, slot: usize, kind: Kind) -> Sample {
        self.samples[slot][kind as usize]
    }

    #[cfg(feature = "memory-lab")]
    pub(super) fn report(&self, output: fn(&[u8]) -> bool) {
        use core::fmt::Write;
        for (slot, samples) in self.samples.iter().enumerate() {
            for kind in [Kind::Receive, Kind::Readiness, Kind::Child] {
                let sample = samples[kind as usize];
                if sample.completed == 0 {
                    continue;
                }
                let pending =
                    u64::from(self.ready[slot].is_some_and(|r| r.kind as usize == kind as usize));
                let mut line = super::Line {
                    bytes: [0; 384],
                    len: 0,
                };
                writeln!(line, "WAIT_DISPATCH task={slot} kind={kind:?} completed={} selected={} cancelled={} pending={pending} total_ready_us={} max_ready_us={} due={} total_late_ticks={} max_late_ticks={}",
                    sample.completed, sample.selected, sample.cancelled, sample.total_ready_us, sample.max_ready_us,
                    sample.due, sample.total_late_ticks, sample.max_late_ticks).unwrap();
                output(&line.bytes[..line.len]);
            }
        }
    }
}
