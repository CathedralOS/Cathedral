// The distribution owns admission and standing policy. What runs is chosen by
// the chooser from what its realm holds; the desktop names only what the
// harness installed at the top of the notes root, in profile order: the
// chooser and the folder of users it is shown. A user realm is a copy of the
// template inside that folder: a matrix whose subtree holds its chrome, Notes
// and Colors, in that order.
use super::*;
const CHOOSER: u64 = 1;
const USERS: u64 = 2;
/// A subtree copy numbers the copies in listing order after the realm's own
/// object, so a user's chrome is the object right after its realm.
const CHROME_OFFSET: u64 = 1;
/// Three 12-page tiles and one replacement fit in the chooser allowance.
const CHOOSER_PAGES: u64 = 48;

/// Root slot 0: the chooser, shown the folder of users and nothing else.
pub(super) fn chooser(matrix: Pair, extent: [u64; 2]) {
    owner(matrix, [wire::RUN_PAGES, 0, CHOOSER_PAGES, 0, 0, 0, 0, 0]);
    owner(matrix, [wire::RUN_OBJECT, 0, CHOOSER, USERS, 1, 0, 0, 0]);
    select_user(matrix, extent);
}

/// Root slot 1: whatever the chooser launches, which is a user matrix. It
/// hosts two children, reads programs of up to 64 pages, and retains their
/// bitmaps and replacements. The two stock app tiles retain 24 reader pages,
/// leaving space for a program of up to 64 pages during admission.
pub(super) fn workspace(matrix: Pair, [width, height]: [u64; 2]) {
    owner(matrix, [wire::LAUNCH_POLICY, 1, 80, 96, 10, 2, 0, 11]);
    owner(
        matrix,
        [wire::LAUNCH_PLACE, 1, 0, 0, width, height, KEYS, 0],
    );
}

/// A user matrix appeared with `realm`: give it its chrome in its first slot,
/// holding the whole user realm so it sees what it may launch, and open its
/// second slot to what the chrome launches. Notes needs 32 private pages and
/// 4 reader pages; the color app fits in that.
pub(super) fn furnish(matrix: Pair, extent: [u64; 2], realm: u64) {
    let chrome = realm + CHROME_OFFSET;
    owner(
        matrix,
        wire::route([wire::RUN_PAGES, 0, CHOOSER_PAGES, 0, 0, 0, 0, 0], 1).unwrap(),
    );
    owner(
        matrix,
        wire::route([wire::RUN_OBJECT, 0, chrome, 0, 0, 0, 0, 0], 1).unwrap(),
    );
    owner(
        matrix,
        wire::route([wire::LAUNCH_POLICY, 1, 32, 4, 0, 0, 0, 6], 1).unwrap(),
    );
    layout::enter_user(matrix, extent);
}

/// The realm a child of `matrix` holds, or zero. The user matrix is root
/// slot 1, so requests into it are routed there.
pub(super) fn realm_of(matrix: Pair, words: wire::Packet) -> u64 {
    let reply = call(matrix, words);
    if reply[0] == 0 { reply[2] } else { 0 }
}

/// After provider replacement, re-establish every realm the tree held: the
/// chooser's folder, the user matrix's realm, its chrome over all of it, and
/// its app.
pub(crate) fn regrant(matrix: Pair, user: u64, app: u64) {
    owner(matrix, [wire::STORAGE_GRANT, 0, 0, USERS, 0, 0, 0, 0]);
    if user == 0 {
        return;
    }
    owner(matrix, [wire::STORAGE_GRANT, 1, 0, user, 0, 0, 0, 0]);
    owner(
        matrix,
        wire::route([wire::STORAGE_GRANT, 0, 0, 0, 0, 0, 0, 0], 1).unwrap(),
    );
    if app != 0 {
        owner(
            matrix,
            wire::route([wire::STORAGE_GRANT, 1, 0, app, 0, 0, 0, 0], 1).unwrap(),
        );
    }
}
