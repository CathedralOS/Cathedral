use super::*;
pub(super) fn exercise(producer: &Peer) {
    for size in [0, 5, u64::MAX] {
        assert_eq!(
            producer.call(abi::MEMORY_ALLOCATE, size, 0),
            abi::INVALID_ARGUMENT
        );
    }
    assert_eq!(producer.call(abi::MEMORY_RELEASE, 0, 0), abi::BAD_HANDLE);
    let mut previous = 0;
    let mut previous_address = 0;
    for _ in 0..16 {
        let handle = producer.ok(abi::MEMORY_ALLOCATE, 4, 0);
        let address = producer.ok(abi::MEMORY_ADDRESS, handle, 0);
        assert_ne!(handle, previous);
        if previous != 0 {
            assert_eq!(address, previous_address);
            assert_eq!(
                producer.call(abi::MEMORY_RELEASE, previous, 0),
                abi::BAD_HANDLE
            );
        }
        assert_eq!(producer.ok(abi::MEMORY_PAGES, handle, 0), 4);
        producer.ok(worker::CHECK, handle, 0);
        producer.ok(worker::FILL, handle, 0x5a);
        producer.ok(worker::CHECK, handle, 0x5a);
        assert_eq!(producer.call(abi::MEMORY_ALLOCATE, 1, 0), abi::NO_MEMORY);
        assert_eq!(
            producer.call(abi::MEMORY_SEAL, handle, u64::MAX),
            abi::DENIED
        );
        assert_eq!(raw::address(handle), Err(Error(abi::DENIED as i64)));
        assert_eq!(raw::accept(handle), Err(Error(abi::DENIED as i64)));
        producer.ok(abi::MEMORY_RELEASE, handle, 0);
        assert_eq!(
            producer.call(abi::MEMORY_ADDRESS, handle, 0),
            abi::BAD_HANDLE
        );
        previous = handle;
        previous_address = address;
    }
    // Several leaves are removed out of insertion order; copied-pointer ledger
    // permissions must remain attached to the correct surviving virtual pages.
    let one = producer.ok(abi::MEMORY_ALLOCATE, 1, 0);
    let two = producer.ok(abi::MEMORY_ALLOCATE, 2, 0);
    producer.ok(worker::FILL, two, 17);
    producer.ok(abi::MEMORY_RELEASE, one, 0);
    producer.ok(worker::CHECK, two, 17);
    let address = producer.ok(abi::MEMORY_ADDRESS, two, 0);
    producer.send(worker::COPYOUT, address, 0);
    loop {
        match producer.pair.0.send(&[0xa5; 64]) {
            Ok(()) => break,
            Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => {
                cathedral_user_runtime::yield_now()
            }
            Err(_) => panic!(),
        }
    }
    assert_eq!(producer.receive(), 64);
    producer.ok(worker::CHECK_HEAD, two, 0xa5);
    producer.ok(abi::MEMORY_RELEASE, two, 0);
}
