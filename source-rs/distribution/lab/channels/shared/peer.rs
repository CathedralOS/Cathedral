//! Each peer owns its reply page, receives only RO access to requests.
use super::*;
pub(crate) fn worker(mode: u64) -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    if mode == 45 {
        let [wait, _, _] = N::create().unwrap();
        output.send(b"parked").unwrap();
        let _ = wait.wait(None);
        panic!("abandoned escrow peer resumed");
    }
    if mode == 41 {
        let mut writer = Writer::allocate(1).unwrap();
        writer.store(0, 0x12345678).unwrap();
        writer.delegate(output).unwrap();
        receive(input);
        return 0;
    }
    if mode == 42 {
        assert_eq!(resources::budget().unwrap().used.private_pages, 1);
        output.send(b"charged").unwrap();
        receive(input);
        assert_eq!(resources::budget().unwrap().used.private_pages, 0);
        return 0;
    }
    if mode == 43 || mode == 44 {
        let reader = Reader::accept(input).unwrap();
        let address = raw::address(reader.handle()).unwrap();
        output.send(&address.to_le_bytes()).unwrap();
        if mode == 43 {
            // SAFETY: Deliberate isolated user fault; parent requires write protection.
            unsafe { (address as *mut u64).write_volatile(0) };
        } else {
            // SAFETY: Deliberate isolated NX fault; never allowed to execute this page.
            let call: extern "C" fn() = unsafe { core::mem::transmute(address as usize) };
            call();
        }
        panic!("shared view escaped page protections");
    }
    let (bytes, _) = receive(input);
    let foreign = u64::from_le_bytes(bytes[..8].try_into().unwrap());
    assert_eq!(raw::address(foreign), Err(Error(abi::BAD_HANDLE as i64)));
    output.send(b"foreign denied").unwrap();
    ipc::wait_many(&[input], time::after(300).unwrap()).unwrap();
    assert_eq!(
        syscall::call(abi::IPC_RECEIVE_SHARED, input.raw(), 1, 0),
        abi::BAD_ADDRESS
    );
    assert_eq!(
        input.receive_memory(&mut [0; 64]).err(),
        Some(Error(abi::TOO_SMALL as i64))
    );
    let reader = Reader::accept(input).unwrap();
    assert_eq!(
        syscall::call(abi::MEMORY_MAP_OBJECT, reader.handle(), 0, 0),
        abi::DENIED
    );
    assert_eq!(
        syscall::call(abi::TASK_IMAGE_FROM_OBJECT, reader.handle(), 0, 0),
        abi::DENIED
    );
    let writer = Writer::allocate(1).unwrap();
    writer.delegate(output).unwrap();
    let (_, wait) = N::accept(input, &mut [0; 64]).unwrap();
    output.send(b"wait accepted").unwrap();
    let (_, signal) = N::accept(input, &mut [0; 64]).unwrap();
    let wait = wait.unwrap();
    let signal = signal.unwrap();
    let mut channel = Mailbox::new(writer, reader).unwrap();
    output.send(b"ready").unwrap();
    for round in 0..ROUNDS {
        let mut bytes = [0; 256];
        let length = loop {
            if let Some(n) = channel.try_receive(&mut bytes).unwrap() {
                break n;
            }
            assert_eq!(
                wait.wait(Some(time::after(300).unwrap())),
                Ok(Wake::Signaled)
            );
        };
        let (expected, n) = payload(round);
        assert_eq!(length, n);
        assert_eq!(&bytes[..n], &expected[..n]);
        channel.try_send(&bytes[..n]).unwrap();
        signal.signal(false).unwrap();
    }
    0
}
