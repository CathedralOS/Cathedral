//! Ancestor revoke/resize and replacement require fresh descendant authorization.
use super::*;
impl Fixture {
    pub(super) fn recovery(&mut self) {
        let old = self.view(2)[1];
        self.pointer(115, 127, 1); // Leave an event queued above the child.
        self.owner(0, [wire::REVOKE, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(self.view(2), wire::error(abi::DENIED));
        assert_eq!(self.next(2, old), wire::error(abi::DENIED));
        self.owner(0, [wire::PLACE, 0, 100, 100, 40, 30, wire::KEY_MASK, 0]);
        assert_eq!(self.view(2), wire::error(abi::DENIED));
        self.owner(
            1,
            [wire::PLACE, 0, 10, 20, 60, 40, 1 << (input::ENTER - 1), 0],
        );
        let resized = self.view(2);
        assert_eq!(&resized[2..4], &[30, 10]);
        assert_eq!(self.next(2, old), wire::error(abi::DENIED));
        assert_eq!(self.next(2, resized[1])[2], wire::NONE);
        // Release outside the newly scoped world, then a new press must work there.
        self.pointer(799, 599, 0);
        self.pointer(115, 127, 1);
        assert_eq!(
            &self.next(2, resized[1])[2..],
            &[wire::POINTER, 5, 7, 1, 1, 0]
        );
        self.pointer(115, 127, 0);
        assert_eq!(self.next(2, resized[1])[2], wire::POINTER);
        // Outer matrix cancels its child. Kernel tears down the inner leaf too;
        // the fixture has neither child's launch, process handle nor private port.
        let sibling = self.view(3);
        self.owner(0, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            self.query(1, [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::PEER_CLOSED)
        );
        self.admit_inner();
        // Observe disconnect before authorizing the replacement host.
        self.owner(0, [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(self.view(2), wire::error(abi::DENIED));
        assert_eq!(self.view(3), sibling);
        self.layout();
        assert_eq!(self.view(2)[2], 60);
        // The sibling's own placement may change in layout, but its process survives.
        assert_eq!(sibling[0], 0);
        assert_eq!(self.view(3)[0], 0);
        self.pointer(799, 599, 0); // Release the physical hold from the revoked binding.
        self.input();
        // A leaf exits while its parent is forwarding a control RPC. Complete the
        // pending reply, reclaim the leaf, and preserve its matrix and sibling.
        assert_eq!(
            self.query(2, [crate::leaf::EXIT_PROBE, 0, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::PEER_CLOSED)
        );
        self.owner(1, [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]);
        self.owner(1, [wire::SPAWN, 0, 0, 0, 0, 0, 0, 0]);
        self.layout();
        assert_eq!(self.view(2)[0], 0);
        assert_eq!(self.view(3)[0], 0);
    }
}
