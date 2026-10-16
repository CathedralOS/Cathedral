//! Immutable mappings stay live after producer death; sealing removes its own write permission.
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    link::Link,
    memory::{Private, Shared},
};
pub fn produce(violate: bool) -> u64 {
    let mut ready = [0; 8];
    cathedral_user_runtime::ipc::Handle::bootstrap(0)
        .unwrap()
        .receive(&mut ready)
        .unwrap();
    let link = Link::at(0).unwrap();
    let (output, input) = link.connect().unwrap();
    let mut region = Private::allocate(2).unwrap();
    let address = region.bytes_mut().as_mut_ptr();
    region.bytes_mut().fill(0x6b);
    let sealed = region.seal(link).unwrap();
    output.send(&sealed.handle().to_le_bytes()).unwrap();
    let mut ack = [0; 8];
    input.receive(&mut ack).unwrap();
    if violate {
        // SAFETY: Deliberate stale mutable pointer write in a disposable process;
        // its real PTE must have lost WRITE before the immutable offer was published.
        unsafe {
            address.write_volatile(0);
        }
        return 253;
    }
    0
}
pub fn consume() -> u64 {
    let mut ready = [0; 8];
    cathedral_user_runtime::ipc::Handle::bootstrap(0)
        .unwrap()
        .receive(&mut ready)
        .unwrap();
    let link = Link::at(0).unwrap();
    let (input, output) = link.connect().unwrap();
    let mut token = [0; 8];
    assert_eq!(input.receive(&mut token).unwrap(), 8);
    let shared = Shared::accept_from(u64::from_le_bytes(token), link).unwrap();
    output.send(b"accepted").unwrap();
    assert_eq!(
        input.receive(&mut token),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    assert_eq!(shared.bytes().len(), 8192);
    assert!(shared.bytes().iter().all(|b| *b == 0x6b));
    drop(shared);
    0
}

pub fn supervise(violate: bool) -> u64 {
    use cathedral_user_runtime::task::{Launch, Outcome, Port};
    let producer = Launch::at(0).unwrap().spawn(0).unwrap();
    let consumer = Launch::at(1).unwrap().spawn(0).unwrap();
    for i in 0..2 {
        Port::at(i)
            .unwrap()
            .connect()
            .unwrap()
            .0
            .send(b"ready")
            .unwrap();
    }
    let outcome = producer.wait().unwrap();
    if violate {
        assert!(matches!(outcome, Outcome::Fault { .. }));
    } else {
        assert_eq!(outcome, Outcome::Returned(0));
    }
    assert_eq!(consumer.wait().unwrap(), Outcome::Returned(0));
    0
}
