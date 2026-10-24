//! Checked extents shared by the native host and its host-side boundary tests.
use crate::PAYLOAD_BYTES;
use cathedral_platform_contracts::storage;

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

/// ENTRIES has a row count, not BULK's byte-count field. Empty replies carry no
/// backing; a nonempty cursor must advance. Decode rows only after this envelope.
pub(crate) fn entries(
    limit: u64,
    after: u64,
    count: u64,
    last: u64,
    backing: Option<usize>,
) -> Option<usize> {
    if limit == 0 || limit > PAYLOAD_BYTES as u64 || count > storage::ENTRIES_PER_PAGE as u64 {
        return None;
    }
    if count == 0 {
        return backing.is_none().then_some(0);
    }
    if last <= after {
        return None;
    }
    let bytes = count.checked_mul(storage::ENTRY_BYTES as u64)?;
    received(limit, 4096, bytes, backing?)
}

/// Canonical metadata and strict cursor ordering are part of the wire contract.
pub(crate) fn entry_rows(bytes: &[u8], after: u64, last: u64) -> Option<()> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(storage::ENTRY_BYTES) {
        return None;
    }
    let mut previous = after;
    for row in bytes.chunks_exact(storage::ENTRY_BYTES) {
        let row = storage::Entry::decode(row)?;
        if row.object <= previous {
            return None;
        }
        previous = row.object;
    }
    (previous == last).then_some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_reject_stale_ids_and_noncanonical_metadata() {
        let mut bytes = [0u8; 2 * storage::ENTRY_BYTES];
        for (index, row) in bytes.chunks_exact_mut(storage::ENTRY_BYTES).enumerate() {
            row.copy_from_slice(
                &storage::Entry {
                    object: 5 + index as u64,
                    ..Default::default()
                }
                .encode(),
            );
        }
        assert_eq!(entry_rows(&bytes, 4, 6), Some(()));
        assert_eq!(entry_rows(&bytes, 5, 6), None);
        assert_eq!(entry_rows(&bytes, 4, 7), None);
        bytes[storage::ENTRY_BYTES..storage::ENTRY_BYTES + 8].copy_from_slice(&5u64.to_le_bytes());
        assert_eq!(entry_rows(&bytes, 4, 5), None);
        bytes[17] = b'x'; // Nonzero data after an empty name's terminator.
        assert_eq!(entry_rows(&bytes[..storage::ENTRY_BYTES], 4, 5), None);
    }

    #[test]
    fn entry_pages_bound_rows_padding_and_cursor_progress() {
        assert_eq!(entries(4080, 4, 51, 55, Some(4096)), Some(4080));
        assert_eq!(entries(4080, 55, 0, 55, None), Some(0));
        for (limit, after, count, last, backing) in [
            (0, 0, 1, 1, Some(4096)),
            (79, 0, 1, 1, Some(4096)),
            (4080, 0, 52, 52, Some(8192)),
            (4080, 0, u64::MAX, 1, Some(4096)),
            (4080, 3, 1, 3, Some(4096)),
            (4080, 4, 1, 3, Some(4096)),
            (4080, 0, 1, 1, Some(79)),
            (4080, 0, 1, 1, None),
            (4080, 0, 0, 0, Some(4096)),
            (u64::MAX, 0, 1, 1, Some(4096)),
        ] {
            assert_eq!(entries(limit, after, count, last, backing), None);
        }
    }

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
