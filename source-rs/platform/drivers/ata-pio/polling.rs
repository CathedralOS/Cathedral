//! Observe one command phase without replaying writes after an uncertain outcome.
//! Keep operation/LBA/elapsed context: a busy flush and a missing DRQ are different
//! failures, even when both exhaust the current bootstrap deadline.
use super::status::{State, classify};
use super::{abi, now, read, yield_now};
use core::fmt::{self, Write};
pub(super) fn wait(data: bool, operation: u8, phase: &str, sector: u64) -> Result<(), u64> {
    let start = now()?;
    let mut polls = 0u64;
    loop {
        let status = read(8)?;
        polls += 1;
        let elapsed = now()?.wrapping_sub(start);
        let state = classify(status, data, elapsed);
        if matches!(state, State::Failed | State::Timeout)
            || (state == State::Ready && elapsed >= 10)
        {
            let mut line = Line {
                bytes: [0; 224],
                len: 0,
            };
            writeln!(line, "ATA wait: op={operation:02x} phase={phase} lba={sector} data={} status={status:02x} ticks={elapsed} polls={polls} result={state:?}", u8::from(data)).ok();
            cathedral_user_runtime::write(&line.bytes[..line.len]).ok();
        }
        match state {
            State::Ready => return Ok(()),
            State::Failed => return Err(abi::IO_ERROR),
            State::Timeout => return Err(abi::TIMED_OUT),
            State::Wait => yield_now(),
        }
    }
}
#[repr(align(256))]
struct Line {
    bytes: [u8; 224],
    len: usize,
}
impl Write for Line {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len + text.len();
        self.bytes
            .get_mut(self.len..end)
            .ok_or(fmt::Error)?
            .copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}
