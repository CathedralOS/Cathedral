//! Startup capabilities supplied by platform/services/matrix/service/children.
//! These names describe local connections, never global services or disk locations.
use crate::{Error, abi, ipc::Handle, task::Environment};
pub struct Connections {
    pub interaction: (Handle, Handle),
    pub wake: Handle,
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
            wake: environment.endpoint(2).unwrap(),
            storage: (
                environment.endpoint(3).unwrap(),
                environment.endpoint(4).unwrap(),
            ),
        })
    }
}
