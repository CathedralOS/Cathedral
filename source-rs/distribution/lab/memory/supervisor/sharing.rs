use super::*;
fn offer(producer: &Peer) -> u64 {
    let handle = producer.ok(abi::MEMORY_ALLOCATE, 4, 0);
    producer.ok(worker::FILL, handle, 42);
    producer.ok(abi::MEMORY_SEAL, handle, 0);
    assert_eq!(producer.call(abi::MEMORY_RELEASE, handle, 0), abi::BUSY);
    handle
}
pub(super) fn exercise(producer: &mut Peer, consumer: &mut Peer) {
    let handle = offer(producer);
    assert_eq!(
        consumer.call(abi::MEMORY_ADDRESS, handle, 0),
        abi::BAD_HANDLE
    );
    let reader = consumer.ok(abi::MEMORY_MAP, handle, 0);
    let address = consumer.ok(abi::MEMORY_ADDRESS, reader, 0);
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
    // any authority over that replacement or consume its fresh private budget.
    for _ in 0..8 {
        let old = offer(producer);
        let old_reader = consumer.ok(abi::MEMORY_MAP, old, 0);
        producer.restart();
        consumer.ok(worker::CHECK, old_reader, 42);
        assert_eq!(producer.call(abi::MEMORY_ADDRESS, old, 0), abi::BAD_HANDLE);
        let new = offer(producer);
        assert_eq!(consumer.call(abi::MEMORY_MAP, new, 0), abi::NO_MEMORY);
        consumer.ok(abi::MEMORY_RELEASE, old_reader, 0);
        assert_eq!(raw::address(old), Err(Error(abi::BAD_HANDLE as i64)));
        let new_reader = consumer.ok(abi::MEMORY_MAP, new, 0);
        consumer.ok(worker::CHECK, new_reader, 42);
        consumer.restart();
        assert_eq!(consumer.call(abi::MEMORY_MAP, new, 0), abi::BAD_HANDLE);
        producer.ok(abi::MEMORY_RELEASE, new, 0);
    }
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
