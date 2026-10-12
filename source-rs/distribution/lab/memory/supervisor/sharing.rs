use super::*;
fn offer(producer: &Peer) -> u64 {
    let handle = producer.ok(abi::MEMORY_ALLOCATE, 4, 0);
    producer.ok(worker::FILL, handle, 42);
    producer.ok(abi::MEMORY_SEAL, handle, 0);
    assert_eq!(producer.call(abi::MEMORY_RELEASE, handle, 0), abi::BUSY);
    handle
}
pub(super) fn exercise(producer: &mut Peer, consumer: &mut Peer) {
    let baseline = producer.ok(abi::PHYSICAL_BUDGET, 1, 0);
    let reader_baseline = consumer.ok(abi::PHYSICAL_BUDGET, 1, 0);
    let handle = offer(producer);
    assert_eq!(producer.ok(abi::PHYSICAL_BUDGET, 1, 0), baseline + 4);
    assert_eq!(
        consumer.call(abi::MEMORY_ADDRESS, handle, 0),
        abi::BAD_HANDLE
    );
    let reader = consumer.ok(abi::MEMORY_MAP, handle, 0);
    let address = consumer.ok(abi::MEMORY_ADDRESS, reader, 0);
    assert_eq!(consumer.ok(abi::PHYSICAL_BUDGET, 1, 0), reader_baseline);
    assert_eq!(consumer.call(abi::MEMORY_MAP, handle, 0), abi::BUSY);
    assert_eq!(consumer.call(abi::MEMORY_ALLOCATE, 1, 0), abi::DENIED);
    consumer.ok(worker::CHECK, reader, 42);
    assert_eq!(consumer.call(worker::COPYOUT, address, 0), abi::BAD_ADDRESS);
    assert_eq!(producer.call(worker::COPYOUT, address, 0), abi::BAD_ADDRESS);
    consumer.ok(abi::MEMORY_RELEASE, reader, 0);
    assert_eq!(consumer.call(abi::MEMORY_MAP, handle, 0), abi::BAD_HANDLE);
    producer.ok(abi::MEMORY_RELEASE, handle, 0);
    assert_eq!(consumer.call(abi::MEMORY_MAP, handle, 0), abi::BAD_HANDLE);

    // An accepted read lease survives producer replacement, but does not confer
    // any authority over that replacement. Backing still charges the persistent
    // producer account; restart cannot mint fresh capacity while a reader pins it.
    for _ in 0..8 {
        let old = offer(producer);
        let old_reader = consumer.ok(abi::MEMORY_MAP, old, 0);
        producer.restart();
        assert_eq!(producer.ok(abi::PHYSICAL_BUDGET, 1, 0), baseline + 4);
        consumer.ok(worker::CHECK, old_reader, 42);
        assert_eq!(producer.call(abi::MEMORY_ADDRESS, old, 0), abi::BAD_HANDLE);
        assert_eq!(producer.call(abi::MEMORY_ALLOCATE, 1, 0), abi::NO_MEMORY);
        consumer.ok(abi::MEMORY_RELEASE, old_reader, 0);
        assert_eq!(producer.ok(abi::PHYSICAL_BUDGET, 1, 0), baseline);
        assert_eq!(raw::address(old), Err(Error(abi::BAD_HANDLE as i64)));
        let new = offer(producer);
        let new_reader = consumer.ok(abi::MEMORY_MAP, new, 0);
        consumer.ok(worker::CHECK, new_reader, 42);
        consumer.restart();
        assert_eq!(consumer.call(abi::MEMORY_MAP, new, 0), abi::BAD_HANDLE);
        producer.ok(abi::MEMORY_RELEASE, new, 0);
    }
    // A surviving reader also pins physical funding while the producer is idle.
    let reserved = producer.ok(abi::PHYSICAL_BUDGET, 0, 0);
    let held = offer(producer);
    let held_reader = consumer.ok(abi::MEMORY_MAP, held, 0);
    producer.child.cancel().unwrap();
    assert_eq!(producer.child.wait().unwrap(), Outcome::Cancelled);
    let launch = Launch::at(producer.index).unwrap();
    assert_eq!(launch.assign_memory(0), Err(Error(abi::BUSY as i64)));
    consumer.ok(worker::CHECK, held_reader, 42);
    consumer.ok(abi::MEMORY_RELEASE, held_reader, 0);
    launch.assign_memory(0).unwrap();
    assert_eq!(launch.spawn(0).unwrap_err(), Error(abi::NO_MEMORY as i64));
    launch.assign_memory(reserved).unwrap();
    *producer = Peer::spawn(producer.index);
    assert_eq!(producer.ok(abi::PHYSICAL_BUDGET, 1, 0), baseline);
    let abandoned = offer(producer);
    producer.restart();
    assert_eq!(
        consumer.call(abi::MEMORY_MAP, abandoned, 0),
        abi::BAD_HANDLE
    );
    let cancelled = offer(producer);
    consumer.restart();
    assert_eq!(
        consumer.call(abi::MEMORY_MAP, cancelled, 0),
        abi::BAD_HANDLE
    );
    producer.ok(abi::MEMORY_RELEASE, cancelled, 0);
}
