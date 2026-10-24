//! Exercise owner-authorized registration through two actual IPC routers.
//! Local and inherited compact names overlap; they must remain different streams.
use super::*;
impl Fixture {
    pub(super) fn registration(&self) {
        let epoch = self.view(2)[1];
        while self.next(2, epoch)[2] != wire::NONE {}
        let create = |index, op, arg| {
            let reply = self.query(index, [op, arg, 0, 0, 0, 0, 0, 0]);
            assert_eq!(reply[0], 0);
            assert_ne!(reply[1], 0);
            reply[1]
        };
        let outer = create(0, wire::INPUT_GROUP_CREATE, 0);
        let inner = create(1, wire::INPUT_GROUP_CREATE, 0);
        let local = create(1, wire::INPUT_CHANNEL_CREATE, inner);
        self.owner(1, [wire::INPUT_POINTER, local, 15, 27, 1, 0, 0, 0]);
        let local_event = self.next(2, epoch);
        assert_eq!(local_event[2], wire::POINTER);
        let mut previous = local_event[6];
        for _ in 0..8 {
            let channel = create(0, wire::INPUT_CHANNEL_CREATE, outer);
            self.owner(0, [wire::INPUT_POINTER, channel, 115, 127, 1, 0, 0, 0]);
            let event = self.next(2, epoch);
            assert_eq!(&event[2..6], &[wire::POINTER, 5, 7, 1]);
            assert!(event[6] > previous);
            assert_ne!(event[7], local_event[7]);
            previous = event[6];
            self.owner(0, [wire::INPUT_CHANNEL_RETIRE, channel, 0, 0, 0, 0, 0, 0]);
            assert_eq!(
                self.next(2, epoch)[2..8],
                [wire::RETIRED, 0, 0, 0, event[6], event[7]]
            );
            assert_eq!(
                self.query(0, [wire::INPUT_POINTER, channel, 115, 127, 0, 0, 0, 0])[0],
                abi::DENIED
            );
        }
        // Retiring inherited sources did not cancel the locally held pointer.
        self.owner(1, [wire::INPUT_POINTER, local, 199, 159, 1, 0, 0, 0]);
        assert_eq!(
            &self.next(2, epoch)[2..7],
            &[wire::POINTER, 59, 39, 1, local_event[6]]
        );
        self.owner(1, [wire::INPUT_CHANNEL_RETIRE, local, 0, 0, 0, 0, 0, 0]);
        assert_eq!(self.next(2, epoch)[2], wire::RETIRED);
        self.owner(1, [wire::INPUT_GROUP_RELEASE, inner, 0, 0, 0, 0, 0, 0]);
        assert_eq!(self.next(2, epoch)[2], wire::GROUP_RETIRED);
        self.owner(0, [wire::INPUT_GROUP_RELEASE, outer, 0, 0, 0, 0, 0, 0]);
        assert_eq!(self.next(2, epoch)[2], wire::GROUP_RETIRED);
        // More distinct groups than the live table can hold must reuse storage,
        // while allocating new receiver-visible identities each time.
        for _ in 0..8 {
            let group = create(0, wire::INPUT_GROUP_CREATE, 0);
            let channel = create(0, wire::INPUT_CHANNEL_CREATE, group);
            self.owner(0, [wire::INPUT_POINTER, channel, 115, 127, 1, 0, 0, 0]);
            let event = self.next(2, epoch);
            assert_eq!(event[2], wire::POINTER);
            self.owner(0, [wire::INPUT_CHANNEL_RETIRE, channel, 0, 0, 0, 0, 0, 0]);
            assert_eq!(self.next(2, epoch)[2], wire::RETIRED);
            self.owner(0, [wire::INPUT_GROUP_RELEASE, group, 0, 0, 0, 0, 0, 0]);
            let ended = self.next(2, epoch);
            assert_eq!(
                (ended[2], ended[6], ended[7]),
                (wire::GROUP_RETIRED, 0, event[7])
            );
        }
        assert_eq!(self.view(2)[1], epoch); // No geometry reset to recycle input slots.
        cathedral_user_runtime::write(
            b"Cathedral matrix: input registration, retirement and namespaces passed\n",
        )
        .unwrap();
    }
}
