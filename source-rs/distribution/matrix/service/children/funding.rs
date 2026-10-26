//! Fund a local admission or a live child's request through its immediate owner.
//! Kernel transfers are atomic per edge. An upstream grant remains ours until
//! reused or subtree teardown, even if a later downstream transfer is refused.
use super::*;
use cathedral_matrix::funding::{Grant as Funding, shortfall};
use cathedral_user_runtime::{resources, task};

fn available() -> Result<[u64; 5], Error> {
    let b = resources::budget()?;
    let f = resources::physical_budget()?;
    Ok([
        b.limits.private_pages - b.used.private_pages,
        b.limits.reader_pages - b.used.reader_pages,
        b.limits.channels - b.used.channels,
        task::available_slots()?,
        f.reserved - f.charged,
    ])
}

/// Stock policy: bounded growth for programs admitted with descendant capacity.
/// A declaration supplies initial demand, never this permission or its ceiling.
pub(super) fn policy(funded: [u64; 5]) -> Option<Funding> {
    (funded[3] != 0).then(|| Funding {
        funded,
        ceiling: core::array::from_fn(|i| funded[i].max([960, 3584, 208, 24, 6400][i])),
    })
}

pub(super) fn ensure(parent: &mut Parent, needed: [u64; 5]) -> Result<(), Error> {
    let mut needed = needed;
    // This process must still receive code and replacement images after lending
    // to a child. Admission funding cannot consume the forwarding path itself.
    if needed[1] != 0 {
        needed[1] += 256;
    }
    let delta = shortfall(available()?, needed);
    if delta != [0; 5] {
        time::storage_sample("funding-shortfall", &delta);
        parent.request_funding(delta)?;
    }
    Ok(())
}

impl Children {
    pub(in crate::service) fn fund(
        &mut self,
        request: &Request,
        delta: [u64; 5],
        parent: &mut Parent,
    ) -> Result<wire::Packet, Error> {
        let Some(slot) = request.client.filter(|&s| s < CHILDREN) else {
            return Ok(wire::error(abi::DENIED));
        };
        let Some(child) = self.slots[slot]
            .as_ref()
            .filter(|c| c.data.0.raw() == request.incarnation)
        else {
            return Ok(wire::error(abi::DENIED));
        };
        let (Some(grant), Some(launch)) = (child.funding, child.reservation) else {
            return Ok(wire::error(abi::DENIED));
        };
        let Some(next) = grant.increased(delta) else {
            time::storage_sample("funding-policy-refused", &delta);
            return Ok(wire::error(abi::NO_MEMORY));
        };
        // Transport errors retire this matrix via its service entrance; never
        // reuse a stream with an unknown late reply. Policy refusals are packets.
        if let Err(error) = ensure(parent, delta) {
            if error.0 == abi::NO_MEMORY as i64 || error.0 == abi::DENIED as i64 {
                return Ok(wire::error(abi::NO_MEMORY));
            }
            return Err(error);
        }
        let mut result = resources::grow(launch, delta);
        if result == Err(Error(abi::NO_PROCESS_CAPACITY as i64)) && delta[3] != 0 {
            // A retired picker can leave free slots below this child's ordering
            // boundary. Request additional capacity once; expose no slot IDs.
            let mut extra = [0; 5];
            extra[3] = delta[3];
            match parent.request_funding(extra) {
                Ok(()) => result = resources::grow(launch, delta),
                Err(e) if e.0 == abi::NO_MEMORY as i64 || e.0 == abi::DENIED as i64 => (),
                Err(e) => return Err(e),
            }
        }
        match result {
            Ok(()) => {
                self.slots[slot].as_mut().unwrap().funding = Some(next);
                time::storage_sample("funding-granted", &delta);
                Ok([0; 8])
            }
            Err(error) => {
                time::storage_sample("funding-transfer-refused", &[error.0 as u64]);
                Ok(wire::error(abi::NO_MEMORY))
            }
        }
    }
}
