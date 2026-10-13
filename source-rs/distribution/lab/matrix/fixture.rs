// Two identical matrix executables, a nested leaf and an outside sibling.
use crate::*;
use cathedral_user_runtime::task::{Child, Launch, Port};
pub(crate) struct Fixture {
    pub(crate) root: Child,
    pub(crate) control: Pair,
}
impl Fixture {
    pub(crate) fn open() -> Self {
        let loans = [
            Launch::at(1).unwrap(),
            Launch::at(2).unwrap(),
            Launch::at(3).unwrap(),
        ];
        let root = Launch::at(0).unwrap().spawn_tree(0, &[], &loans).unwrap();
        // Enumeration compacts after loans; another fixture may retain its own
        // provider launch. The old loan name itself must no longer grant spawning.
        assert!(loans[0].spawn(0).is_err());
        let control = Port::at(0).unwrap().connect().unwrap();
        let fixture = Self { root, control };
        fixture.admit_inner();
        // Leaf budget is now lent to the inner matrix, so sibling is local index 1.
        fixture.owner(0, [wire::SPAWN, 1, 1, 0, 0, 0, 0, 0]);
        fixture
    }
    pub(crate) fn admit_inner(&self) {
        self.owner(0, [wire::SPAWN, 0, 0, 2, 0, 0, 0, 0]);
        self.owner(1, [wire::SPAWN, 0, 0, 0, 0, 0, 0, 0]);
    }
    pub(crate) fn query(&self, index: usize, mut words: wire::Packet) -> wire::Packet {
        words = match index {
            0 => words,
            1 => wire::route(words, 0).unwrap(),
            2 => wire::route(wire::route(words, 0).unwrap(), 0).unwrap(),
            3 => wire::route(words, 1).unwrap(),
            _ => unreachable!(),
        };
        self.control
            .0
            .send(&wire::encode(words))
            .unwrap_or_else(|error| {
                panic!(
                    "matrix send {words:?}: {error:?}; root outcome {:?}",
                    self.root.wait_until(time::now().unwrap())
                )
            });
        let mut bytes = [0; 64];
        let length = self
            .control
            .1
            .receive_until(&mut bytes, time::after(300).unwrap())
            .unwrap_or_else(|error| {
                panic!(
                    "matrix request {words:?}: {error:?}; root outcome {:?}",
                    self.root.wait_until(time::now().unwrap())
                )
            });
        wire::decode(&bytes[..length]).unwrap()
    }
    pub(crate) fn owner(&self, index: usize, words: wire::Packet) {
        assert_eq!(self.query(index, words), [0; 8]);
    }
}
