// Omega selects owner child-control work; this host retains exact requests and resources.
use super::*;
use cathedral_stock_matrix::policy::{
    admission,
    control::{self as policy, Operation},
};
impl Children {
    pub(in crate::service) fn dispatch(
        &mut self,
        request: &Request,
        parent: &mut Parent,
        words: wire::Packet,
        server: &mut Server,
        world: &mut World,
    ) -> Result<bool, Error> {
        let mut facts = policy::Facts {
            owner: request.control,
            loss: self.loss.is_some(),
            launching: self.launching.is_some(),
            storing: parent.storing(),
            ..Default::default()
        };
        for slot in 0..CHILDREN {
            facts.occupied |= u64::from(self.slots[slot].is_some()) << slot;
            facts.unavailable |= u64::from(!server.available(slot)) << slot;
            facts.pending |= u64::from(self.pending[slot].is_some()) << slot;
            facts.standing |= u64::from(self.standing[slot].is_some()) << slot;
        }
        let decision = policy::decide(words, CHILDREN, facts);
        let result = match decision.operation {
            Operation::Unhandled => return Ok(false),
            Operation::Survey => {
                self.survey_loss(*request, parent, server)?;
                return Ok(true);
            }
            Operation::Stored => {
                self.owner_stored(request, words, parent, server)?;
                return Ok(true);
            }
            Operation::Forward => match self.forward(words, words[0] >> wire::OP_BITS) {
                Ok(slot) => {
                    assert_eq!(slot, decision.slot);
                    self.pending[slot] =
                        Some((*request, time::after(decision.deadline)?, Purpose::Owner));
                    self.retiring[slot] = None;
                    return Ok(true);
                }
                Err(error) => Err(error),
            },
            operation => {
                self.owner_operation(operation, decision.slot, words, parent, server, world)
            }
        };
        if result.is_ok() {
            self.recovery_owner_changed(words);
            self.launch_owner_changed(words);
        }
        server.reply(
            request,
            &wire::encode(wire::error(result.err().map_or(0, |e| e.0 as u64))),
        )?;
        Ok(true)
    }
    fn owner_operation(
        &mut self,
        operation: Operation,
        slot: usize,
        words: wire::Packet,
        parent: &mut Parent,
        server: &mut Server,
        world: &mut World,
    ) -> Result<(), Error> {
        use Operation::*;
        match operation {
            Deny => Err(Error(abi::DENIED as i64)),
            Busy => Err(Error(abi::BUSY as i64)),
            Invalid => Err(Error(abi::INVALID_ARGUMENT as i64)),
            Images => {
                (0..abi::MAX_LAUNCHES)
                    .filter(|i| words[2] & (1 << i) != 0)
                    .try_for_each(|i| {
                        cathedral_user_runtime::task::Executable::at(i as u64).map(|_| ())
                    })?;
                self.allowed_images[slot] = words[2];
                Ok(())
            }
            Fallback => self.set_fallback(words[1], words[2]),
            Management => {
                if words[2] == 0 {
                    self.management.revoke(slot);
                    Ok(())
                } else {
                    let incarnation = server.incarnation(slot).ok_or(Error(abi::DENIED as i64))?;
                    self.management
                        .grant(slot, incarnation, (words[2] - 1) as usize)
                        .map_err(|c| Error(c as i64))
                }
            }
            Configure => self.configure(parent, world, server),
            Restore => self.restore_storage(parent, server),
            Imports => {
                self.allowed_imports[slot] = Some(words[2]);
                Ok(())
            }
            Pages => {
                self.next_pages[slot] = Some([words[2], words[3]]);
                Ok(())
            }
            Memory => {
                self.next_frames[slot] = Some(words[2]);
                Ok(())
            }
            Policy => {
                let place = self.standing[slot].and_then(|s| s.place);
                self.standing[slot] = Some(Standing {
                    pages: [words[2], words[3]],
                    channels: words[4],
                    descendants: words[5],
                    cpu: words[6],
                    surfaces: words[7],
                    place,
                });
                Ok(())
            }
            Placement => {
                self.standing[slot]
                    .as_mut()
                    .expect("authorized standing policy")
                    .place = Some((
                    Rect {
                        x: words[2],
                        y: words[3],
                        width: words[4],
                        height: words[5],
                    },
                    words[6],
                ));
                Ok(())
            }
            Spawn => self.start(slot, words[2], words[3], words[4], server, world),
            Run => self.run(
                slot,
                [words[2], words[3], words[4], words[5], words[6], words[7]],
                server,
                world,
            ),
            Stop => {
                self.launch_owner_changed(words);
                if slot == 0 {
                    self.launcher = None;
                }
                self.stop(slot, server, world, parent)
            }
            _ => Err(Error(abi::IO_ERROR as i64)),
        }
    }
    // Read/admit is asynchronous: begin_launch retains this exact owner request.
    fn owner_stored(
        &mut self,
        request: &Request,
        words: wire::Packet,
        parent: &mut Parent,
        server: &mut Server,
    ) -> Result<(), Error> {
        let [_, slot, object, scope, argument, channels, descendants, cpu] = words;
        let slot = slot as usize;
        // Recognize the owner's initial Login using only this matrix's realm.
        if admission::inspect_picker(parent.upstream, slot, scope) {
            let named = parent.local_storage(|realm| {
                realm.bind().map_err(|c| Error(c as i64))?;
                Ok((realm.lookup(b"Login").ok(), realm.lookup(b"Users").ok()))
            });
            if let Ok((login, users)) = named
                && admission::matches_picker(login, users, object, scope)
            {
                self.login = true;
                self.entered = false;
                self.startup_recovery.owner_recovers = true;
            }
        }
        self.begin_launch(
            *request,
            parent::LaunchRequest {
                slot,
                object,
                scope,
                argument,
                channels,
                descendants,
                cpu,
                pages: None,
                stamp: 0,
                imports: self.allowed_imports[slot]
                    .unwrap_or(cathedral_platform_contracts::imports::ALL),
            },
            parent,
            server,
        )
    }
}
