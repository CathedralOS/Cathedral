//! Validate every launch loan before admission; commit without allocation after it.
use super::{Session, abi, next_epoch};

pub(super) struct Plan {
    loans: [Option<(usize, u64, u64)>; abi::MAX_LAUNCHES],
}
impl Plan {
    pub(super) fn prepare(
        session: &Session,
        owner: usize,
        child: usize,
        handles: &[u64],
    ) -> Result<Self, u64> {
        let mut indices = [0; abi::MAX_LAUNCHES];
        for (position, &handle) in handles.iter().enumerate() {
            let index = session
                .launches
                .iter()
                .position(|launch| launch.model.launch(owner) == Ok(handle))
                .ok_or(abi::BAD_HANDLE)?;
            if indices[..position].contains(&index) {
                return Err(abi::INVALID_ARGUMENT);
            }
            session.launches[index]
                .model
                .can_lend(owner, handle, child)?;
            indices[position] = index;
        }
        let mut loans = [None; abi::MAX_LAUNCHES];
        for (loan, &index) in loans.iter_mut().zip(&indices[..handles.len()]) {
            // Reserve reclamation's name now, so teardown cannot fail for lack of
            // handle space. Failed admission burns names but retains authority.
            let epoch = next_epoch().ok_or(abi::NO_MEMORY)?;
            let returned = next_epoch().ok_or(abi::NO_MEMORY)?;
            *loan = Some((index, epoch, returned));
        }
        Ok(Self { loans })
    }
    pub(super) fn commit(self, session: &mut Session, child: usize) {
        for (index, epoch, returned) in self.loans.into_iter().flatten() {
            session.launches[index].model.lend(child, epoch, returned);
        }
    }
}
