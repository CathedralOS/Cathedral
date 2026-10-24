//! Checked extents shared by the native host and its host-side boundary tests.
use crate::PAYLOAD_BYTES;

pub(crate) const MAX_STAGED_BYTES: usize = 64 * 1024;

/// A staged object accepts a contiguous prefix only; its size cannot change.
#[derive(Clone, Copy)]
pub(crate) struct Transfer {
    pub total: usize,
    pub filled: usize,
}
impl Transfer {
    pub fn chunk(self, total: u64, offset: u64, length: usize) -> Option<core::ops::Range<usize>> {
        if total != self.total as u64
            || self.total == 0
            || self.total > MAX_STAGED_BYTES
            || offset != self.filled as u64
            || length == 0
            || length > PAYLOAD_BYTES
        {
            return None;
        }
        let end = self.filled.checked_add(length)?;
        (end <= self.total).then_some(self.filled..end)
    }
}

/// BULK's extra field is the returned byte count, never the page-rounded size.
pub(crate) fn received(limit: u64, requested: u64, returned: u64, backing: usize) -> Option<usize> {
    let limit = usize::try_from(limit).ok()?;
    let returned = usize::try_from(returned).ok()?;
    (limit != 0
        && limit <= PAYLOAD_BYTES
        && returned != 0
        && returned <= limit
        && returned as u64 <= requested
        && returned <= backing)
        .then_some(returned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_large_bitmap_requires_complete_ordered_chunks() {
        let mut transfer = Transfer {
            total: 46080,
            filled: 0,
        };
        let first = transfer.chunk(46080, 0, PAYLOAD_BYTES).unwrap();
        transfer.filled = first.end;
        assert!(transfer.chunk(46080, 0, 100).is_none());
        assert!(transfer.chunk(46081, transfer.filled as u64, 100).is_none());
        assert!(
            transfer
                .chunk(46080, transfer.filled as u64 + 1, 100)
                .is_none()
        );
        let last = transfer
            .chunk(46080, transfer.filled as u64, 46080 - transfer.filled)
            .unwrap();
        transfer.filled = last.end;
        assert_eq!(transfer.filled, transfer.total);
        assert!(transfer.chunk(46080, 46080, 1).is_none());
    }

    #[test]
    fn oversized_and_empty_transfers_are_refused() {
        for total in [0, MAX_STAGED_BYTES + 1, usize::MAX] {
            assert!(
                Transfer { total, filled: 0 }
                    .chunk(total as u64, 0, 1)
                    .is_none()
            );
        }
        let transfer = Transfer {
            total: MAX_STAGED_BYTES,
            filled: 0,
        };
        assert!(transfer.chunk(MAX_STAGED_BYTES as u64, 0, 0).is_none());
        assert!(
            transfer
                .chunk(MAX_STAGED_BYTES as u64, 0, PAYLOAD_BYTES + 1)
                .is_none()
        );
    }

    #[test]
    fn bulk_never_exposes_padding_or_unrequested_bytes() {
        assert_eq!(received(244, 5, 5, 4096), Some(5));
        for (limit, request, reply, backing) in [
            (0, 5, 5, 4096),
            (244, 4, 5, 4096),
            (244, 245, 245, 4096),
            (244, 5, 5, 4),
            (244, 5, 0, 4096),
            (u64::MAX, 5, 5, 4096),
            (244, u64::MAX, u64::MAX, 4096),
        ] {
            assert_eq!(received(limit, request, reply, backing), None);
        }
    }
}
