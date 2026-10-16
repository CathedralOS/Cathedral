//! Invalid admission requests must preserve authority and accounting at the syscall boundary.
use crate::{abi, syscall::call as raw};
pub(super) fn check(image: cathedral_user_runtime::task::Executable) {
    let before = cathedral_user_runtime::resources::budget().unwrap();
    let physical = cathedral_user_runtime::resources::physical_budget().unwrap();
    assert_eq!(raw(abi::PHYSICAL_ASSIGN, 0, 0, 0), abi::BAD_HANDLE);
    assert_eq!(raw(abi::PHYSICAL_BUDGET, 0, 1, 0), abi::INVALID_ARGUMENT);
    assert_eq!(raw(abi::PHYSICAL_BUDGET, 0, 0, 1), abi::INVALID_ARGUMENT);
    let slots = cathedral_user_runtime::task::available_slots().unwrap();
    for address in [0, u64::MAX, 0x8000207ff8] {
        assert_eq!(raw(abi::PHYSICAL_BUDGET, address, 0, 0), abi::BAD_ADDRESS);
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
        cathedral_user_runtime::resources::physical_budget().unwrap(),
        physical
    );
    assert_eq!(
        cathedral_user_runtime::task::available_slots().unwrap(),
        slots
    );
}
