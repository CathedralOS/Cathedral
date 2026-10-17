// The distribution owns installed choices and admission. The launcher is an
// ordinary isolated child: it reports a choice, never receives launch authority.
// Programs come from the notes root, where the harness installed them in the
// order of distribution/profile.json's `programs`; a program's state nests
// under its own object, which is the realm it is granted when it runs.
use super::*;
pub(super) const NOTES: u64 = 1;
pub(super) const COLORS: u64 = 2;
pub(super) const CLOSE: u64 = 3;
pub(super) const MENU_STATUS: u64 = 200;
/// Why the last choice was refused, reported to the launcher with the active program.
pub(super) const UNSAVED: u64 = 1;
pub(super) const BUSY: u64 = 2;

/// The installed programs by object number: the workspace matrix, and inside
/// it Notes and the color app; the launcher is a second copy of the desktop
/// executable beside the workspace, so the two never share a realm.
const WORKSPACE_PROGRAM: u64 = 1;
const NOTES_PROGRAM: u64 = 2;
const COLORS_PROGRAM: u64 = 3;
const MENU_PROGRAM: u64 = 4;
/// Entry roles of the executables (see each main.rs): a matrix takes role 0;
/// the desktop executable's role 2 runs the supplied app, 3 for the launcher
/// and 2 for Colors; Notes takes role 2.
const MATRIX_ROLE: u64 = 0;
const APP_ROLE: u64 = 2;
const MENU_ARGUMENT: u64 = 3;
const COLORS_ARGUMENT: u64 = 2;

/// After provider replacement, re-establish the chain of realms: the workspace
/// holds its own object, and the open program holds its own beneath it.
pub(crate) fn regrant(matrix: Pair, program: u64) {
    owner(
        matrix,
        [wire::STORAGE_GRANT, 0, 0, WORKSPACE_PROGRAM, 0, 0, 0, 0],
    );
    if program != 0 {
        owner(
            matrix,
            wire::route([wire::STORAGE_GRANT, 0, 0, program, 0, 0, 0, 0], 0).unwrap(),
        );
    }
}

/// Root slot 0: a nested matrix whose single child is the active program. It
/// reads the programs it hosts from its own realm, so it needs reader pages
/// for the largest of them beside the bitmaps it retains.
pub(super) fn workspace(matrix: Pair) {
    owner(matrix, [wire::RUN_PAGES, 0, 40, 72, 0, 0, 0, 0]);
    owner(
        matrix,
        [
            wire::RUN_OBJECT,
            0,
            WORKSPACE_PROGRAM,
            MATRIX_ROLE,
            0,
            10,
            1,
            0,
        ],
    );
    owner(matrix, [wire::SURFACE_LIMIT, 0, 8, 0, 0, 0, 0, 0]);
    owner(matrix, [wire::PLACE, 0, 310, 140, 430, 240, KEYS, 0]);
}

/// Root slot 1: the launcher, isolated with navigation keys and no authority.
pub(super) fn menu(matrix: Pair) {
    owner(matrix, [wire::RUN_PAGES, 1, 24, 0, 0, 0, 0, 0]);
    owner(
        matrix,
        [
            wire::RUN_OBJECT,
            1,
            MENU_PROGRAM,
            APP_ROLE,
            MENU_ARGUMENT,
            0,
            0,
            0,
        ],
    );
    owner(matrix, [wire::SURFACE_LIMIT, 1, 2, 0, 0, 0, 0, 0]);
    owner(
        matrix,
        [wire::PLACE, 1, 40, 140, 220, 130, wire::KEY_MASK, 0],
    );
}

/// Carry out a menu choice. Returns why it was refused, or zero when done.
pub(super) fn select(session: &mut admission::Session, active: &mut u64, choice: u64) -> u64 {
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
    }
    *active = 0;
    session.program = 0;
    if choice == CLOSE {
        cathedral_user_runtime::write(b"Cathedral desktop: workspace closed\n").unwrap();
        return 0;
    }
    // Notes retains two sealed bitmaps and their replacements, publishes through
    // one more page and reads bulk replies through reader pages. Colors keeps
    // one bitmap and publishes through one page.
    let (program, argument, private, readers) = if choice == NOTES {
        (NOTES_PROGRAM, 0, 32, 4)
    } else {
        (COLORS_PROGRAM, COLORS_ARGUMENT, 2, 0)
    };
    owner(
        matrix,
        wire::route([wire::RUN_PAGES, 0, private, readers, 0, 0, 0, 0], 0).unwrap(),
    );
    owner(
        matrix,
        wire::route(
            [wire::RUN_OBJECT, 0, program, APP_ROLE, argument, 0, 0, 0],
            0,
        )
        .unwrap(),
    );
    session.program = program;
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
