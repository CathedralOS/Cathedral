//! One shared setup/request stream per provider incarnation. F5 polls opening;
//! synchronous lab admission waits on the same owner's current interests.
use super::*;
use cathedral_user_runtime::connection::{Client, Connecting};
#[cfg(not(feature = "shell"))]
use cathedral_user_runtime::event;

pub(super) enum Control {
    Opening(Connecting<1>),
    Ready(Client),
}
impl Control {
    /// Takes endpoint custody even on failure. Setup alone is not readiness.
    pub fn open(pair: Pair, deadline: u64) -> Result<Self, Error> {
        Client::begin_connect(pair, deadline).map(Self::Opening)
    }
    pub fn poll(&mut self) -> Result<Option<([u8; 64], usize)>, Error> {
        match self {
            Self::Opening(opening) => {
                if let Some(mut client) = opening.poll()? {
                    client.begin(storage::HEALTH, None)?;
                    *self = Self::Ready(client);
                    write(b"Cathedral desktop: shared storage control established\n")?;
                }
                Ok(None)
            }
            Self::Ready(client) => match client.poll()? {
                Some((bytes, n, None)) => Ok(Some((bytes, n))),
                Some(_) => Err(Error(abi::IO_ERROR as i64)),
                None => Ok(None),
            },
        }
    }
    #[cfg(not(feature = "shell"))]
    pub fn wait(&self, until: u64) -> Result<(), Error> {
        match self {
            Self::Opening(opening) => event::wait(&[opening.interest()?], until)?,
            Self::Ready(client) => event::wait(&client.interests()?, until)?,
        };
        Ok(())
    }
    pub fn health(&mut self) -> Result<(), Error> {
        match self {
            Self::Opening(_) => Err(Error(abi::BUSY as i64)),
            Self::Ready(client) => client.begin(storage::HEALTH, None),
        }
    }
    #[cfg(any(
        not(feature = "shell"),
        feature = "notes-probes",
        feature = "storage-live-probes"
    ))]
    pub fn command(&mut self, bytes: &[u8]) -> Result<([u8; 64], usize), Error> {
        match self {
            Self::Opening(_) => Err(Error(abi::BUSY as i64)),
            Self::Ready(client) => client.call(bytes, time::after(300)?),
        }
    }
}
