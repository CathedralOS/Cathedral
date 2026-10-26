//! Resolve sealed program imports before reserving a process or creating channels.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::imports::{self, Refusal};
use cathedral_user_runtime::{Error, memory::Object};

pub(super) struct Bound {
    pub binding: u64,
    pub profile: Option<[u64; 3]>,
    pub brokered: bool,
    pub resources: Option<(u64, u64)>,
}
pub(super) fn resolve(object: &mut Object, offered: u64) -> Result<Bound, Error> {
    let declaration =
        cathedral_stock_matrix::policy::imports::from_elf(object.map()?).map_err(status);
    object.unmap()?;
    let declaration = declaration?;
    let brokered = declaration.as_ref().is_some_and(|d| d.brokered);
    // Omega validates the declaration, selects the stock offer and matches it.
    // Sealed image mapping remains here, before any grant is reserved.
    let bound = cathedral_stock_matrix::policy::imports::resolve(declaration.as_ref(), offered)
        .map_err(status)?;
    Ok(Bound {
        binding: bound,
        profile: declaration.as_ref().and_then(|d| d.profile),
        brokered,
        resources: declaration.as_ref().and_then(|d| d.resources),
    })
}
fn status(refusal: Refusal) -> Error {
    Error(match refusal {
        Refusal::Malformed => abi::BAD_EXECUTABLE,
        Refusal::Missing(_) => imports::MISSING,
        Refusal::Version(_) => imports::INCOMPATIBLE,
    } as i64)
}
