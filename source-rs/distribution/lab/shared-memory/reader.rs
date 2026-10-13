use super::*;
pub fn run(mode: u64) -> u64 {
    if mode == 1 {
        return 0;
    }
    let input = introduced(Handle::bootstrap(0).unwrap());
    let (_, object) = input.receive_memory(&mut [0; 64]).unwrap();
    let mut object = object.unwrap();
    // B leaves its send right open until exit. A and B are dead before this map.
    assert!(
        matches!(input.receive(&mut [0; 64]), Err(Error(code)) if code == abi::PEER_CLOSED as i64)
    );
    for _ in 0..4 {
        for (i, &byte) in object.map().unwrap().iter().enumerate() {
            assert_eq!(byte, (i % 251) as u8);
        }
        object.unmap().unwrap();
    }
    if mode == 2 {
        let address = object.map().unwrap().as_ptr() as *mut u8;
        // SAFETY: Deliberate user protection fault; kernel lab requires a write permission fault.
        unsafe {
            address.write_volatile(99);
        }
        panic!("sealed reader became writable");
    }
    drop(object);
    input.close().unwrap();
    0
}
