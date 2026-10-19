//! Explicitly subdivide the owner's conserved capacity before root admission.
use super::*;
#[cfg(feature = "shell")]
use cathedral_user_runtime::resources;
use cathedral_user_runtime::task::Executable;
pub(super) fn assign(root: Launch) -> Result<(), Error> {
    let capacity = task::available_slots()?;
    assert_eq!(capacity, if cfg!(feature = "shell") { 6 } else { 4 });
    #[cfg(not(feature = "shell"))]
    let pressure = Executable::at(2).is_ok();
    // Descendant reservations return to the surviving distribution owner on root
    // death. Re-fund the replacement explicitly; restarting never mints budgets.
    // The numbers mirror what each profile grants the root matrix. The shell's
    // root is a hub over a user matrix: it retains that subtree's images beside
    // their replacements and lends the user matrix a reader budget of its own.
    cathedral_user_runtime::resources::assign(root, limits())?;
    root.assign_slots(capacity)?;
    root.assign_memory((capacity + 1) * abi::MAX_ADMISSION_FRAMES as u64)?;
    // The shell loads ordinary programs from storage, but its recovery screen
    // must remain loadable when Login itself is missing or unusable.
    #[cfg(feature = "shell")]
    root.allow_executable(Executable::at(0)?)?;
    #[cfg(not(feature = "shell"))]
    for index in 0..if pressure { 3 } else { 2 } {
        root.allow_executable(Executable::at(index)?)?;
    }
    Ok(())
}

pub(super) fn limits() -> cathedral_user_runtime::resources::Limits {
    cathedral_user_runtime::resources::Limits {
        private_pages: if cfg!(feature = "shell") {
            256
        } else if cfg!(feature = "notes") {
            128
        } else {
            16
        },
        reader_pages: if cfg!(feature = "shell") {
            384
        } else if cfg!(feature = "notes") {
            128
        } else {
            8
        },
        channels: if cfg!(feature = "shell") { 32 } else { 20 },
    }
}

#[cfg(feature = "shell")]
pub(super) fn reset() {
    // The root account is idle and every transferred endpoint has been retired.
    // Restore the root's fixed allowance before measuring. Combining it all
    // into the owner would exceed the per-account cap; conservation still holds.
    let root = Launch::at(2).unwrap();
    root.assign_slots(0).unwrap();
    root.assign_memory(0).unwrap();
    resources::assign(root, funding::limits()).unwrap();
}
#[cfg(feature = "shell")]
pub(super) fn snapshot() -> [u64; 10] {
    let resources::Budget { limits, used } = resources::budget().unwrap();
    let physical = resources::physical_budget().unwrap();
    [
        task::available_slots().unwrap(),
        physical.reserved,
        physical.charged,
        limits.private_pages,
        limits.reader_pages,
        limits.channels,
        used.channels,
        used.private_pages,
        used.reader_pages,
        ipc::budget().unwrap().charged_handles,
    ]
}
