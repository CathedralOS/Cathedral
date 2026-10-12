//! Invalid resources requests must preserve authority and accounting at the syscall boundary.
use crate::{abi, syscall::call as raw};
pub(super) fn check(launch: cathedral_user_runtime::task::Launch) {
    let before = cathedral_user_runtime::resources::budget().unwrap();
    for address in [0, u64::MAX] {
        assert_eq!(raw(abi::RESOURCE_BUDGET, address, 0, 0), abi::BAD_ADDRESS);
        assert_eq!(raw(abi::PHYSICAL_BUDGET, address, 0, 0), abi::BAD_ADDRESS);
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
