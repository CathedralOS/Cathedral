//! Validate every launch loan before admission; commit without allocation after it.
use super::{Session, abi};

#[derive(Clone, Copy)]
struct Loan {
    index: usize,
    names: [u64; 2],
    returned: [u64; 2],
}
pub(super) struct Plan {
    loans: [Option<Loan>; abi::MAX_LAUNCHES],
}
impl Plan {
    pub(super) fn prepare(
        session: &mut Session,
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
            let names = session.names.pair(child, child)?;
            let returned = session.names.pair(owner, owner)?;
            *loan = Some(Loan {
                index,
                names,
                returned,
            });
        }
        Ok(Self { loans })
    }
    pub(super) fn commit(self, session: &mut Session, child: usize) {
        for Loan {
            index,
            names,
            returned,
        } in self.loans.into_iter().flatten()
        {
            session.launches[index].images.fill(None);
            session.launches[index].model.lend(child, names, returned);
            session
                .pool
                .reattach(session.launches[index].model.child, child);
            // The lent account is dormant; its future descendants subdivide the
            // borrower's turns rather than competing with it at the outer level.
            session
                .scheduler
                .set_parent(session.launches[index].model.child, child);
        }
    }
}
