//! Stock launch target/role choices in Omega; authenticated grants stay with the host.
pub struct Facts {
    pub busy: bool,
    pub desktop: bool,
    pub chrome: bool,
    pub requester: Option<usize>,
    pub occupied: u64,
    pub unavailable: u64,
    pub storing: bool,
    pub receipts: bool,
    pub tickets: bool,
    pub standing: u64,
    pub binding: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Decision {
    pub status: u64,
    pub slot: usize,
    pub full: bool,
}
#[inline(never)]
pub fn decide(object: u64, withhold: u64, f: Facts) -> Decision {
    let mut p = [0; 128];
    p[..16].copy_from_slice(&[
        11,
        5,
        f.busy.into(),
        f.desktop.into(),
        f.chrome.into(),
        f.requester.map_or(u64::MAX, |s| s as u64),
        f.occupied,
        f.unavailable,
        f.storing.into(),
        f.receipts.into(),
        f.tickets.into(),
        object,
        withhold,
        cathedral_platform_contracts::imports::SUPPORTED,
        f.standing,
        f.binding.into(),
    ]);
    let status = super::native::decide(&mut p);
    Decision {
        status,
        slot: p[24] as usize,
        full: p[25] != 0,
    }
}
pub struct Roles {
    pub selection: bool,
    pub child: bool,
    pub desktop: bool,
}
pub fn roles(
    login: bool,
    entered: bool,
    slot: usize,
    desktop: bool,
    requester: Option<usize>,
) -> Roles {
    let flags = role_decision(
        6,
        [
            login.into(),
            entered.into(),
            slot as u64,
            desktop.into(),
            requester.map_or(0, |s| s.saturating_add(1) as u64),
        ],
    );
    Roles {
        selection: flags & 1 != 0,
        child: flags & 2 != 0,
        desktop: flags & 4 != 0,
    }
}
pub fn inspect_picker(upstream: bool, slot: usize, scope: u64) -> bool {
    role_decision(7, [upstream.into(), slot as u64, scope, 0, 0]) == 1
}
pub fn matches_picker(login: Option<u64>, users: Option<u64>, object: u64, scope: u64) -> bool {
    role_decision(
        8,
        [login.unwrap_or(0), users.unwrap_or(0), object, scope, 0],
    ) == 1
}
#[inline(never)]
fn role_decision(operation: u64, facts: [u64; 5]) -> u64 {
    let mut p = [0; 128];
    p[0] = 11;
    p[1] = operation;
    p[2..7].copy_from_slice(&facts);
    super::native::decide(&mut p)
}
