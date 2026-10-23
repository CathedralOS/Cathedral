//! Allocation-free reporting; sample before formatting. Peak is cumulative,
//! requested heap bytes only; sealed reader pages and stack are separate budgets.
//! Beside the heap, each report carries the private pages and frames the
//! kernel charges this account, which include reply pages still held by their
//! readers: a reply nobody releases shows there, not in the heap.
use core::fmt::{self, Write};
struct Line {
    bytes: [u8; 192],
    len: usize,
}
impl Write for Line {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len + text.len();
        let target = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        target.copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}
pub fn report(phase: &str) {
    let s = crate::HEAP.stats();
    let pages = cathedral_user_runtime::resources::budget().map_or(0, |b| b.used.private_pages);
    let frames = cathedral_user_runtime::resources::physical_budget().map_or(0, |p| p.charged);
    let mut line = Line {
        bytes: [0; 192],
        len: 0,
    };
    writeln!(
        line,
        "Cathedral objects heap: {phase} live={} peak={} largest={} failures={} capacity={} pages={pages} frames={frames}",
        s.live, s.peak, s.largest, s.failures, s.capacity
    )
    .unwrap();
    cathedral_user_runtime::write(&line.bytes[..line.len]).unwrap();
}
