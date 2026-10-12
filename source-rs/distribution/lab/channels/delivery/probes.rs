//! Channel creation and endpoint moves validate copies before changing ownership.
use crate::{
    Error, Handle, abi, ipc,
    syscall::{Packet, call as raw},
};

pub fn creation() {
    let before = ipc::budget().unwrap();
    for address in [0, u64::MAX, 0x8000203ff8] {
        assert_eq!(raw(abi::IPC_CREATE, address, 0, 0), abi::BAD_ADDRESS);
        assert_eq!(raw(abi::IPC_BUDGET, address, 0, 0), abi::BAD_ADDRESS);
    }
    assert_eq!(raw(abi::IPC_CREATE, 0, 1, 0), abi::INVALID_ARGUMENT);
    assert_eq!(raw(abi::IPC_BUDGET, 0, 1, 0), abi::INVALID_ARGUMENT);
    assert_eq!(raw(abi::IPC_BUDGET, 0, 0, 1), abi::INVALID_ARGUMENT);
    assert_eq!(ipc::budget().unwrap(), before);
}

pub fn send(destination: Handle, endpoint: Handle) {
    for address in [0, u64::MAX, 0x8000203ff8] {
        assert_eq!(
            raw(abi::IPC_SEND_CAP, destination.raw(), address, 0),
            abi::BAD_ADDRESS
        );
    }
    let mut packet = Packet([0; abi::IPC_PACKET_BYTES]);
    packet.0[..8].copy_from_slice(&65u64.to_le_bytes());
    packet.0[8..16].copy_from_slice(&endpoint.raw().to_le_bytes());
    assert_eq!(
        raw(
            abi::IPC_SEND_CAP,
            destination.raw(),
            packet.0.as_ptr() as u64,
            0
        ),
        abi::INVALID_ARGUMENT
    );
    packet.0[..8].copy_from_slice(&0u64.to_le_bytes());
    packet.0[8..16].copy_from_slice(&destination.raw().to_le_bytes());
    assert_eq!(
        raw(
            abi::IPC_SEND_CAP,
            destination.raw(),
            packet.0.as_ptr() as u64,
            0
        ),
        abi::DENIED
    );
}

pub fn receive(input: Handle) {
    let readonly = b"read-only";
    for address in [0, u64::MAX, 0x8000203ff8, readonly.as_ptr() as u64] {
        assert_eq!(
            raw(abi::IPC_RECEIVE_CAP, input.raw(), address, 0),
            abi::BAD_ADDRESS
        );
    }
    assert_eq!(
        raw(abi::IPC_RECEIVE_CAP, input.raw(), 0, 1),
        abi::INVALID_ARGUMENT
    );
    assert_eq!(
        input.receive(&mut [0; 64]),
        Err(Error(abi::TOO_SMALL as i64))
    );
}
