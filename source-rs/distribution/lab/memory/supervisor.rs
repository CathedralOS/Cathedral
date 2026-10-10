//! Coordinate private allocation, peer lifetime and hardware-fault exercises.
mod faults;
mod private;
mod sharing;
use super::worker;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    memory::raw,
    task::{Child, Launch, Outcome, Port},
};
pub(super) fn run() -> u64 {
    assert_eq!(raw::allocate(1), Err(Error(abi::DENIED as i64)));
    let mut producer = Peer::spawn(0);
    let mut consumer = Peer::spawn(1);
    private::exercise(&producer);
    sharing::exercise(&mut producer, &mut consumer);
    faults::exercise(&mut producer, &mut consumer);
    // Parent exit cancels two live children, including an accepted buffer.
    let handle = producer.ok(abi::MEMORY_ALLOCATE, 4, 0);
    producer.ok(abi::MEMORY_SEAL, handle, 0);
    consumer.ok(abi::MEMORY_MAP, handle, 0);
    cathedral_user_runtime::write(b"Cathedral: page budgets leases faults and reuse passed\n")
        .unwrap();
    0
}
struct Peer {
    index: u64,
    child: Child,
    pair: (Handle, Handle),
}
impl Peer {
    fn spawn(index: u64) -> Self {
        let child = Launch::at(index).unwrap().spawn(0).unwrap();
        Self {
            index,
            child,
            pair: Port::at(index).unwrap().connect().unwrap(),
        }
    }
    fn send(&self, op: u64, first: u64, second: u64) {
        let mut bytes = [0; 24];
        for (word, output) in [op, first, second]
            .into_iter()
            .zip(bytes.chunks_exact_mut(8))
        {
            output.copy_from_slice(&word.to_le_bytes());
        }
        self.pair.0.send(&bytes).unwrap();
    }
    fn call(&self, op: u64, first: u64, second: u64) -> u64 {
        self.send(op, first, second);
        self.receive()
    }
    fn receive(&self) -> u64 {
        let mut bytes = [0; 64];
        let len = self.pair.1.receive(&mut bytes).unwrap();
        assert_eq!(len, 8);
        u64::from_le_bytes(bytes[..8].try_into().unwrap())
    }
    fn ok(&self, op: u64, first: u64, second: u64) -> u64 {
        let result = self.call(op, first, second);
        assert!((result as i64) >= 0);
        result
    }
    fn restart(&mut self) {
        self.child.cancel().unwrap();
        assert_eq!(self.child.wait().unwrap(), Outcome::Cancelled);
        *self = Self::spawn(self.index);
    }
    fn fault(&mut self, address: u64, operation: u64, error: u64) {
        self.send(worker::FAULT, address, operation);
        let Outcome::Fault {
            vector,
            error: actual,
            address: actual_address,
            ..
        } = self.child.wait().unwrap()
        else {
            panic!()
        };
        assert_eq!((vector, actual & 31, actual_address), (14, error, address));
        *self = Self::spawn(self.index);
    }
}
