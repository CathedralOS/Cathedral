//! Supply authenticated launch facts to Omega, then retain the selected local grants.
use super::*;
use cathedral_stock_matrix::policy::admission;
impl Children {
    /// A child's request for a sibling. The other slot must be free and opened
    /// by the owner; the program is read with the requester's own scope, so a
    /// launcher can only launch what it can see.
    pub(in crate::service) fn launch(
        &mut self,
        request: Request,
        object: u64,
        withhold: u64,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let requester = request.client.filter(|s| *s < CHILDREN);
        let binding = requester.and_then(|s| parent.storage_binding(server, s));
        let mut facts = admission::Facts {
            busy: self.launching.is_some() || self.loss.is_some(),
            desktop: self.desktop,
            chrome: self.chrome(&request, world),
            requester,
            occupied: 0,
            unavailable: 0,
            storing: parent.storing(),
            receipts: requester.is_some_and(|s| self.observations.available(s)),
            tickets: self.catalogue.has_ticket_capacity(),
            standing: 0,
            binding: binding.is_some(),
        };
        for slot in 0..CHILDREN {
            facts.occupied |= u64::from(self.slots[slot].is_some()) << slot;
            facts.unavailable |= u64::from(!server.available(slot)) << slot;
            facts.standing |= u64::from(self.standing[slot].is_some()) << slot;
        }
        let decision = admission::decide(object, withhold, facts);
        if decision.status != 0 {
            if decision.full {
                time::storage_sample("funding-local-slots-refused", &[3]);
            }
            return server.reply(&request, &wire::encode(wire::error(decision.status)));
        }
        let slot = decision.slot;
        let standing = self.standing[slot].expect("authorized standing policy");
        let (_, stamp) = binding.expect("authenticated storage binding");
        self.begin_launch(
            request,
            parent::LaunchRequest {
                slot,
                object,
                scope: object,
                argument: 0,
                channels: standing.channels,
                descendants: standing.descendants,
                cpu: standing.cpu,
                pages: Some(standing.pages),
                stamp,
                imports: self.allowed_imports[slot]
                    .unwrap_or(cathedral_platform_contracts::imports::ALL)
                    & !withhold,
            },
            parent,
            server,
        )
    }
}
