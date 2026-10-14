//! Invalid tree requests must preserve authority and accounting at the syscall boundary.
use crate::{abi, syscall::call as raw};
pub(super) fn check(launch: cathedral_user_runtime::task::Launch) {
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
    #[repr(align(256))]
    struct Request([u64; abi::SPAWN_TREE_BYTES / 8]);
    let mut request = Request([0; abi::SPAWN_TREE_BYTES / 8]);
    let loans = abi::SPAWN_BYTES / 8;
    request.0[loans] = abi::MAX_LAUNCHES as u64 + 1;
    assert_eq!(
        raw(
            abi::TASK_SPAWN_TREE,
            launch.raw(),
            request.0.as_ptr() as u64,
            0
        ),
        abi::INVALID_ARGUMENT
    );
    request.0[loans] = 0;
    request.0[loans + 1] = launch.raw();
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
