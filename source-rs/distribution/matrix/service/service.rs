// Consume local invalidations, wake children, then dispatch ownership and input.
use cathedral_kernel_contracts::user as abi;
use cathedral_matrix::world::{Grant, Rect};
const CHILDREN: usize = 5;
const LAST_CHILD: u64 = CHILDREN as u64 - 1;
const SOURCE: usize = CHILDREN;
const STORAGE_BASE: usize = CHILDREN + 1;
const PRODUCER_BASE: usize = STORAGE_BASE + CHILDREN;
const STREAMS: usize = PRODUCER_BASE + CHILDREN + 1;
const BACKGROUND: usize = 4;
type World = cathedral_matrix::world::World<CHILDREN>;
use cathedral_desktop_matrix_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    event::Interest,
    server::{Event, Server},
    task::Environment,
};
use parent::Parent;

/// Stock sibling stacking: slot zero below slot one, regardless of launch order.
fn surface_limit(world: &mut World, slot: usize, count: usize) -> Result<(), u64> {
    let base = match slot {
        0 => 0,
        1 => world
            .backed_surfaces()
            .checked_sub(count)
            .ok_or(abi::NO_MEMORY)?,
        _ => return Err(abi::DENIED),
    };
    world.surface_range(slot, base, count)
}

/// Whether this matrix still has to read its realm for what to become. The
/// root adapter and owner-driven fixtures are configured by commands instead.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Startup {
    Hosted,
    Realm,
}
/// Omega owns the turn sequence; the host executes one operation and reports facts.
pub fn run(environment: Environment) -> Result<(), Error> {
    let mut host = host::Host::initialize(environment)?;
    let mut session = cathedral_stock_matrix::policy::session::State::default();
    loop {
        let (flags, event) = host.facts(session.observations());
        let command = session
            .command(flags, event)
            .expect("completed host operation");
        let result = host.execute(command)?;
        assert!(session.complete(command, result));
    }
}
