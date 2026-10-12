//! Validate all funding and names before installing a new executable/account pairing.
use super::*;
use crate::user_tasks::lifecycle::supervision::Supervisor;
pub(super) fn reserve(
    session: &mut Session,
    caller: usize,
    executable: u64,
    address: u64,
) -> Result<u64, u64> {
    let template = session.executables.resolve(caller, executable)?;
    let mut bytes = [0; 48];
    // SAFETY: Serialized trap; full source range checked before any mutation.
    if !unsafe {
        session.tasks[caller]
            .space
            .as_ref()
            .unwrap()
            .copy_from_user(address, &mut bytes)
    } {
        return Err(abi::BAD_ADDRESS);
    }
    let words: [u64; 6] =
        core::array::from_fn(|i| u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap()));
    if words[3] > abi::MAX_TASKS as u64
        || words[4] == 0
        || words[4] > abi::MAX_ADMISSION_FRAMES as u64
    {
        return Err(abi::INVALID_ARGUMENT);
    }
    let (child, mask) = session.pool.prepare(caller, words[3] as usize)?;
    let index = session
        .launches
        .iter()
        .position(|l| l.model.child == child)
        .ok_or(abi::NO_MEMORY)?;
    assert!(session.launches[index].dynamic && session.launches[index].template.is_none());
    let requested = core::array::from_fn(|i| words[i] as usize);
    // Construct the same lender chain used for explicit launch loans. If the
    // creator dies, its new account returns to the nearest surviving ancestor.
    let mut ancestry = [0; abi::MAX_TASKS];
    let mut len = 0;
    let mut current = Some(caller);
    while let Some(owner) = current {
        ancestry[len] = owner;
        len += 1;
        current = session.scheduler.parent(owner);
    }
    let root = ancestry[len - 1];
    let mut model = Supervisor::new(session.names.pair(root, root)?, root, root, child)?;
    for &owner in ancestry[..len - 1].iter().rev() {
        let returned = session.names.pair(model.owner, model.owner)?;
        model.lend(owner, session.names.pair(owner, owner)?, returned);
    }
    let physical = usize::try_from(words[5]).map_err(|_| abi::INVALID_ARGUMENT)?;
    session
        .memory
        .funding
        .check_assign(caller, child, physical)?;
    // Names may be burned on failure, but no capacity or capability is lost.
    crate::user_tasks::resources::assign(
        &mut session.ipc,
        &mut session.memory.model,
        caller,
        child,
        requested,
    )?;
    session
        .memory
        .funding
        .assign(caller, child, physical)
        .expect("validated physical assignment");
    session.pool.commit(caller, child, mask);
    let ticket = model.launch(caller).unwrap();
    let launch = &mut session.launches[index];
    launch.model = model;
    launch.template = Some(template);
    launch.frame_limit = words[4] as usize;
    // A reused arena slot must not reveal another reservation's CPU history.
    // Accounting/debt itself remains persistent in the scheduler.
    launch.cpu_origin = session.scheduler.cpu_report(child).direct_us;
    // Executable authority supplies code, never a device aperture or physical input.
    // Clock access can only be inherited from an authorized creator.
    session.clock.allow(child, session.clock.allowed(caller));
    session.scheduler.set_parent(child, caller);
    Ok(ticket)
}
