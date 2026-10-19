//! Enforce delegated task management using the authenticated interaction stream.
//! The slot is owner policy; the ticket resolves only to its exact observed task.
use super::*;
impl Children {
    pub(in crate::service) fn managed_task(&mut self, request: &Request) -> wire::Packet {
        let manager = request.client.unwrap();
        match self.management.target(manager, request.incarnation) {
            Ok(target) => self.management.query(
                manager,
                request.incarnation,
                self.slots[target].as_ref().map(|child| child.task.raw()),
            ),
            Err(code) => wire::error(code),
        }
    }
    pub(in crate::service) fn terminate_task(
        &mut self,
        request: &Request,
        ticket: u64,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<wire::Packet, Error> {
        let manager = request.client.unwrap();
        let target = match self
            .management
            .target(manager, request.incarnation)
            .and_then(|slot| {
                self.management.authorize(
                    manager,
                    request.incarnation,
                    ticket,
                    self.slots[slot].as_ref().map(|child| child.task.raw()),
                )
            }) {
            Ok(target) => target,
            Err(code) => return Ok(wire::error(code)),
        };
        if self.loss.is_some() || parent.storing() || parent.upstream_busy() {
            return Ok(wire::error(abi::BUSY));
        }
        if target == 0 {
            self.launcher = None;
        }
        self.stop(target, server, world, parent)?;
        cathedral_user_runtime::write(b"Cathedral matrix: managed task stopped\n")?;
        Ok([0; 8])
    }
    pub(in crate::service) fn management_owner_changed(&mut self, words: wire::Packet) {
        match words {
            [wire::REVOKE, slot @ 0..=LAST_CHILD, 0, 0, 0, 0, 0, 0] => {
                self.management.revoke(slot as usize)
            }
            [wire::BIND, ..] => {
                for slot in 0..CHILDREN {
                    self.management.revoke(slot);
                }
            }
            _ => (),
        }
    }
}
