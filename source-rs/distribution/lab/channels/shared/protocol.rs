//! Deterministic malformed-peer and backpressure probes using actual mapped RAM.
use super::*;
pub(super) fn run() {
    let (send, receive) = ipc::channel().unwrap();
    let mut hostile = Writer::allocate(1).unwrap();
    hostile.delegate(send).unwrap();
    let input = Reader::accept(receive).unwrap();
    let mut output = Writer::allocate(1).unwrap();
    output.unmap().unwrap();
    output.map().unwrap();
    output.store(0, 0).unwrap();
    assert_eq!(
        hostile.store(512, 0),
        Err(Error(abi::INVALID_ARGUMENT as i64))
    );
    assert_eq!(
        hostile.load(usize::MAX),
        Err(Error(abi::INVALID_ARGUMENT as i64))
    );
    assert_eq!(
        syscall::call(abi::MEMORY_FREEZE, hostile.handle(), 0, 0),
        abi::BUSY
    );
    let mut mailbox = Mailbox::new(output, input).unwrap();
    assert_eq!(
        mailbox.try_send(&[0; 257]),
        Err(Error(abi::INVALID_ARGUMENT as i64))
    );
    mailbox.try_send(b"first").unwrap();
    assert_eq!(
        mailbox.try_send(b"overwrite"),
        Err(Error(abi::WOULD_BLOCK as i64))
    );
    hostile.store(1, 2).unwrap();
    assert_eq!(
        mailbox.try_send(b"fake ack"),
        Err(Error(abi::IO_ERROR as i64))
    );
    hostile.store(1, 1).unwrap();
    mailbox.try_send(b"second").unwrap();
    let mut bytes = [0xaa; 256];
    hostile.store(0, 1).unwrap();
    hostile.store(2, u64::MAX).unwrap();
    assert_eq!(
        mailbox.try_receive(&mut bytes),
        Err(Error(abi::IO_ERROR as i64))
    );
    assert_eq!(bytes, [0xaa; 256]);
    hostile.store(2, 8).unwrap();
    hostile.store(3, 123).unwrap();
    assert_eq!(
        mailbox.try_receive(&mut bytes[..7]),
        Err(Error(abi::TOO_SMALL as i64))
    );
    assert_eq!(mailbox.try_receive(&mut bytes), Ok(Some(8)));
    assert_eq!(u64::from_le_bytes(bytes[..8].try_into().unwrap()), 123);
    assert_eq!(mailbox.try_receive(&mut bytes), Ok(None));
    hostile.store(0, u64::MAX).unwrap();
    assert_eq!(
        mailbox.try_receive(&mut bytes),
        Err(Error(abi::IO_ERROR as i64))
    );
    drop(hostile); // Local reader pins the pages after the writer is gone.
    drop(mailbox);
    send.close().unwrap();
    receive.close().unwrap();
}
