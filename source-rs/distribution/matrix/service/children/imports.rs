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
    // This host serves interaction, storage and input production. Capture is
    // implemented by Split; recognizing its manifest bit cannot create a provider.
    let offered = offered & (imports::ALL | imports::INPUT_PRODUCER);
    let declaration = imports::from_elf(object.map()?).map_err(status);
    object.unmap()?;
    let declaration = declaration?;
    // Older saved program copies retain their bytes. Their documented legacy
    // bundle required both connections; never infer optionality for old code.
    let required = declaration
        .as_ref()
        .map_or(imports::REQUIRED.as_slice(), |d| d.entries());
    let brokered = declaration.as_ref().is_some_and(|d| d.brokered);
    let offered = if brokered {
        offered
    } else {
        // The positional bundle can represent only interaction and storage.
        // Future broker services must not appear bound without a transport.
        offered & imports::ALL
    };
    let bound = imports::bind(required, offered).map_err(status)?;
    // This host is an interactive matrix. Programs with other startup shapes
    // need a host for that shape rather than being handed undeclared endpoints.
    if bound & imports::INTERACTION == 0 {
        return Err(status(Refusal::Missing(imports::INTERACTION)));
    }
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
