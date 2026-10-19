//! Resolve sealed program imports before reserving a process or creating channels.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::imports::{self, Refusal};
use cathedral_user_runtime::{Error, memory::Object};

pub(super) struct Bound {
    pub binding: u64,
    pub brokered: bool,
    pub resources: Option<(u64, u64)>,
}
pub(super) fn resolve(object: &mut Object, offered: u64) -> Result<Bound, Error> {
    let declaration = imports::from_elf(object.map()?).map_err(status);
    object.unmap()?;
    let declaration = declaration?;
    // Older saved program copies retain their bytes. Their documented legacy
    // bundle required both connections; never infer optionality for old code.
    let required = declaration
        .as_ref()
        .map_or(imports::REQUIRED.as_slice(), |d| d.entries());
    let bound = imports::bind(required, offered).map_err(status)?;
    // This host is an interactive matrix. Programs with other startup shapes
    // need a host for that shape rather than being handed undeclared endpoints.
    if bound & imports::INTERACTION == 0 {
        return Err(status(Refusal::Missing(imports::INTERACTION)));
    }
    Ok(Bound {
        binding: bound,
        brokered: declaration.as_ref().is_some_and(|d| d.brokered),
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
