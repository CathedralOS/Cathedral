//! Invalid tree requests must preserve authority and accounting at the syscall boundary.
use crate::{abi, syscall::call as raw};
pub(super) fn check(launch: cathedral_user_runtime::task::Launch) {
    for address in [0, u64::MAX, 0x8000207ff8] {
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
    request.0[loans] = abi::MAX_SPAWN_LOANS as u64 + 1;
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

/// The saved-program ABI ends at byte 176. Poison adjacent memory so an arena-
/// sized copy would refuse this otherwise valid recursive launch.
pub(super) fn legacy_spawn(
    launch: cathedral_user_runtime::task::Launch,
    endpoints: [cathedral_user_runtime::ipc::Handle; 3],
    loans: [cathedral_user_runtime::task::Launch; 2],
) -> cathedral_user_runtime::task::Child {
    #[repr(align(512))]
    struct Record([u64; 64]);
    let mut record = Record([u64::MAX; 64]);
    record.0[..22].fill(0); // Original wire format, independent of arena constants.
    record.0[0] = 2;
    record.0[1] = 3;
    for (word, handle) in record.0[2..5].iter_mut().zip(endpoints) {
        *word = handle.raw();
    }
    record.0[7] = 2;
    record.0[8] = loans[0].raw();
    record.0[9] = loans[1].raw();
    let child = raw(
        abi::TASK_SPAWN_TREE,
        launch.raw(),
        record.0.as_ptr() as u64,
        0,
    );
    assert!((child as i64) >= 0, "legacy tree request refused");
    cathedral_user_runtime::task::Child::from_raw(child)
}
