//! Restore the private counter, commit both records, then verify them on health requests.
#[cfg(feature = "recovery-lab")]
mod probes;
mod state;
use cathedral_user_runtime::{Error, ipc::Handle, storage::Client, write};
pub fn run(_generation: u64) -> Result<(), Error> {
    let mut store = Client::at(0)?;
    state::advance(&mut store)?;
    #[cfg(feature = "recovery-lab")]
    probes::catalog(&mut store, _generation)?;
    let input = Handle::bootstrap(0)?;
    let output = Handle::bootstrap(1)?;
    write(b"Cathedral: private counter ready\n")?;
    loop {
        let mut request = [0; 64];
        let len = input.receive(&mut request)?;
        if &request[..len] == b"health" {
            // A replaced storage provider invalidates the old pair. Retry reads only.
            if state::read(&mut store).is_err() {
                state::read(&mut store)?;
            }
            output.send(b"ready")?;
        } else {
            output.send(b"denied")?;
        }
    }
}
