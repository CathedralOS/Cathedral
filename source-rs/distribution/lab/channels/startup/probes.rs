//! Invalid startup requests must preserve authority and accounting at the syscall boundary.
use crate::{
    Handle, abi,
    syscall::{Packet, call as raw},
};
pub(super) fn check(launch: cathedral_user_runtime::task::Launch, endpoint: Handle) {
    for address in [0, u64::MAX, 0x8000203ff8] {
        assert_eq!(
            raw(abi::TASK_SPAWN_WITH, launch.raw(), address, 0),
            abi::BAD_ADDRESS
        );
        assert_eq!(raw(abi::TASK_ENVIRONMENT, address, 0, 0), abi::BAD_ADDRESS);
    }
    assert_eq!(
        raw(abi::TASK_ENVIRONMENT, b"readonly".as_ptr() as u64, 0, 0),
        abi::BAD_ADDRESS
    );
    assert_eq!(raw(abi::TASK_ENVIRONMENT, 0, 1, 0), abi::INVALID_ARGUMENT);
    assert_eq!(
        raw(abi::TASK_SPAWN_WITH, launch.raw(), 0, 1),
        abi::INVALID_ARGUMENT
    );
    let mut packet = Packet([0; abi::IPC_PACKET_BYTES]);
    packet.0[8..16].copy_from_slice(&5u64.to_le_bytes());
    assert_eq!(
        raw(
            abi::TASK_SPAWN_WITH,
            launch.raw(),
            packet.0.as_ptr() as u64,
            0
        ),
        abi::INVALID_ARGUMENT
    );
    packet.0[8..16].copy_from_slice(&0u64.to_le_bytes());
    packet.0[16..24].copy_from_slice(&endpoint.raw().to_le_bytes());
    assert_eq!(
        raw(
            abi::TASK_SPAWN_WITH,
            launch.raw(),
            packet.0.as_ptr() as u64,
            0
        ),
        abi::INVALID_ARGUMENT
    );
    packet.0[8..16].copy_from_slice(&1u64.to_le_bytes());
    assert_eq!(
        raw(abi::TASK_SPAWN_WITH, 0, packet.0.as_ptr() as u64, 0),
        abi::BAD_HANDLE
    );
}
