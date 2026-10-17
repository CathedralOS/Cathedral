// The distribution owns installed choices and admission. The launcher is an
// ordinary isolated child: it reports a choice, never receives launch authority.
use super::*;
pub(super) const NOTES: u64 = 1;
pub(super) const COLORS: u64 = 2;
pub(super) const CLOSE: u64 = 3;
pub(super) const CLONE_WORLD: u64 = 4;
pub(super) const NEXT_WORLD: u64 = 5;
pub(super) const MENU_STATUS: u64 = 200;
/// Why the last choice was refused, reported to the launcher with the active program.
pub(super) const UNSAVED: u64 = 1;
pub(super) const BUSY: u64 = 2;
pub(super) const FAILED: u64 = 3;

/// The workspace gets one world as its scope; Notes inside it gets the whole
/// of that, so its home is the world.
pub(crate) fn grant_notes(matrix: Pair, world: u64) {
    owner(matrix, [wire::STORAGE_GRANT, 0, 0, world, 0, 0, 0, 0]);
    owner(
        matrix,
        wire::route([wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
    );
}

/// Root slot 0: a nested matrix whose single child is the active program.
pub(super) fn workspace(matrix: Pair) {
    owner(matrix, [wire::RUN_PAGES, 0, 40, 40, 0, 0, 0, 0]);
    owner(matrix, [wire::RUN, 0, 0, 0, 10, 1, 7, 0]);
    owner(matrix, [wire::SURFACE_LIMIT, 0, 8, 0, 0, 0, 0, 0]);
    owner(matrix, [wire::PLACE, 0, 310, 140, 430, 240, KEYS, 0]);
}

/// Root slot 1: the launcher, isolated with navigation keys and no authority.
/// It holds its menu bitmap and the replacement being published.
pub(super) fn menu(matrix: Pair) {
    owner(matrix, [wire::RUN_PAGES, 1, 2 * view::PAGES, 0, 0, 0, 0, 0]);
    owner(matrix, [wire::RUN, 1, 2, 3, 0, 0, 0, 0]);
    owner(matrix, [wire::SURFACE_LIMIT, 1, 2, 0, 0, 0, 0, 0]);
    owner(
        matrix,
        [wire::PLACE, 1, 40, 140, 220, 150, wire::KEY_MASK, 0],
    );
}

/// Carry out a menu choice. Returns why it was refused, or zero when done.
pub(super) fn select(session: &mut admission::Session, active: &mut u64, choice: u64) -> u64 {
    if !(NOTES..=NEXT_WORLD).contains(&choice) || choice == *active {
        return 0;
    }
    if choice == NEXT_WORLD && session.worlds.position().1 == 1 {
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
        // Never kill a draft to make room for a different program or world. The
        // user can save with F1 or explicitly reload with F2 before the switch.
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
    // A world change keeps the program and reopens it in the new world; the
    // clone happens before anything closes, so a refusal leaves everything open.
    let next = match choice {
        CLONE_WORLD => {
            if session.worlds.clone_current(matrix).is_err() {
                cathedral_user_runtime::write(b"Cathedral desktop: world clone refused\n").unwrap();
                return FAILED;
            }
            cathedral_user_runtime::write(b"Cathedral desktop: cloned the world\n").unwrap();
            *active
        }
        NEXT_WORLD => {
            session.worlds.next();
            cathedral_user_runtime::write(b"Cathedral desktop: switched world\n").unwrap();
            *active
        }
        CLOSE => 0,
        program => program,
    };
    if *active != 0 {
        owner(
            matrix,
            wire::route([wire::STOP, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
        );
        owner(matrix, [wire::STORAGE_REVOKE, 0, 0, 0, 0, 0, 0, 0]);
    }
    *active = 0;
    session.notes_open = false;
    if next == 0 {
        if choice == CLOSE {
            cathedral_user_runtime::write(b"Cathedral desktop: workspace closed\n").unwrap();
        }
        return 0;
    }
    open(session, next);
    *active = next;
    0
}

/// Admit a program into the workspace with its pages, grant and placement.
fn open(session: &mut admission::Session, program: u64) {
    let matrix = session.matrix;
    // Notes retains two sealed bitmaps and their replacements, publishes through
    // one more page and reads bulk replies through reader pages. Colors keeps
    // one bitmap and publishes through one page.
    let (private, readers) = if program == NOTES { (32, 4) } else { (2, 0) };
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
                program,
                if program == NOTES { 0 } else { 2 },
                0,
                0,
                0,
                0,
            ],
            0,
        )
        .unwrap(),
    );
    if program == NOTES {
        grant_notes(matrix, session.worlds.current());
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
    cathedral_user_runtime::write(if program == NOTES {
        b"Cathedral desktop: opened Notes\n"
    } else {
        b"Cathedral desktop: opened Colors\n"
    })
    .unwrap();
}
