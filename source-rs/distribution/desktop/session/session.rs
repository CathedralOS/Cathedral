// The F5 desktop. Admit providers and the funded root matrix, show the chooser
// the folder of users, open the workspace slot to what it launches, start
// physical input, then furnish each user matrix that appears and supervise
// storage. Admission and standing policy live in shell/; the chooser and the
// apps are programs in the store. The lab module composes the same admission,
// apps and budgets into the probe desktop from the boot catalogue.
use super::*;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::{ipc, task::Launch};
pub(crate) const KEYS: u64 = if cfg!(feature = "notes") {
    wire::VALID_KEY_MASK
} else {
    wire::KEY_MASK
};
pub(crate) const BACKGROUND: u64 = 0x101827;
pub(crate) fn owner(pair: Pair, words: wire::Packet) {
    assert_eq!(call(pair, words), [0; 8], "owner request {words:?}");
}
#[cfg(feature = "shell")]
pub(crate) fn run() -> u64 {
    let mut session = admission::start();
    let matrix = session.matrix;
    owner(
        matrix,
        [
            wire::BIND,
            session.extent[0],
            session.extent[1],
            KEYS,
            0,
            0,
            0,
            0,
        ],
    );
    shell::chooser(matrix);
    shell::workspace(matrix);
    session.start_input();
    cathedral_user_runtime::write(b"Cathedral desktop: ready; choose a user\n").unwrap();
    loop {
        // The chooser launches a user matrix into the workspace slot; the
        // desktop learns of it by its realm and gives it its chrome. The app
        // the chrome launches is tracked only so storage replacement can
        // restore its realm.
        let user = shell::realm_of(matrix, [wire::STORAGE_BINDING, 1, 0, 0, 0, 0, 0, 0]);
        if user != session.user {
            session.user = user;
            session.app = 0;
            if user != 0 {
                shell::furnish(matrix, user);
                cathedral_user_runtime::write(b"Cathedral desktop: user matrix furnished\n")
                    .unwrap();
            } else {
                cathedral_user_runtime::write(b"Cathedral desktop: user matrix closed\n").unwrap();
            }
        }
        if user != 0 {
            session.app = shell::realm_of(
                matrix,
                wire::route([wire::STORAGE_BINDING, 1, 0, 0, 0, 0, 0, 0], 1).unwrap(),
            );
        }
        persistence::supervise(&mut session);
        // Polling is only how the owner notices; rendering and input wake the
        // programs independently through their matrix connections.
        match matrix
            .1
            .receive_until(&mut [0; 64], time::after(10).unwrap())
        {
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            result => panic!("desktop idle {result:?}"),
        }
    }
}
