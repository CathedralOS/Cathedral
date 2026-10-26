//! Two identical matrix executables, a nested leaf and an outside sibling.
//! All owner requests borrow one retained shared client. The adversarial leaf
//! relay still uses its older combined parent stream; see leaf.rs.
use crate::*;
use cathedral_user_runtime::{
    connection::Client,
    task::{Child, Launch, Port},
};
pub(crate) struct Fixture {
    pub(crate) root: Child,
    control: Client,
}
impl Fixture {
    pub(crate) fn open() -> Self {
        let loans = [
            Launch::at(1).unwrap(),
            Launch::at(2).unwrap(),
            Launch::at(3).unwrap(),
        ];
        let launch = Launch::at(0).unwrap();
        // Profiles with dynamic accounts let the root run programs from its
        // realm; fund every account it may admit before it starts.
        let dynamic = cathedral_user_runtime::task::available_slots().unwrap();
        if dynamic > 0 {
            launch.assign_slots(dynamic).unwrap();
            launch
                .assign_memory((dynamic + 1) * abi::MAX_ADMISSION_FRAMES as u64)
                .unwrap();
        }
        let root = launch.spawn_tree(0, &[], &loans).unwrap();
        // Enumeration compacts after loans; another fixture may retain its own
        // provider launch. The old loan name itself must no longer grant spawning.
        assert!(loans[0].spawn(0).is_err());
        let control = Client::connect(
            Port::at(0).unwrap().connect().unwrap(),
            time::after(300).unwrap(),
        )
        .unwrap();
        let mut fixture = Self { root, control };
        fixture.owner(0, [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]);
        cathedral_user_runtime::write(b"Cathedral matrix fixture: shared owner control ready\n")
            .unwrap();
        fixture.admit_inner();
        // Leaf budget is now lent to the inner matrix, so sibling is local index 1.
        fixture.owner(0, [wire::SPAWN, 1, 1, 0, 0, 0, 0, 0]);
        fixture
    }
    pub(crate) fn admit_inner(&mut self) {
        self.owner(0, [wire::SPAWN, 0, 0, 2, 0, 0, 0, 0]);
        self.owner(1, [wire::SPAWN, 0, 0, 0, 0, 0, 0, 0]);
    }
    pub(crate) fn query(&mut self, index: usize, mut words: wire::Packet) -> wire::Packet {
        words = match index {
            0 => words,
            1 => wire::route(words, 0).unwrap(),
            2 => wire::route(wire::route(words, 0).unwrap(), 0).unwrap(),
            3 => wire::route(words, 1).unwrap(),
            _ => unreachable!(),
        };
        let (bytes, length) = self
            .control
            .call(&wire::encode(words), time::after(300).unwrap())
            .unwrap_or_else(|error| {
                panic!(
                    "matrix request {words:?}: {error:?}; root outcome {:?}",
                    self.root.wait_until(time::now().unwrap())
                )
            });
        wire::decode(&bytes[..length]).unwrap()
    }
    #[cfg(not(feature = "storage-lab"))]
    pub(crate) fn idle(&self) {
        assert!(!self.control.pending());
        // Watch lifetime only. A coalesced notification is not a new reply;
        // the next retained call owns all receive/correlation work.
        let _ = cathedral_user_runtime::event::wait(
            &self.control.interests().unwrap()[..1],
            time::after(100).unwrap(),
        );
    }

    pub(crate) fn owner(&mut self, index: usize, words: wire::Packet) {
        assert_eq!(self.query(index, words), [0; 8]);
    }
}
