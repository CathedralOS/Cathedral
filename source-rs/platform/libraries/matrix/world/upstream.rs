//! Apply one parent snapshot before routing its local input into this world.
use super::*;
impl<const N: usize> World<N> {
    pub fn inherit_poll(&mut self, reply: wire::Packet) -> Grant {
        let grant = if reply[0] == 0 {
            Grant {
                epoch: reply[1],
                width: reply[2],
                height: reply[3],
                keys: reply[4] & wire::VALID_KEY_MASK,
            }
        } else {
            Grant::default()
        };
        self.inherit(grant);
        self.surface_capacity(if grant.valid() {
            (reply[4] >> wire::SURFACE_BITS) as usize
        } else {
            0
        });
        if grant.valid() {
            let event = wire::poll_event(reply);
            let channel = u32::try_from(event[6]).unwrap_or(0);
            let result = match event[2] {
                wire::NONE => Ok(()),
                wire::POINTER => self.pointer_stream(channel, event[3], event[4], event[5]),
                wire::KEY => self.key_stream(channel, event[3], event[4]),
                wire::CANCEL => self.inherit_stream_cancel(channel),
                _ => Err(abi::INVALID_ARGUMENT),
            };
            if result.is_err() {
                self.inherit(Grant::default());
                return Grant::default();
            }
        }
        grant
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn revoked_parent_cannot_replay_input_or_revive_child_authority() {
        let mut world = World::<2>::empty();
        let snapshot = [
            0,
            1,
            300,
            200,
            wire::KEY_MASK | (8 << wire::SURFACE_BITS),
            0,
            0,
            0,
        ];
        world.inherit_poll(snapshot);
        world
            .place(
                0,
                Rect {
                    x: 50,
                    y: 40,
                    width: 100,
                    height: 80,
                },
                wire::KEY_MASK,
            )
            .unwrap();
        world.surface_limit(0, 4).unwrap();
        let view = world.request(0, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(&view[2..4], &[100, 80]);
        world.create_surface(0, 0, 30);
        world.inherit_poll(wire::error(abi::DENIED));
        assert_eq!(world.backed_surfaces(), 0);
        assert_eq!(
            world.request(0, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[0],
            abi::DENIED
        );
        assert!(world.take_retired_surface().is_some());
        let mut replacement = snapshot;
        replacement[1] = 2;
        world.inherit_poll(replacement);
        assert_eq!(
            world.request(0, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[0],
            abi::DENIED
        );
        world
            .place(
                0,
                Rect {
                    x: 0,
                    y: 0,
                    width: 50,
                    height: 40,
                },
                0,
            )
            .unwrap();
        let fresh = world.request(0, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        assert_ne!(fresh[1], view[1]);
        assert_eq!(&fresh[2..4], &[50, 40]);
    }
}
