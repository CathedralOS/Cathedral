//! A nested producer reserves local capacity only after its parent accepts a
//! smaller incoming ceiling. Retirement restores the available upstream space.
use super::*;
impl Fixture {
    pub(super) fn nested_producer(&self) {
        let epoch = self.view(2)[1];
        self.pointer(115, 127, 0);
        assert_eq!(self.next(2, epoch)[2], wire::POINTER);
        self.owner(
            1,
            [
                wire::INPUT_PRODUCER_GRANT,
                0,
                2,
                wire::PRODUCER_FOCUS,
                0,
                0,
                0,
                0,
            ],
        );
        let info = self.query(2, [wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(&info[2..4], &[200, 160]);
        self.owner(2, [wire::PRODUCER_POINTER, info[1], 1, 15, 27, 1, 0, 0]);
        let local = self.next(2, epoch);
        assert_eq!(&local[2..6], &[wire::POINTER, 5, 7, 1]);
        let group = self.query(0, [wire::INPUT_GROUP_CREATE, 0, 0, 0, 0, 0, 0, 0])[1];
        let first = self.query(0, [wire::INPUT_CHANNEL_CREATE, group, 0, 0, 0, 0, 0, 0])[1];
        let second = self.query(0, [wire::INPUT_CHANNEL_CREATE, group, 0, 0, 0, 0, 0, 0])[1];
        assert!(group != 0 && first != 0 && second != 0);
        self.owner(0, [wire::INPUT_POINTER, first, 115, 127, 1, 0, 0, 0]);
        let inherited = self.next(2, epoch);
        assert_eq!(inherited[2], wire::POINTER);
        assert_ne!(inherited[7], local[7]);
        assert_eq!(
            self.query(0, [wire::INPUT_POINTER, second, 115, 127, 1, 0, 0, 0])[0],
            abi::NO_MEMORY
        );
        assert_eq!(self.view(2)[1], epoch);
        self.owner(
            2,
            [
                wire::PRODUCER_KEY,
                info[1],
                2,
                input::ENTER as u64,
                input::PRESS as u64,
                0,
                0,
                0,
            ],
        );
        assert_eq!(self.next(2, epoch)[2], wire::KEY);
        for channel in [first, second] {
            self.owner(0, [wire::INPUT_CHANNEL_RETIRE, channel, 0, 0, 0, 0, 0, 0]);
        }
        assert_eq!(self.next(2, epoch)[2], wire::RETIRED);
        self.owner(0, [wire::INPUT_GROUP_RELEASE, group, 0, 0, 0, 0, 0, 0]);
        assert_eq!(self.next(2, epoch)[2], wire::GROUP_RETIRED);
        self.owner(2, [wire::PRODUCER_RELEASE, info[1], 0, 0, 0, 0, 0, 0]);
        assert_eq!(self.next(2, epoch)[2], wire::RETIRED);
        assert_eq!(self.next(2, epoch)[2], wire::RETIRED);
        assert_eq!(self.next(2, epoch)[2], wire::GROUP_RETIRED);
        // The following root producer scenario needs three inherited channels
        // including the existing source; it proves automatic widening to four.
        cathedral_user_runtime::write(
            b"Cathedral matrix: nested producer capacity negotiation passed\n",
        )
        .unwrap();
    }
}
