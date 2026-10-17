// The distribution owns installed choices and admission. The launcher is an
// ordinary isolated child: it reports a choice, never receives launch authority.
use super::*;
const NOTES: u64 = 1;
const COLORS: u64 = 2;
const CLOSE: u64 = 3;
const MENU_STATUS: u64 = 200;
/// Why the last choice was refused, reported to the launcher with the active program.
const UNSAVED: u64 = 1;
const BUSY: u64 = 2;

pub(super) fn grant_notes(matrix: Pair) {
    owner(matrix, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    owner(
        matrix,
        wire::route([wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
    );
}

fn workspace(matrix: Pair) {
    owner(matrix, [wire::RUN_PAGES, 0, 40, 40, 0, 0, 0, 0]);
    owner(matrix, [wire::RUN, 0, 0, 0, 10, 1, 7, 0]);
    owner(matrix, [wire::SURFACE_LIMIT, 0, 8, 0, 0, 0, 0, 0]);
    owner(matrix, [wire::PLACE, 0, 310, 140, 430, 240, KEYS, 0]);
}

/// Returns why the choice was refused, or zero when it was carried out.
fn select(session: &mut admission::Session, active: &mut u64, choice: u64) -> u64 {
    if !(NOTES..=CLOSE).contains(&choice) || choice == *active {
        return 0;
    }
    let matrix = session.matrix;
    if *active == NOTES {
        let health = call(
            matrix,
            wire::route(
                wire::route([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
                0,
            )
            .unwrap(),
        );
        // Never kill a draft to make room for a different program. The user can
        // save with F1 or explicitly reload with F2 before requesting the switch.
        // An unanswered probe means the app is busy, not that the draft is gone.
        if health[0] != 0 {
            cathedral_user_runtime::write(b"Cathedral desktop: notes busy; kept open\n").unwrap();
            return BUSY;
        }
        if health[4] != 0 {
            cathedral_user_runtime::write(b"Cathedral desktop: unsaved note kept open\n").unwrap();
            return UNSAVED;
        }
    }
    if *active != 0 {
        owner(
            matrix,
            wire::route([wire::STOP, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
        );
        owner(matrix, [wire::STORAGE_REVOKE, 0, 0, 0, 0, 0, 0, 0]);
    }
    *active = 0;
    session.notes_open = false;
    if choice == CLOSE {
        cathedral_user_runtime::write(b"Cathedral desktop: workspace closed\n").unwrap();
        return 0;
    }
    // Notes retains two sealed bitmaps and their replacements, publishes through
    // one more page and reads bulk replies through reader pages. Colors keeps
    // one bitmap and publishes through one page.
    let (private, readers) = if choice == NOTES { (32, 4) } else { (2, 0) };
    owner(
        matrix,
        wire::route([wire::RUN_PAGES, 0, private, readers, 0, 0, 0, 0], 0).unwrap(),
    );
    owner(
        matrix,
        wire::route(
            [
                wire::RUN,
                0,
                choice,
                if choice == NOTES { 0 } else { 2 },
                0,
                0,
                0,
                0,
            ],
            0,
        )
        .unwrap(),
    );
    if choice == NOTES {
        grant_notes(matrix);
        session.notes_open = true;
    }
    owner(
        matrix,
        wire::route([wire::SURFACE_LIMIT, 0, 6, 0, 0, 0, 0, 0], 0).unwrap(),
    );
    owner(
        matrix,
        wire::route([wire::PLACE, 0, 20, 40, 360, 180, KEYS, 0], 0).unwrap(),
    );
    *active = choice;
    cathedral_user_runtime::write(if choice == NOTES {
        b"Cathedral desktop: opened Notes\n"
    } else {
        b"Cathedral desktop: opened Colors\n"
    })
    .unwrap();
    0
}

pub(super) fn run() -> u64 {
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
    workspace(matrix);
    owner(matrix, [wire::RUN_PAGES, 1, 24, 0, 0, 0, 0, 0]);
    owner(matrix, [wire::RUN, 1, 2, 3, 0, 0, 0, 0]);
    owner(matrix, [wire::SURFACE_LIMIT, 1, 2, 0, 0, 0, 0, 0]);
    owner(
        matrix,
        [wire::PLACE, 1, 40, 140, 220, 130, wire::KEY_MASK, 0],
    );
    session.start_input();
    cathedral_user_runtime::write(b"Cathedral desktop: ready; choose a program\n").unwrap();
    let (mut sequence, mut active, mut blocked) = (0, 0, 0);
    loop {
        let reply = call(
            matrix,
            wire::route([MENU_STATUS, active, blocked, 0, 0, 0, 0, 0], 1).unwrap(),
        );
        assert_eq!(reply[0], 0);
        if reply[1] != sequence {
            sequence = reply[1];
            blocked = select(&mut session, &mut active, reply[2]);
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
