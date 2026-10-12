//! Raw invalid pointers exercise atomicity at the real syscall boundary.
use super::*;
fn raw(number: u64, a: u64, b: u64, c: u64) -> u64 {
    let result;
    // SAFETY: Integers only, deliberately invalid pointers reach checked kernel copies.
    // No Rust references are made from them; the trap preserves non-result registers.
    unsafe {
        core::arch::asm!("int 0x80", inlateout("rax") number => result, in("rdi") a, in("rsi") b, in("rdx") c, options(nostack, preserves_flags));
    }
    result
}
pub fn resources(launch: cathedral_user_runtime::task::Launch) {
    let before = cathedral_user_runtime::resources::budget().unwrap();
    for address in [0, u64::MAX] {
        assert_eq!(raw(abi::RESOURCE_BUDGET, address, 0, 0), abi::BAD_ADDRESS);
        assert_eq!(
            raw(abi::RESOURCE_ASSIGN, launch.raw(), address, 0),
            abi::BAD_ADDRESS
        );
    }
    assert_eq!(raw(abi::RESOURCE_ASSIGN, 0, 0, 0), abi::BAD_HANDLE);
    assert_eq!(
        raw(abi::RESOURCE_ASSIGN, launch.raw(), 0, 1),
        abi::INVALID_ARGUMENT
    );
    assert_eq!(cathedral_user_runtime::resources::budget().unwrap(), before);
}
pub fn admission(image: cathedral_user_runtime::task::Executable) {
    let before = cathedral_user_runtime::resources::budget().unwrap();
    let slots = cathedral_user_runtime::task::available_slots().unwrap();
    for address in [0, u64::MAX] {
        assert_eq!(
            raw(abi::TASK_RESERVE, image.raw(), address, 0),
            abi::BAD_ADDRESS
        );
    }
    assert_eq!(
        raw(abi::TASK_RESERVE, image.raw(), 0, 1),
        abi::INVALID_ARGUMENT
    );
    assert_eq!(cathedral_user_runtime::resources::budget().unwrap(), before);
    assert_eq!(
        cathedral_user_runtime::task::available_slots().unwrap(),
        slots
    );
}

pub fn tree(launch: cathedral_user_runtime::task::Launch) {
    for address in [0, u64::MAX, 0x8000203ff8] {
        assert_eq!(
            raw(abi::TASK_SPAWN_TREE, launch.raw(), address, 0),
            abi::BAD_ADDRESS
        );
    }
    assert_eq!(
        raw(abi::TASK_SPAWN_TREE, launch.raw(), 0, 1),
        abi::INVALID_ARGUMENT
    );
    #[repr(align(128))]
    struct Request([u64; abi::SPAWN_TREE_BYTES / 8]);
    let mut request = Request([0; abi::SPAWN_TREE_BYTES / 8]);
    request.0[6] = abi::MAX_LAUNCHES as u64 + 1;
    assert_eq!(
        raw(
            abi::TASK_SPAWN_TREE,
            launch.raw(),
            request.0.as_ptr() as u64,
            0
        ),
        abi::INVALID_ARGUMENT
    );
    request.0[6] = 0;
    request.0[7] = launch.raw();
    assert_eq!(
        raw(
            abi::TASK_SPAWN_TREE,
            launch.raw(),
            request.0.as_ptr() as u64,
            0
        ),
        abi::INVALID_ARGUMENT
    );
}
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
#[repr(align(128))]
struct Packet([u8; abi::IPC_PACKET_BYTES]);

pub fn startup(launch: cathedral_user_runtime::task::Launch, endpoint: Handle) {
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
