// Load one scoped broker, acquire approved connections, then call providers directly.
// transport.rs owns acquisition/interaction correlation; offer.rs owns the
// receiver custody selected by the matrix using authenticated server identity.
use crate::{
    Error, abi,
    ipc::{self, Handle},
    task::Environment,
    time,
};
use cathedral_platform_contracts::{imports, matrix, services as wire};
pub type Pair = (Handle, Handle);
#[derive(Clone, Copy)]
pub struct Interaction {
    pub pair: Pair,
    pub wake: Wake,
}
/// Ordinary apps serialize calls; owners may choose a larger negotiated window.
pub type Services = ServiceBindings<1>;
pub struct ServiceBindings<const WIDTH: usize> {
    broker: Pair,
    shared: Option<crate::connection::Pipeline<WIDTH>>,
    interaction: Option<Interaction>,
    storage: Option<Pair>,
    storage_absent: bool,
    producer: Option<Pair>,
    producer_absent: bool,
    capture: Option<Pair>,
    capture_absent: bool,
    failed: bool,
    brokered: bool,
}
impl<const WIDTH: usize> ServiceBindings<WIDTH> {
    pub fn load(environment: &Environment) -> Result<Self, Error> {
        if environment.len() == 2 {
            return Ok(Self {
                broker: (
                    environment.endpoint(0).unwrap(),
                    environment.endpoint(1).unwrap(),
                ),
                shared: None,
                interaction: None,
                storage: None,
                storage_absent: false,
                producer: None,
                producer_absent: false,
                capture: None,
                capture_absent: false,
                failed: false,
                brokered: true,
            });
        }
        // Explicit compatibility for older catalogue/boot compositions. No lookup.
        let old = crate::matrix::OptionalConnections::load(environment)?;
        Ok(Self {
            broker: old.interaction,
            shared: None,
            interaction: Some(Interaction {
                pair: old.interaction,
                wake: old.wake,
            }),
            storage: old.storage,
            storage_absent: old.storage.is_none(),
            producer: None,
            producer_absent: true,
            capture: None,
            capture_absent: true,
            failed: false,
            brokered: false,
        })
    }
    pub fn is_brokered(&self) -> bool {
        self.brokered
    }
    pub fn interaction(&mut self, minimum: u64, maximum: u64) -> Result<Interaction, Error> {
        version(minimum, maximum)?;
        if let Some(connection) = self.interaction {
            return Ok(connection);
        }
        let wake = self.acquire_wake(minimum, maximum)?;
        let connection = Interaction {
            pair: self.broker,
            wake,
        };
        self.interaction = Some(connection);
        Ok(connection)
    }
    pub fn storage(&mut self, minimum: u64, maximum: u64) -> Result<Pair, Error> {
        version(minimum, maximum)?;
        if self.storage_absent {
            return Err(Error(wire::UNAVAILABLE as i64));
        }
        if let Some(connection) = self.storage {
            return Ok(connection);
        }
        let send = match self.acquire(imports::STORAGE, minimum, maximum, 0) {
            Ok(send) => send,
            Err(Error(code)) if code == wire::UNAVAILABLE as i64 => {
                self.storage_absent = true;
                return Err(Error(code));
            }
            Err(error) => return Err(error),
        };
        let receive = match self.acquire(imports::STORAGE, minimum, maximum, 1) {
            Ok(receive) => receive,
            Err(error) => {
                send.close()?;
                return Err(error);
            }
        };
        self.storage = Some((send, receive));
        Ok((send, receive))
    }
    /// Acquire the explicitly offered producer pair, distinct from interaction.
    /// The provider authenticates this endpoint and its bounded production grant.
    pub fn input_producer(&mut self, minimum: u64, maximum: u64) -> Result<Pair, Error> {
        version(minimum, maximum)?;
        if self.producer_absent {
            return Err(Error(wire::UNAVAILABLE as i64));
        }
        if let Some(pair) = self.producer {
            return Ok(pair);
        }
        let send = match self.acquire(imports::INPUT_PRODUCER, minimum, maximum, 0) {
            Ok(send) => send,
            Err(Error(code)) if code == wire::UNAVAILABLE as i64 => {
                self.producer_absent = true;
                return Err(Error(code));
            }
            Err(error) => return Err(error),
        };
        let receive = match self.acquire(imports::INPUT_PRODUCER, minimum, maximum, 1) {
            Ok(receive) => receive,
            Err(error) => {
                send.close()?;
                return Err(error);
            }
        };
        self.producer = Some((send, receive));
        Ok((send, receive))
    }
    /// Acquire only the explicitly offered capture connection.
    pub fn capture(&mut self, minimum: u64, maximum: u64) -> Result<Pair, Error> {
        version(minimum, maximum)?;
        if self.capture_absent {
            return Err(Error(wire::UNAVAILABLE as i64));
        }
        if let Some(pair) = self.capture {
            return Ok(pair);
        }
        let send = match self.acquire(imports::CAPTURE, minimum, maximum, 0) {
            Ok(send) => send,
            Err(Error(code)) if code == wire::UNAVAILABLE as i64 => {
                self.capture_absent = true;
                return Err(Error(code));
            }
            Err(error) => return Err(error),
        };
        let receive = match self.acquire(imports::CAPTURE, minimum, maximum, 1) {
            Ok(receive) => receive,
            Err(error) => {
                send.close()?;
                return Err(error);
            }
        };
        self.capture = Some((send, receive));
        Ok((send, receive))
    }
}
fn version(minimum: u64, maximum: u64) -> Result<(), Error> {
    if minimum == 0 || minimum > maximum {
        return Err(Error(wire::INVALID as i64));
    }
    if !(minimum..=maximum).contains(&1) {
        return Err(Error(wire::INCOMPATIBLE as i64));
    }
    Ok(())
}
