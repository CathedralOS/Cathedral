// Matrix startup connections: parent admission builds them, child entry reads them.
// These names describe local connections, never global services or disk locations.
// Parent construction and rollback: admission/admission.rs. Children load this view.
use crate::{Error, abi, ipc::Handle, services::Wake, task::Environment};
pub use cathedral_platform_contracts::imports;

/// Typed view of a version-1 matrix environment with optional persistence.
/// An absent storage endpoint is not a dummy connection returning errors.
pub struct OptionalConnections {
    pub interaction: (Handle, Handle),
    pub wake: Wake,
    pub storage: Option<(Handle, Handle)>,
}
impl OptionalConnections {
    pub fn load(environment: &Environment) -> Result<Self, Error> {
        if !matches!(environment.len(), 3 | 5) {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        Ok(Self {
            interaction: (
                environment.endpoint(0).unwrap(),
                environment.endpoint(1).unwrap(),
            ),
            wake: Wake::Copied(environment.endpoint(2).unwrap()),
            storage: environment.endpoint(3).zip(environment.endpoint(4)),
        })
    }
}
pub struct Connections {
    pub interaction: (Handle, Handle),
    pub wake: Wake,
    pub storage: (Handle, Handle),
}
impl Connections {
    pub fn load(environment: &Environment) -> Result<Self, Error> {
        if environment.len() != 5 {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        Ok(Self {
            interaction: (
                environment.endpoint(0).unwrap(),
                environment.endpoint(1).unwrap(),
            ),
            wake: Wake::Copied(environment.endpoint(2).unwrap()),
            storage: (
                environment.endpoint(3).unwrap(),
                environment.endpoint(4).unwrap(),
            ),
        })
    }
}
