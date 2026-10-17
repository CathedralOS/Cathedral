// The distribution owns installed choices and admission. The launcher is an
// ordinary isolated child: it reports a choice, never receives launch authority.
use super::*;
const NOTES: u64 = 1;
const COLORS: u64 = 2;
const CLOSE: u64 = 3;
const MENU_STATUS: u64 = 200;

pub(super) fn grant_notes(matrix: Pair) {
    owner(matrix, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
    owner(
        matrix,
        wire::route([wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
    );
}

fn workspace(matrix: Pair) {
    owner(matrix, [wire::RUN_PAGES, 0, 24, 24, 0, 0, 0, 0]);
    owner(matrix, [wire::RUN, 0, 0, 0, 10, 1, 7, 0]);
    owner(matrix, [wire::SURFACE_LIMIT, 0, 8, 0, 0, 0, 0, 0]);
    owner(matrix, [wire::PLACE, 0, 310, 140, 430, 240, KEYS, 0]);
}

fn select(session: &mut admission::Session, active: &mut u64, choice: u64) -> bool {
    if !(NOTES..=CLOSE).contains(&choice) || choice == *active {
        return false;
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
        if health[0] != 0 || health[4] != 0 {
            cathedral_user_runtime::write(b"Cathedral desktop: unsaved note kept open\n").unwrap();
            return true;
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
        return false;
    }
    owner(
        matrix,
        wire::route(
            [
                wire::RUN_PAGES,
                0,
                if choice == NOTES { 24 } else { 1 },
                0,
                0,
                0,
                0,
                0,
            ],
            0,
        )
        .unwrap(),
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
        wire::route([wire::SURFACE_LIMIT, 0, 4, 0, 0, 0, 0, 0], 0).unwrap(),
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
    false
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
    let (mut sequence, mut active, mut blocked) = (0, 0, false);
    loop {
        let reply = call(
            matrix,
            wire::route([MENU_STATUS, active, u64::from(blocked), 0, 0, 0, 0, 0], 1).unwrap(),
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
