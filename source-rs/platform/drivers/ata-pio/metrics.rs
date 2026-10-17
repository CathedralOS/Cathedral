//! Per-process diagnostic counters, sampled around provider operations.
use core::sync::atomic::{AtomicU64, Ordering::Relaxed};
static READS: AtomicU64 = AtomicU64::new(0);
static WRITES: AtomicU64 = AtomicU64::new(0);
static FLUSHES: AtomicU64 = AtomicU64::new(0);
static WAIT_TICKS: AtomicU64 = AtomicU64::new(0);
static POLLS: AtomicU64 = AtomicU64::new(0);
pub(crate) fn operation(op: u8) {
    match op {
        0x20 => &READS,
        0x30 => &WRITES,
        0xe7 => &FLUSHES,
        _ => return,
    }
    .fetch_add(1, Relaxed);
}
pub(crate) fn waited(ticks: u64, polls: u64) {
    WAIT_TICKS.fetch_add(ticks, Relaxed);
    POLLS.fetch_add(polls, Relaxed);
}
/// Reads, writes, flushes, cumulative phase-wait ticks and status polls.
/// These count driver operations, not physical-media bytes or queue service time.
pub fn snapshot() -> [u64; 5] {
    [&READS, &WRITES, &FLUSHES, &WAIT_TICKS, &POLLS].map(|n| n.load(Relaxed))
}
