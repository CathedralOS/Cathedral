use super::*;
pub(super) fn exercise(producer: &mut Peer, consumer: &mut Peer) {
    // Each case resumes a fresh incarnation and therefore also exercises full
    // fault-time reclamation, stale offer retirement and reserved-table reuse.
    for (operation, offset, error) in [(2, 0, 21), (1, 4096, 6), (0, 4096, 4)] {
        let handle = producer.ok(abi::MEMORY_ALLOCATE, 1, 0);
        let address = producer.ok(abi::MEMORY_ADDRESS, handle, 0);
        producer.fault(address + offset, operation, error);
        assert_eq!(raw::address(handle), Err(Error(abi::BAD_HANDLE as i64)));
    }
    let handle = producer.ok(abi::MEMORY_ALLOCATE, 1, 0);
    let address = producer.ok(abi::MEMORY_ADDRESS, handle, 0);
    producer.ok(abi::MEMORY_RELEASE, handle, 0);
    producer.fault(address, 0, 4);

    for operation in [1, 2] {
        let handle = producer.ok(abi::MEMORY_ALLOCATE, 1, 0);
        producer.ok(abi::MEMORY_SEAL, handle, 0);
        let address = consumer.ok(abi::MEMORY_MAP, handle, 0);
        consumer.fault(address, operation, if operation == 1 { 7 } else { 21 });
        producer.ok(abi::MEMORY_RELEASE, handle, 0);
    }
    let handle = producer.ok(abi::MEMORY_ALLOCATE, 1, 0);
    producer.ok(worker::FILL, handle, 91);
    producer.ok(abi::MEMORY_SEAL, handle, 0);
    let address = consumer.ok(abi::MEMORY_MAP, handle, 0);
    producer.fault(address, 1, 7);
    consumer.ok(worker::CHECK, handle, 91);
    consumer.ok(abi::MEMORY_RELEASE, handle, 0);
    consumer.fault(address, 0, 4);
}
