//! Start the profile's providers and establish readiness before admitting the app.
//! Descend into each provider for its control protocol and recovery handshake.
pub(super) mod display;
pub(super) mod input;
pub(super) mod storage;
use super::child::Service;
use cathedral_session_protocol::launch;
use cathedral_user_runtime::Error;
pub(super) struct Providers {
    pub display: Service,
    pub input: Service,
    pub storage: Service,
}
pub(super) fn start() -> Result<Providers, Error> {
    let mut display = Service::start(launch::DISPLAY)?;
    let mut input = Service::start(launch::INPUT)?;
    input::await_ready(&mut input)?;
    display::ensure_ready(&mut display)?;
    let mut storage = Service::start(launch::STORAGE)?;
    storage::ensure_ready(&mut storage)?;
    Ok(Providers {
        display,
        input,
        storage,
    })
}
