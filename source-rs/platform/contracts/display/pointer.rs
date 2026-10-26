//! Experimental logical pointers; independent of keyboard focus and production seats.
//! All packets use display's six-word encoding. Coordinates on delivery are signed
//! two's-complement app-local values (capture can move outside the app's scope).
pub const SOURCE_MODE: u64 = 0x4d4f5553;
pub const DEVICE: u64 = 32; // source-only: dx,dy,buttons,reset,0
pub const NEXT: u64 = 33; // client poll: all unused fields zero
pub const CAPTURE: u64 = 34; // client: pointer, enable,0,0,0
pub const DRIVE: u64 = 35; // client: lease,x,y,buttons,0; local clamped coordinates
pub const ENABLE: u64 = 36; // control: 1/0,0,0,0,0
pub const LEASE: u64 = 37; // control: client,1/0,0,0,0; reply carries lease epoch
pub const STATE: u64 = 38; // control: pointer,0,0,0,0; reply x,y,buttons,capture+1,lease
/// One outstanding wait per app; an event or control WAKE completes it.
/// Connection closure wakes the underlying receive with a transport error.
pub const WAIT: u64 = 39;
pub const STATS: u64 = 40; // control: replies requests, waits, delivered, redraws,0
pub const WAKE: u64 = 41; // control: client,0,0,0,0; finish one event wait with NONE
/// Source-only ordered reports: count followed by up to four packed reports.
/// Each word carries signed 16-bit dx/dy, three button bits and one reset bit;
/// high bits and unused words must be zero. Count must be in 1..=4.
/// One reply covers the entire validated batch; no partial malformed submission.
pub const DEVICE_BATCH: u64 = 42;
pub const SOURCE_BATCH: usize = 4;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SourceReport {
    pub dx: i16,
    pub dy: i16,
    pub buttons: u8,
    pub reset: bool,
}
#[derive(Default)]
pub struct SourceBatch {
    reports: [SourceReport; SOURCE_BATCH],
    count: usize,
}
impl SourceBatch {
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn is_full(&self) -> bool {
        self.count == SOURCE_BATCH
    }
    pub fn push(&mut self, report: SourceReport) -> Option<()> {
        if self.is_full() || report.buttons > 7 {
            return None;
        }
        self.reports[self.count] = report;
        self.count += 1;
        Some(())
    }
    pub fn reports(&self) -> &[SourceReport] {
        &self.reports[..self.count]
    }
    pub fn words(&self) -> [u64; 6] {
        let mut words = [DEVICE_BATCH, self.count as u64, 0, 0, 0, 0];
        for (out, report) in words[2..].iter_mut().zip(self.reports()) {
            *out = u64::from(report.dx as u16)
                | (u64::from(report.dy as u16) << 16)
                | (u64::from(report.buttons) << 32)
                | (u64::from(report.reset) << 35);
        }
        words
    }
    pub fn decode(words: [u64; 6]) -> Option<Self> {
        if words[0] != DEVICE_BATCH || !(1..=SOURCE_BATCH as u64).contains(&words[1]) {
            return None;
        }
        let count = words[1] as usize;
        if words[2 + count..].iter().any(|&word| word != 0) {
            return None;
        }
        let mut batch = Self::default();
        for &word in &words[2..2 + count] {
            if word >> 36 != 0 {
                return None;
            }
            batch.push(SourceReport {
                dx: word as i16,
                dy: (word >> 16) as i16,
                buttons: ((word >> 32) & 7) as u8,
                reset: word & (1 << 35) != 0,
            })?;
        }
        Some(batch)
    }
}
pub const MOTION: u64 = 1;
pub const BUTTONS: u64 = 2;
pub const CANCEL: u64 = 3;
pub const NONE: u64 = 0;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Event {
    pub pointer: u64,
    pub kind: u64,
    pub x: i64,
    pub y: i64,
    pub buttons: u8,
}
impl Event {
    pub fn words(self) -> [u64; 6] {
        [
            0,
            self.pointer,
            self.kind,
            self.x as u64,
            self.y as u64,
            u64::from(self.buttons),
        ]
    }
    pub fn decode(words: [u64; 6]) -> Option<Self> {
        if words[0] != 0 || words[1] > 1 || words[2] > CANCEL || words[5] > 7 {
            return None;
        }
        Some(Self {
            pointer: words[1],
            kind: words[2],
            x: words[3] as i64,
            y: words[4] as i64,
            buttons: words[5] as u8,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_batch_preserves_order_signed_motion_edges_and_reset() {
        let reports = [
            SourceReport {
                dx: i16::MIN,
                dy: i16::MAX,
                buttons: 1,
                reset: false,
            },
            SourceReport {
                dx: 1,
                dy: -1,
                buttons: 0,
                reset: false,
            },
            SourceReport {
                dx: 0,
                dy: 0,
                buttons: 7,
                reset: false,
            },
            SourceReport {
                reset: true,
                ..Default::default()
            },
        ];
        let mut batch = SourceBatch::default();
        for report in reports {
            batch.push(report).unwrap();
        }
        assert!(batch.push(SourceReport::default()).is_none());
        assert_eq!(
            SourceBatch::decode(batch.words()).unwrap().reports(),
            reports
        );
    }
    #[test]
    fn malformed_source_batch_has_no_decoded_prefix() {
        for words in [
            [DEVICE_BATCH, 0, 0, 0, 0, 0],
            [DEVICE_BATCH, 5, 0, 0, 0, 0],
            [DEVICE_BATCH, u64::MAX, 0, 0, 0, 0],
            [DEVICE_BATCH, 1, 0, 1, 0, 0],
            [DEVICE_BATCH, 2, 123, 1 << 36, 0, 0],
            [DEVICE, 1, 0, 0, 0, 0],
        ] {
            assert!(SourceBatch::decode(words).is_none());
        }
        let mut batch = SourceBatch::default();
        assert!(
            batch
                .push(SourceReport {
                    buttons: 8,
                    ..Default::default()
                })
                .is_none()
        );
        assert!(batch.is_empty());
    }
}
