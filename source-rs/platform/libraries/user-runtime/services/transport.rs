//! One selected transport owns acquisition and interaction correlation. Optional
//! service discovery remains lazy; an upgraded binding never falls back to copies.
use super::*;
use crate::{connection::Client, memory::Object};
enum Acquired {
    Endpoint(crate::connection::Endpoint),
    Notification(crate::connection::NotificationGrant),
}

impl Services {
    /// Select shared transport before using this supplied broker. Repeated
    /// selection preserves its request sequence and any previously acquired rights.
    pub fn enable_shared(&mut self) -> Result<(), Error> {
        if self.failed {
            return Err(Error(abi::PEER_CLOSED as i64));
        }
        if self.shared.is_none() {
            match Client::connect(self.broker, time::after(300)?) {
                Ok(client) => self.shared = Some(client),
                Err(error) => {
                    self.failed = true;
                    return Err(error);
                }
            }
        }
        Ok(())
    }
    /// The acquired interaction and broker share this connection. Never construct
    /// a new client from its raw pair or reset IDs between the two operation kinds.
    pub fn request_interaction(
        &mut self,
        words: matrix::Packet,
        image: Option<&Object>,
    ) -> Result<matrix::Packet, Error> {
        if self.failed {
            return Err(Error(abi::PEER_CLOSED as i64));
        }
        if self.interaction.is_none() {
            return Err(Error(abi::DENIED as i64));
        }
        let result = (|| {
            if let Some(client) = &mut self.shared {
                let (bytes, n, object) =
                    client.call_retained_memory(&matrix::encode(words), image)?;
                if object.is_some() {
                    return Err(Error(abi::IO_ERROR as i64));
                }
                matrix::decode(&bytes[..n]).ok_or(Error(abi::IO_ERROR as i64))
            } else {
                crate::matrix::upstream::exchange(self.broker, words, image)
            }
        })();
        if let Err(error) = &result {
            time::storage_sample("matrix-exchange-failed", &[words[0], error.0 as u64]);
            self.retire_binding();
        }
        result
    }
    fn retire_binding(&mut self) {
        self.failed = true;
        if let Some(client) = &mut self.shared {
            client.retire();
        } else {
            let _ = self.broker.0.close();
            let _ = self.broker.1.close();
        }
    }
    pub(super) fn acquire_wake(&mut self, minimum: u64, maximum: u64) -> Result<Wake, Error> {
        match self.acquire_right(imports::INTERACTION, minimum, maximum, 0, true)? {
            Acquired::Notification(right) => Ok(Wake::Notification(right.into_handle())),
            Acquired::Endpoint(_) => unreachable!("typed acquisition"),
        }
    }
    pub(super) fn acquire(
        &mut self,
        contract: u64,
        minimum: u64,
        maximum: u64,
        part: u64,
    ) -> Result<Handle, Error> {
        match self.acquire_right(contract, minimum, maximum, part, false)? {
            Acquired::Endpoint(right) => Ok(right.into_handle()),
            Acquired::Notification(_) => unreachable!("typed acquisition"),
        }
    }
    fn acquire_right(
        &mut self,
        contract: u64,
        minimum: u64,
        maximum: u64,
        part: u64,
        notification: bool,
    ) -> Result<Acquired, Error> {
        if self.failed {
            return Err(Error(abi::PEER_CLOSED as i64));
        }
        let mut refused = false;
        let result = (|| {
            let (bytes, len, endpoint) = if let Some(client) = &mut self.shared {
                let request = matrix::encode(wire::request(contract, minimum, maximum, part));
                if notification {
                    let (bytes, len, right) = client.call_retained_notification(&request)?;
                    (bytes, len, right.map(Acquired::Notification))
                } else {
                    let (bytes, len, right) = client.call_retained_endpoint(&request)?;
                    (bytes, len, right.map(Acquired::Endpoint))
                }
            } else {
                self.broker.0.send(&matrix::encode(wire::request(
                    contract, minimum, maximum, part,
                )))?;
                match ipc::wait_many(&[self.broker.1], time::after(300)?) {
                    Err(Error(code)) if code == abi::TIMED_OUT as i64 => {
                        // This request still owns the reply stream. A late endpoint
                        // belongs to this acquisition, not a new request or process.
                        time::storage_sample("service-acquire-delayed", &[contract, part]);
                    }
                    Err(error) => return Err(error),
                    Ok(_) => (),
                }
                let mut bytes = [0; 64];
                if notification {
                    let (len, right) =
                        crate::notification::Notification::accept(self.broker.1, &mut bytes)?;
                    (
                        bytes,
                        len,
                        right
                            .map(crate::connection::NotificationGrant::received)
                            .map(Acquired::Notification),
                    )
                } else {
                    let (len, right) = self.broker.1.receive_with(&mut bytes)?;
                    (
                        bytes,
                        len,
                        right
                            .map(crate::connection::Endpoint::received)
                            .map(Acquired::Endpoint),
                    )
                }
            };
            match (matrix::decode(&bytes[..len]), endpoint) {
                (Some([0, 1, 0, 0, 0, 0, 0, 0]), Some(endpoint)) => Ok(endpoint),
                (Some([code, 0, 0, 0, 0, 0, 0, 0]), None) if code != 0 => {
                    refused = true;
                    Err(Error(code as i64))
                }
                (_, endpoint) => {
                    drop(endpoint);
                    Err(Error(abi::IO_ERROR as i64))
                }
            }
        })();
        // A late reply cannot be mistaken for a later acquisition. Terminal/ambiguous
        // transport failure poisons this binding rather than silently retrying it.
        if result.is_err() && !refused {
            self.retire_binding();
        }
        result
    }
}
