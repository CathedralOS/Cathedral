//! Compare observable replies while changing only authority outside the leaf.
use super::*;
impl Fixture {
    pub(super) fn confinement(&self) {
        let view = self.view(2);
        assert_eq!(view[0], 0);
        assert_eq!(&view[2..], &[60, 40, 1 << (input::ENTER - 1), 0, 0, 0]);
        for x in [320, 180, 100] {
            self.owner(0, [wire::PLACE, 0, x, 100, 200, 160, wire::KEY_MASK, 0]);
            assert_eq!(self.view(2), view);
        }
        self.owner(0, [wire::REVOKE, 1, 0, 0, 0, 0, 0, 0]);
        self.owner(0, [wire::PLACE, 1, 600, 300, 120, 90, wire::KEY_MASK, 0]);
        assert_eq!(self.view(2), view);
        assert_eq!(self.next(2, view[1])[2], wire::NONE);
        for op in [
            wire::BIND,
            wire::PLACE,
            wire::REVOKE,
            wire::POINTER_SOURCE,
            wire::KEY_SOURCE,
            wire::HEALTH,
            u64::MAX,
        ] {
            assert_eq!(
                call(self.peers[2], [op, 0, 0, 0, 0, 0, 0, 0]),
                wire::error(abi::DENIED)
            );
        }
        assert_eq!(
            call(self.peers[2], [wire::VIEW, 1, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::DENIED)
        );
    }
    pub(super) fn input(&self) {
        let epoch = self.view(2)[1];
        self.pointer(115, 127, 1);
        assert_eq!(&self.next(2, epoch)[2..], &[wire::POINTER, 5, 7, 1, 0, 0]);
        self.pointer(799, 599, 1);
        assert_eq!(&self.next(2, epoch)[2..], &[wire::POINTER, 59, 39, 1, 0, 0]);
        self.pointer(799, 599, 0);
        assert_eq!(&self.next(2, epoch)[2..], &[wire::POINTER, 59, 39, 0, 0, 0]);
        self.key(input::UP, input::PRESS);
        self.key(input::UP, input::RELEASE);
        assert_eq!(self.next(2, epoch)[2], wire::NONE);
        self.key(input::ENTER, input::PRESS);
        assert_eq!(
            &self.next(2, epoch)[2..],
            &[wire::KEY, input::ENTER as u64, input::PRESS as u64, 0, 0, 0]
        );
        self.key(input::ENTER, input::RELEASE);
        assert_eq!(self.next(2, epoch)[2], wire::KEY);
    }
}
