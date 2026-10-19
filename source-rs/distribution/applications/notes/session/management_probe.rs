//! Test-only unprivileged calls and a non-yielding app for desktop recovery.
use super::*;
pub fn denied(connections: &cathedral_user_runtime::matrix::OptionalConnections) {
    for pair in [Some(connections.interaction), connections.storage]
        .into_iter()
        .flatten()
    {
        for words in [
            [wire::MANAGED_TASK, 0, 0, 0, 0, 0, 0, 0],
            [wire::TERMINATE_TASK, 1, 0, 0, 0, 0, 0, 0],
            [wire::MANAGEMENT_GRANT, 1, 1, 0, 0, 0, 0, 0],
        ] {
            assert_eq!(call(pair, words), wire::error(abi::DENIED));
        }
    }
    cathedral_user_runtime::write(b"Cathedral Notes: ungranted management denied\n").unwrap();
}
pub fn event(event: wire::Packet) {
    if event[2] == wire::KEY && event[3] == input::F8 as u64 && event[4] == input::PRESS as u64 {
        cathedral_user_runtime::write(b"Cathedral Notes: deliberate management hang\n").unwrap();
        loop {
            core::hint::spin_loop();
        }
    }
}
