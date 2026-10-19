// Create a matrix startup bundle, move it into a child, then accept its control port.
// The caller supplies code/funding and installs local authority only after success.
use crate::{
    Error,
    ipc::Handle,
    task::{Child, Launch},
};
type Pair = (Handle, Handle);

/// Successful admission transfers these retained rights to the caller. No Drop
/// stops the process implicitly; the owner controls teardown and reservation release.
pub struct Admitted {
    pub task: Child,
    pub control: Pair,
    pub data: Pair,
    pub notify: Handle,
    pub storage: Option<Pair>,
}

pub fn spawn(
    target: Launch,
    argument: u64,
    loans: &[Launch],
    storage: bool,
) -> Result<Admitted, Error> {
    let port = target.port()?;
    let connections = connections::Connections::create(storage)?;
    let child = match target.spawn_tree(argument, connections.initial(), loans) {
        Ok(child) => child,
        Err(error) => {
            connections.rollback()?;
            return Err(error);
        }
    };
    let control = match port.connect() {
        Ok(pair) => pair,
        Err(error) => {
            child.cancel()?;
            child.wait()?;
            connections.close_parent()?;
            return Err(error);
        }
    };
    Ok(Admitted {
        task: child,
        control,
        data: connections.data,
        notify: connections.notify,
        storage: connections.storage,
    })
}

impl Admitted {
    /// After reclaiming the task and detaching server slots, close retained rights.
    /// Does not release a launch reservation or choose a restart policy.
    pub fn close_connections(self) -> Result<(), Error> {
        for endpoint in [
            self.control.0,
            self.control.1,
            self.data.0,
            self.data.1,
            self.notify,
        ] {
            endpoint.close()?;
        }
        if let Some((input, output)) = self.storage {
            input.close()?;
            output.close()?;
        }
        Ok(())
    }
}
