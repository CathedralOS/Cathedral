// The F5 desktop. Admit providers and the funded root matrix, open the
// worlds, lay out the workspace and the program menu, start physical input,
// then serve menu choices and supervise storage. What each choice does lives
// in shell/, the worlds in worlds.rs; the apps live in app/ and
// ../applications/notes. The lab module composes the same admission, apps
// and budgets into the probe desktop.
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
    shell::workspace(matrix);
    shell::menu(matrix);
    session.start_input();
    cathedral_user_runtime::write(b"Cathedral desktop: ready; choose a program\n").unwrap();
    let (mut sequence, mut active, mut blocked) = (0, 0, 0);
    loop {
        let (world, worlds) = session.worlds.position();
        let reply = call(
            matrix,
            wire::route(
                [shell::MENU_STATUS, active, blocked, world, worlds, 0, 0, 0],
                1,
            )
            .unwrap(),
        );
        assert_eq!(reply[0], 0);
        if reply[1] != sequence {
            sequence = reply[1];
            blocked = shell::select(&mut session, &mut active, reply[2]);
        }
        persistence::supervise(&mut session);
        // An owner poll is only the menu's notification path; rendering and input
        // still wake the apps independently through their matrix connections.
        match matrix
            .1
            .receive_until(&mut [0; 64], time::after(10).unwrap())
        {
            Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
            result => panic!("desktop idle {result:?}"),
        }
    }
}
