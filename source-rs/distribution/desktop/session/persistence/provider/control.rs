//! One setup/request stream per provider incarnation. F5 yields during opening;
//! positional fault labs explicitly retain their unsolicited copied messages.
use super::*;
#[cfg(feature = "shell")]
use cathedral_user_runtime::connection::{Client, Connecting};

pub(super) enum Control {
    #[cfg(feature = "shell")]
    Opening(Connecting<1>),
    #[cfg(feature = "shell")]
    Ready(Client),
    #[cfg(not(feature = "shell"))]
    Copied(Pair),
}
impl Control {
    /// Takes endpoint custody on success and failure. The first request is
    /// HEALTH; setup completion alone never establishes provider readiness.
    pub fn open(pair: Pair, _deadline: u64) -> Result<Self, Error> {
        #[cfg(feature = "shell")]
        {
            Client::begin_connect(pair, _deadline).map(Self::Opening)
        }
        #[cfg(not(feature = "shell"))]
        {
            let control = Self::Copied(pair);
            pair.0.send(storage::HEALTH)?;
            Ok(control)
        }
    }
    pub fn poll(&mut self, _until: u64) -> Result<Option<([u8; 64], usize)>, Error> {
        match self {
            #[cfg(feature = "shell")]
            Self::Opening(opening) => {
                if let Some(mut client) = opening.poll()? {
                    client.begin(storage::HEALTH, None)?;
                    *self = Self::Ready(client);
                    write(b"Cathedral desktop: shared storage control established\n")?;
                }
                Ok(None)
            }
            #[cfg(feature = "shell")]
            Self::Ready(client) => match client.poll()? {
                Some((bytes, n, None)) => Ok(Some((bytes, n))),
                Some(_) => Err(Error(abi::IO_ERROR as i64)),
                None => Ok(None),
            },
            #[cfg(not(feature = "shell"))]
            Self::Copied(pair) => {
                let mut bytes = [0; 64];
                match pair.1.receive_until(&mut bytes, _until) {
                    Ok(n) => Ok(Some((bytes, n))),
                    Err(Error(c)) if c == abi::TIMED_OUT as i64 => Ok(None),
                    Err(error) => Err(error),
                }
            }
        }
    }
    pub fn health(&mut self) -> Result<(), Error> {
        match self {
            #[cfg(feature = "shell")]
            Self::Opening(_) => Err(Error(abi::BUSY as i64)),
            #[cfg(feature = "shell")]
            Self::Ready(client) => client.begin(storage::HEALTH, None),
            #[cfg(not(feature = "shell"))]
            Self::Copied(pair) => pair.0.send(storage::HEALTH),
        }
    }
    #[cfg(any(feature = "notes-probes", feature = "storage-live-probes"))]
    pub fn command(&mut self, bytes: &[u8]) -> Result<([u8; 64], usize), Error> {
        let until = time::after(300)?;
        match self {
            #[cfg(feature = "shell")]
            Self::Opening(_) => Err(Error(abi::BUSY as i64)),
            #[cfg(feature = "shell")]
            Self::Ready(client) => client.call(bytes, until),
            #[cfg(not(feature = "shell"))]
            Self::Copied(pair) => {
                pair.0.send(bytes)?;
                let mut reply = [0; 64];
                let n = pair.1.receive_until(&mut reply, until)?;
                Ok((reply, n))
            }
        }
    }
}

// Shared states drop their own typed custody. Copied fixtures lend only raw aliases.
impl Drop for Control {
    fn drop(&mut self) {
        #[cfg(not(feature = "shell"))]
        {
            let Self::Copied(pair) = self;
            pair.0.close().ok();
            pair.1.close().ok();
        }
    }
}
