//! An ordinary child receives a narrow producer grant over its authenticated
//! interaction connection. Owner control is never lent to that process.
use super::*;
impl Fixture {
    pub(super) fn producers(&mut self) {
        let epoch = self.view(2)[1];
        self.nested_producer();
        self.view(3); // Establish the outside sibling's current connection.
        assert_eq!(
            self.query(3, [wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0])[0],
            abi::DENIED
        );
        // Two fixed adapter channels are reserved before dynamic grants.
        assert_eq!(
            self.query(
                0,
                [
                    wire::INPUT_PRODUCER_GRANT,
                    1,
                    3,
                    wire::PRODUCER_FOCUS,
                    0,
                    0,
                    0,
                    0
                ]
            )[0],
            abi::NO_MEMORY
        );
        self.owner(
            0,
            [
                wire::INPUT_PRODUCER_GRANT,
                1,
                2,
                wire::PRODUCER_FOCUS,
                0,
                0,
                0,
                0,
            ],
        );
        let info = self.query(3, [wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(info[0], 0);
        assert_eq!(
            &info[2..],
            &[800, 600, wire::KEY_MASK, 2, wire::PRODUCER_FOCUS, 0]
        );
        let token = info[1];
        assert_eq!(
            self.query(2, [wire::PRODUCER_POINTER, token, 1, 115, 127, 1, 0, 0])[0],
            abi::DENIED
        );
        assert_eq!(
            self.query(3, [wire::PRODUCER_POINTER, token, 3, 115, 127, 1, 0, 0])[0],
            abi::DENIED
        );
        self.owner(3, [wire::PRODUCER_POINTER, token, 1, 115, 127, 1, 0, 0]);
        let event = self.next(2, epoch);
        assert_eq!(&event[2..6], &[wire::POINTER, 5, 7, 1]);
        self.owner(
            3,
            [
                wire::PRODUCER_KEY,
                token,
                2,
                input::ENTER as u64,
                input::PRESS as u64,
                0,
                0,
                0,
            ],
        );
        let key = self.next(2, epoch);
        assert_eq!(
            &key[2..5],
            &[wire::KEY, input::ENTER as u64, input::PRESS as u64]
        );
        assert_ne!(event[6], key[6]);
        assert_eq!(event[7], key[7]);
        self.owner(3, [wire::PRODUCER_RELEASE, token, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            self.query(3, [wire::PRODUCER_POINTER, token, 1, 115, 127, 0, 0, 0])[0],
            abi::DENIED
        );
        assert_eq!(self.next(2, epoch)[2], wire::RETIRED);
        assert_eq!(self.next(2, epoch)[2], wire::RETIRED);
        assert_eq!(self.next(2, epoch)[2], wire::GROUP_RETIRED);
        self.owner(
            0,
            [
                wire::INPUT_PRODUCER_GRANT,
                1,
                2,
                wire::PRODUCER_FOCUS,
                0,
                0,
                0,
                0,
            ],
        );
        let fresh = self.query(3, [wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0]);
        assert!(fresh[1] > token);
        assert_eq!(
            self.query(
                3,
                [
                    wire::PRODUCER_KEY,
                    token,
                    2,
                    input::ENTER as u64,
                    input::RELEASE as u64,
                    0,
                    0,
                    0
                ]
            )[0],
            abi::DENIED
        );
        self.owner(3, [wire::PRODUCER_POINTER, fresh[1], 1, 115, 127, 1, 0, 0]);
        assert_eq!(self.next(2, epoch)[2], wire::POINTER);
        self.owner(0, [wire::STOP, 1, 0, 0, 0, 0, 0, 0]);
        assert_eq!(self.next(2, epoch)[2], wire::RETIRED);
        assert_eq!(self.next(2, epoch)[2], wire::GROUP_RETIRED);
        cathedral_user_runtime::write(
            b"Cathedral matrix: scoped producer quotas, focus and disconnect passed\n",
        )
        .unwrap();
    }
}
