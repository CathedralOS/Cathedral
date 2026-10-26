//! Omega selects stock resource offers; the host owns and allocates actual grants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offer {
    pub pages: [u64; 2],
    pub channels: u64,
    pub descendants: u64,
    pub cpu: u64,
    pub imports: u64,
    pub surfaces: u64,
}
pub fn launcher(pages: u64) -> Offer {
    recipe(1, [pages, 0, 0, 0]).expect("launcher recipe")
}
pub fn picker(backed: usize) -> Result<Offer, u64> {
    recipe(2, [0, 0, 0, backed as u64])
}
pub fn background() -> Offer {
    recipe(3, [0; 4]).expect("background recipe")
}
pub fn workspace(pages: [u64; 2], surfaces: u64) -> Offer {
    recipe(4, [pages[0], pages[1], surfaces, 0]).expect("workspace recipe")
}
pub fn user(backed: usize) -> Result<Offer, u64> {
    recipe(5, [0, 0, 0, backed as u64])
}
fn recipe(role: u64, facts: [u64; 4]) -> Result<Offer, u64> {
    let mut p = [0; 128];
    p[1] = 1;
    p[2] = role;
    p[3..7].copy_from_slice(&facts);
    execute(&mut p)?;
    Ok(Offer {
        pages: [p[32], p[33]],
        channels: p[34],
        descendants: p[35],
        cpu: p[36],
        imports: p[37],
        surfaces: p[38],
    })
}
pub struct Facts {
    pub desktop: bool,
    pub app_slot: bool,
    pub pages: [u64; 2],
    pub profile: Option<[u64; 3]>,
    pub resources: Option<(u64, u64)>,
    pub channels: u64,
    pub descendants: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Allocation {
    pub pages: [u64; 2],
    pub channels: u64,
    pub descendants: u64,
    pub surfaces: Option<u64>,
}
pub fn allocate(facts: Facts) -> Result<Allocation, u64> {
    let mut p = [0; 128];
    p[1] = 2;
    p[2] = facts.desktop.into();
    p[3] = facts.app_slot.into();
    p[4..6].copy_from_slice(&facts.pages);
    p[6] = facts.profile.is_some().into();
    p[7..10].copy_from_slice(&facts.profile.unwrap_or([0; 3]));
    p[10] = facts.resources.is_some().into();
    let (channels, descendants) = facts.resources.unwrap_or_default();
    p[11] = channels;
    p[12] = descendants;
    p[13] = facts.channels;
    p[14] = facts.descendants;
    execute(&mut p)?;
    Ok(Allocation {
        pages: [p[32], p[33]],
        channels: p[34],
        descendants: p[35],
        surfaces: (p[36] != 0).then_some(p[37]),
    })
}
pub fn preferred_surfaces(
    requested: u64,
    ceiling: u64,
    descendants: u64,
    explicit: bool,
    backed: usize,
) -> Result<u64, u64> {
    let mut p = [0; 128];
    p[1] = 3;
    p[2..7].copy_from_slice(&[
        requested,
        ceiling,
        descendants,
        explicit.into(),
        backed as u64,
    ]);
    execute(&mut p)?;
    Ok(p[32])
}
#[inline(never)]
fn execute(p: &mut [u64; 128]) -> Result<(), u64> {
    p[0] = 17;
    match super::native::decide(p) {
        0 => Ok(()),
        status => Err(status),
    }
}
