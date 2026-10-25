//! An underfunded inner matrix cannot borrow an outside app's frame reservation.
use super::*;
pub(super) fn exercise(matrix: Pair, pressure: bool) {
    pixel(matrix, 510, 190, app::RIGHT);
    owner(matrix, [wire::RUN_MEMORY, 0, 256, 0, 0, 0, 0, 0]);
    launch_inner(matrix, pressure);
    let nested = |words| wire::route(words, 0).unwrap();
    for _ in 0..2 {
        owner(matrix, nested([wire::RUN_MEMORY, 0, 1, 0, 0, 0, 0, 0]));
        assert_eq!(
            call(matrix, nested([wire::RUN, 0, 1, 0, APP_CHANNELS, 0, 0, 0])),
            wire::error(abi::NO_MEMORY)
        );
    }
    // The inner service's own image, stack and tables already occupy part of 256.
    // Giving its child all 256 would withdraw live funding, so admission is denied.
    assert_eq!(
        call(matrix, nested([wire::RUN, 0, 1, 0, APP_CHANNELS, 0, 0, 0])),
        wire::error(abi::BUSY)
    );
    // Exercise real app IPC and new scanout while the other branch cannot admit.
    let sibling = |op| wire::route([op, 0, 0, 0, 0, 0, 0, 0], 1).unwrap();
    assert_eq!(call(matrix, sibling(1))[0], 0);
    pixel(matrix, 540, 225, app::RIGHT);
    assert_eq!(call(matrix, sibling(2))[0], 0);
    pixel(matrix, 540, 225, 0xe5e7eb);
    owner(matrix, [wire::STOP, 0, 0, 0, 0, 0, 0, 0]);
    cathedral_user_runtime::write(b"Cathedral matrix desktop: physical admission exhaustion preserved sibling IPC and rendering\n").unwrap();
}
