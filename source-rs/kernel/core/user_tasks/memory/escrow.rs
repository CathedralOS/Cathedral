//! Finite IPC-discard snapshots retain reader slots until each reference is dropped.
use super::*;

pub(super) unsafe fn step(session: &mut Session, frames: &mut Frames<'_>) {
    if !session.memory.escrow_active {
        // Snapshot a finite wave. New discards wait in IPC until this wave ends.
        session.memory.escrow_slots = session.ipc.take_retired_memory();
        session.memory.escrow_active = true;
    }
    if session.memory.escrow_object.is_none() {
        let next = session
            .memory
            .escrow_slots
            .iter()
            .enumerate()
            .find_map(|(word, &bits)| {
                (bits != 0).then(|| word * 64 + bits.trailing_zeros() as usize)
            });
        if let Some(slot) = next {
            session.memory.escrow_slots[slot / 64] &= !(1 << (slot % 64));
            if let Some(grant) = session.memory.model.reads[slot] {
                assert!(!grant.published && grant.mapping.is_none());
            }
            session.memory.escrow_object = session.memory.model.drop_read(slot);
        }
    }
    if let Some(object) = session.memory.escrow_object {
        // SAFETY: Escrow never published a mapping. Its cursor pins any retiring
        // metadata slot until every physical frame is refunded.
        if unsafe { session.memory.reclaim_step(object, frames) } {
            session.memory.escrow_object = None;
        }
    }
    session.memory.escrow_active = session.memory.escrow_object.is_some()
        || session.memory.escrow_slots.iter().any(|&bits| bits != 0);
}
