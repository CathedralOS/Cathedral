// Two identical matrix executables, a nested leaf and an outside sibling.
use super::*;
use cathedral_contracts::input;
use cathedral_user_runtime::task::{Child, Launch, Port};
struct Fixture {
    root: Child,
    control: Pair,
}
impl Fixture {
    fn open() -> Self {
        let loans = [
            Launch::at(1).unwrap(),
            Launch::at(2).unwrap(),
            Launch::at(3).unwrap(),
        ];
        let root = Launch::at(0).unwrap().spawn_tree(0, &[], &loans).unwrap();
        assert!(Launch::at(1).is_err());
        let control = Port::at(0).unwrap().connect().unwrap();
        let fixture = Self { root, control };
        fixture.admit_inner();
        // Leaf budget is now lent to the inner matrix, so sibling is local index 1.
        fixture.owner(0, [wire::SPAWN, 1, 1, 0, 0, 0, 0, 0]);
        fixture
    }
    fn admit_inner(&self) {
        self.owner(0, [wire::SPAWN, 0, 0, 2, 0, 0, 0, 0]);
        self.owner(1, [wire::SPAWN, 0, 0, 0, 0, 0, 0, 0]);
    }
    fn query(&self, index: usize, mut words: wire::Packet) -> wire::Packet {
        words = match index {
            0 => words,
            1 => wire::route(words, 0).unwrap(),
            2 => wire::route(wire::route(words, 0).unwrap(), 0).unwrap(),
            3 => wire::route(words, 1).unwrap(),
            _ => unreachable!(),
        };
        call(self.control, words)
    }
    fn owner(&self, index: usize, words: wire::Packet) {
        assert_eq!(self.query(index, words), [0; 8]);
    }
    fn layout(&self) {
        self.owner(0, [wire::BIND, 800, 600, wire::KEY_MASK, 0, 0, 0, 0]);
        self.owner(0, [wire::PLACE, 0, 100, 100, 200, 160, wire::KEY_MASK, 0]);
        self.owner(0, [wire::PLACE, 1, 500, 100, 100, 80, wire::KEY_MASK, 0]);
        self.owner(
            1,
            [wire::PLACE, 0, 10, 20, 60, 40, 1 << (input::ENTER - 1), 0],
        );
    }
    fn view(&self, leaf: usize) -> wire::Packet {
        self.query(leaf, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])
    }
    fn next(&self, leaf: usize, epoch: u64) -> wire::Packet {
        self.query(leaf, [wire::NEXT, epoch, 0, 0, 0, 0, 0, 0])
    }
    fn pointer(&self, x: u64, y: u64, buttons: u64) {
        self.owner(0, [wire::POINTER_SOURCE, x, y, buttons, 0, 0, 0, 0]);
    }
    fn key(&self, key: u8, state: u8) {
        self.owner(
            0,
            [wire::KEY_SOURCE, key as u64, state as u64, 0, 0, 0, 0, 0],
        );
    }
}
pub fn run() -> u64 {
    let mut fixture = Fixture::open();
    fixture.layout();
    fixture.confinement();
    fixture.input();
    fixture.recovery();
    cathedral_user_runtime::write(b"Cathedral matrix: two identical worlds, local-only views, attenuated input and recovery passed\n").unwrap();
    loop {
        assert_eq!(
            fixture.root.wait_until(time::now().unwrap()),
            Err(Error(abi::TIMED_OUT as i64))
        );
        let _ = fixture
            .control
            .1
            .receive_until(&mut [0; 64], time::after(100).unwrap());
    }
}
