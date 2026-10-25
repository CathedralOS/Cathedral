// Create a matrix startup bundle, move it into a child, then accept its control port.
// The caller supplies code/funding and installs local authority only after success.
use crate::{
    Error, abi,
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
    pub notify: crate::services::WakeSource,
    pub storage: Option<Pair>,
    pub producer: Option<Pair>,
    pub capture: Option<Pair>,
    pub services: Option<crate::services::Offer>,
}

pub fn spawn(
    target: Launch,
    argument: u64,
    loans: &[Launch],
    storage: bool,
) -> Result<Admitted, Error> {
    spawn_mode(target, argument, loans, storage, None)
}
/// Admit only the broker pair; other preallocated endpoints remain in its offer.
pub fn spawn_brokered(
    target: Launch,
    argument: u64,
    loans: &[Launch],
    granted: u64,
) -> Result<Admitted, Error> {
    use cathedral_platform_contracts::imports;
    if granted & !imports::SUPPORTED != 0 || granted & imports::INTERACTION == 0 {
        return Err(Error(crate::abi::INVALID_ARGUMENT as i64));
    }
    spawn_mode(
        target,
        argument,
        loans,
        granted & imports::STORAGE != 0,
        Some(granted),
    )
}
fn spawn_mode(
    target: Launch,
    argument: u64,
    loans: &[Launch],
    storage: bool,
    granted: Option<u64>,
) -> Result<Admitted, Error> {
    let port = target.port()?;
    let producer =
        granted.is_some_and(|g| g & cathedral_platform_contracts::imports::INPUT_PRODUCER != 0);
    let capture = granted.is_some_and(|g| g & cathedral_platform_contracts::imports::CAPTURE != 0);
    let connections =
        connections::Connections::create(storage, producer, capture, granted.is_some())?;
    let child = match target.spawn_tree(
        argument,
        if granted.is_some() {
            connections.broker_initial()
        } else {
            connections.initial()
        },
        loans,
    ) {
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
            if granted.is_some() {
                connections.close_deferred()?;
            }
            connections.close_parent()?;
            return Err(error);
        }
    };
    Ok(Admitted {
        services: granted.map(|granted| {
            crate::services::Offer::new(granted, connections.deferred(), connections.wake.unwrap())
        }),
        task: child,
        control,
        data: connections.data,
        notify: connections.notify,
        storage: connections.storage,
        producer: connections.producer,
        capture: connections.capture,
    })
}

impl Admitted {
    /// After reclaiming the task and detaching server slots, close retained rights.
    /// Does not release a launch reservation or choose a restart policy.
    pub fn close_connections(self) -> Result<(), Error> {
        if let Some(services) = self.services {
            services.close()?;
        }
        self.notify.close()?;
        for endpoint in [self.control.0, self.control.1] {
            endpoint.close()?;
        }
        // Server removal may already retire selected shared slots. Endpoint
        // names are never reused: only absence of these exact names is harmless.
        for (input, output) in [Some(self.data), self.storage, self.producer, self.capture]
            .into_iter()
            .flatten()
        {
            for endpoint in [input, output] {
                match endpoint.close() {
                    Ok(()) => (),
                    Err(Error(code)) if code == abi::BAD_HANDLE as i64 => (),
                    Err(error) => return Err(error),
                }
            }
        }
        Ok(())
    }
}
