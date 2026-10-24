// Native seam for the distribution-owned Omega chooser decisions.
// Start across the seam at source/distribution/applications/chooser/policy/main.omg.
// This is an app-local synchronous packet, not the platform app-host ABI or IPC.
use cathedral_desktop_matrix_contracts::matrix as wire;
use cathedral_platform_contracts::storage::{ENTRIES_PER_PAGE, metadata};

pub const ENTRIES: usize = wire::config::LAUNCHER_SURFACES - 2;
pub const USER_ENTRIES: usize = 4;
pub const NEW: usize = ENTRIES;

fn request(kind: u64, fields: &[u64]) -> [u64; 128] {
    let mut packet = [0; 128];
    packet[0] = kind;
    packet[1..1 + fields.len()].copy_from_slice(fields);
    packet
}
fn evaluate(packet: &mut [u64; 128]) {
    assert_eq!(native::decide(packet), 1, "Omega chooser decision refused");
}
pub fn verify() {
    let mut packet = request(0, &[]);
    evaluate(&mut packet);
    assert_eq!(
        packet[1..5],
        [
            128,
            ENTRIES as u64,
            USER_ENTRIES as u64,
            ENTRIES_PER_PAGE as u64
        ]
    );
}
pub fn classify(value: metadata::Metadata) -> (u64, [u8; 13]) {
    let mut packet = request(1, &[]);
    for (word, byte) in packet[16..80].iter_mut().zip(value.encode()) {
        *word = byte.into();
    }
    evaluate(&mut packet);
    (packet[1], core::array::from_fn(|i| packet[80 + i] as u8))
}
pub fn catalogue(
    picker: bool,
    own: u64,
    template: u64,
    background: u64,
    ids: &[u64],
) -> Option<([u64; ENTRIES], usize)> {
    assert!(ids.len() <= ENTRIES_PER_PAGE);
    let mut packet = request(
        2,
        &[picker.into(), own, template, background, ids.len() as u64],
    );
    packet[16..16 + ids.len()].copy_from_slice(ids);
    if native::decide(&mut packet) != 1 {
        return None;
    }
    let count = packet[80] as usize;
    assert!(count <= ENTRIES);
    Some((core::array::from_fn(|i| packet[81 + i]), count))
}
pub struct Geometry {
    pub count: usize,
    pub dimensions: (usize, usize),
    pub origin: (u64, u64),
    pub tasks_left: u64,
    pub items: [usize; ENTRIES],
}
pub fn geometry(picker: bool, count: usize, width: u64, height: u64, tile: usize) -> Geometry {
    let mut p = request(
        3,
        &[picker.into(), count as u64, width, height, tile as u64],
    );
    evaluate(&mut p);
    Geometry {
        count: p[80] as usize,
        dimensions: (p[81] as usize, p[82] as usize),
        origin: (p[83], p[84]),
        tasks_left: p[85],
        items: core::array::from_fn(|i| p[86 + i] as usize),
    }
}
#[allow(clippy::too_many_arguments)]
pub fn selection(
    selected: usize,
    event: wire::Packet,
    view: wire::Packet,
    clicked: bool,
    count: usize,
    picker: bool,
    managed: bool,
    retry: bool,
) -> (usize, bool, bool) {
    let mut p = request(
        4,
        &[
            picker.into(),
            count as u64,
            view[2],
            view[3],
            selected as u64,
            event[2],
            event[3],
            event[4],
            clicked.into(),
            managed.into(),
            retry.into(),
        ],
    );
    evaluate(&mut p);
    (p[80] as usize, p[81] != 0, p[82] != 0)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Launch(u64),
    Copy(u64),
    Settings,
    Manage,
    Retry,
}
pub fn action(
    picker: bool,
    count: usize,
    selected: usize,
    ids: &[u64; ENTRIES],
    retry: bool,
) -> Action {
    let mut p = request(
        5,
        &[
            picker.into(),
            count as u64,
            selected as u64,
            ids.get(selected).copied().unwrap_or(0),
            ids[0],
            retry.into(),
        ],
    );
    evaluate(&mut p);
    match p[80] {
        0 => Action::None,
        1 => Action::Launch(p[81]),
        2 => Action::Copy(p[81]),
        3 => Action::Settings,
        4 => Action::Manage,
        5 => Action::Retry,
        _ => unreachable!(),
    }
}
#[derive(Clone, Copy)]
pub enum Operation {
    Launch = 0,
    Close = 2,
    Copy = 3,
    Retry = 4,
}
pub fn outcome(operation: Operation, reply: wire::Packet) -> (&'static str, Option<u64>) {
    let mut p = request(6, &[operation as u64, reply[0], reply[2]]);
    evaluate(&mut p);
    (message(p[80]), (p[81] != 0).then_some(p[81]))
}
pub fn task_click(
    entries: &[wire::Packet; 3],
    view: wire::Packet,
    event: wire::Packet,
    clicked: bool,
    left: u64,
) -> Option<(u64, bool)> {
    let mut p = request(
        7,
        &[
            left,
            view[2],
            view[3],
            event[2],
            event[3],
            event[4],
            clicked.into(),
        ],
    );
    for (target, entry) in p[16..40].chunks_exact_mut(8).zip(entries) {
        target.copy_from_slice(entry);
    }
    evaluate(&mut p);
    (p[82] != 0).then_some((p[80], p[81] != 0))
}
pub fn after_select(status: u64, close: bool) -> (bool, &'static str) {
    let mut p = request(8, &[status, close.into()]);
    evaluate(&mut p);
    (p[80] != 0, message(p[81]))
}
pub fn observation(reply: wire::Packet) -> (bool, &'static str) {
    let mut p = request(9, &[reply[0], reply[1]]);
    evaluate(&mut p);
    (p[80] != 0, message(p[81]))
}
pub fn user_name(count: usize) -> [u8; 13] {
    let mut p = request(10, &[count as u64]);
    evaluate(&mut p);
    core::array::from_fn(|i| p[80 + i] as u8)
}
fn message(tag: u64) -> &'static str {
    match tag {
        0 => "",
        1 => "CANNOT OPEN",
        2 => "TRY AGAIN",
        3 => "NO ROOM",
        4 => "UNSAVED WORK",
        5 => "APP STOPPED",
        _ => unreachable!(),
    }
}
