// Recovery authority follows the interaction chain. Only the root owner
// replaces a provider; descendants can queue one attempt through explicit grants.
use super::*;
use cathedral_user_runtime::server::Request;
pub(super) struct Recovery {
    authority: authority::Authority,
    state: u64,
}
impl Recovery {
    pub const fn empty() -> Self {
        Self {
            authority: authority::Authority::empty(),
            state: wire::SAVES_READY,
        }
    }
}
impl Parent {
    pub(in crate::service) fn grant_recovery(&mut self, slot: usize, incarnation: Option<u64>) {
        self.recovery.authority.grant(slot, incarnation);
    }
    pub(in crate::service) fn recovery_owner(
        &mut self,
        server: &Server,
        words: wire::Packet,
    ) -> wire::Packet {
        match words {
            [
                wire::SAVE_RECOVERY_GRANT,
                slot @ 0..=LAST_CHILD,
                enabled @ 0..=1,
                0,
                0,
                0,
                0,
                0,
            ] => {
                let incarnation = if enabled == 1 {
                    let Some(i) = server.incarnation(slot as usize) else {
                        return wire::error(abi::DENIED);
                    };
                    Some(i)
                } else {
                    None
                };
                self.grant_recovery(slot as usize, incarnation);
                [0; 8]
            }
            [wire::SAVE_RECOVERY_POLL, state @ 0..=2, 0, 0, 0, 0, 0, 0] if !self.upstream => {
                self.recovery.state = state;
                let requested = self
                    .recovery
                    .authority
                    .take(|slot| server.incarnation(slot))
                    && state == wire::SAVES_OFFLINE;
                [0, u64::from(requested), 0, 0, 0, 0, 0, 0]
            }
            _ => wire::error(abi::DENIED),
        }
    }
    pub(in crate::service) fn recovery_request(
        &mut self,
        request: &Request,
        retry: bool,
    ) -> Result<wire::Packet, Error> {
        let Some(slot) = request.client.filter(|slot| *slot < CHILDREN) else {
            return Ok(wire::error(abi::DENIED));
        };
        if !self.recovery.authority.allowed(slot, request.incarnation) {
            return Ok(wire::error(abi::DENIED));
        }
        if self.upstream {
            return self.call([
                if retry {
                    wire::RETRY_SAVES
                } else {
                    wire::SAVE_AVAILABILITY
                },
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]);
        }
        if retry
            && (self.recovery.state != wire::SAVES_OFFLINE
                || !self.recovery.authority.request(slot, request.incarnation))
        {
            return Ok(wire::error(abi::BUSY));
        }
        Ok([0, self.recovery.state, 0, 0, 0, 0, 0, 0])
    }
}
