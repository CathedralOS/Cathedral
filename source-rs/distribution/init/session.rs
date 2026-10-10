//! Distribution session entrance. profile.json selects the child executables;
//! providers starts and readies them, then supervision owns replacement policy.
mod child;
mod counter;
mod providers;
#[cfg(feature = "recovery-lab")]
mod recovery;
mod supervision;
use cathedral_user_runtime::Error;
pub fn run() -> Result<(), Error> {
    let providers = providers::start()?;
    let counter = child::Service::start(cathedral_session_protocol::launch::COUNTER)?;
    let application = child::Service::start(cathedral_session_protocol::launch::APPLICATION)?;
    supervision::run(providers, application, counter)
}
