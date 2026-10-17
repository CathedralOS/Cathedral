// The distribution owns installed choices and admission. The launcher is an
// ordinary isolated child: it reports a choice, never receives launch authority.
use super::*;
pub(super) const NOTES: u64 = 1;
pub(super) const COLORS: u64 = 2;
pub(super) const CLOSE: u64 = 3;
pub(super) const MENU_STATUS: u64 = 200;
/// Why the last choice was refused, reported to the launcher with the active program.
pub(super) const UNSAVED: u64 = 1;
pub(super) const BUSY: u64 = 2;

pub(crate) fn grant_notes(matrix: Pair) {
    owner(matrix, [wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0]);
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
pub(super) fn menu(matrix: Pair) {
    owner(matrix, [wire::RUN_PAGES, 1, 24, 0, 0, 0, 0, 0]);
    owner(matrix, [wire::RUN, 1, 2, 3, 0, 0, 0, 0]);
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
