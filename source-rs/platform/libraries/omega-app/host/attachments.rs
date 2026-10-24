//! One app-charged object. Failed staging/publication drops producer custody.
use crate::{PACKET_WORDS, transfer::Transfer};
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    memory::{Object, Private},
};

pub(super) enum Staged {
    Writing { pages: Private, progress: Transfer },
    Ready(Object),
}

pub(super) fn append(slot: &mut Option<Staged>, packet: &[u64; PACKET_WORDS]) -> Result<(), Error> {
    let invalid = || Error(abi::INVALID_ARGUMENT as i64);
    // Take before validating: every failure clears an unfinished/ready object.
    let previous = slot.take();
    let bytes = super::exchange::payload(packet)?;
    let (mut pages, mut progress) = match previous {
        Some(Staged::Writing { pages, progress }) => (pages, progress),
        Some(Staged::Ready(_)) => return Err(invalid()),
        None => {
            let progress = Transfer {
                total: usize::try_from(packet[0]).map_err(|_| invalid())?,
                filled: 0,
            };
            progress
                .chunk(packet[0], packet[1], bytes.len())
                .ok_or_else(invalid)?;
            (Private::allocate(progress.total.div_ceil(4096))?, progress)
        }
    };
    let range = progress
        .chunk(packet[0], packet[1], bytes.len())
        .ok_or_else(invalid)?;
    progress.filled = range.end;
    pages.bytes_mut()[range].copy_from_slice(bytes);
    *slot = Some(if progress.filled == progress.total {
        let mut object = pages.freeze()?;
        object.unmap()?;
        Staged::Ready(object)
    } else {
        Staged::Writing { pages, progress }
    });
    Ok(())
}

pub(super) fn take(
    slot: &mut Option<Staged>,
    packet: &[u64; PACKET_WORDS],
) -> Result<Option<Object>, Error> {
    match slot.take() {
        None => super::exchange::attachment(packet),
        Some(Staged::Ready(object)) if packet[8] == 0 => Ok(Some(object)),
        _ => Err(Error(abi::INVALID_ARGUMENT as i64)),
    }
}
