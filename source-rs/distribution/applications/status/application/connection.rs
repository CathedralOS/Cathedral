//! Init status client: distribution/init/session/supervision/control.rs.
//! Peer indices follow distribution/profile.json and session-protocol::link.
//! Connections and observed health; no provider lifecycle authority lives here.
use cathedral_contracts::user as abi;
use cathedral_session_protocol as session;
use cathedral_user_runtime::{Error, ipc::Handle, link::Link, time};
pub struct Connections {
    control: (Handle, Handle),
    pub display: Option<(Handle, Handle)>,
    pub input: Option<(Handle, Handle)>,
    pub storage: Option<(Handle, Handle)>,
    pub status: session::Status,
}
impl Connections {
    pub fn open() -> Result<Self, Error> {
        Ok(Self {
            control: (Handle::bootstrap(0)?, Handle::bootstrap(1)?),
            display: None,
            input: None,
            storage: None,
            status: session::Status::default(),
        })
    }
    pub fn request(&self, request: &[u8]) -> Result<([u8; 64], usize), Error> {
        self.control.1.send(request)?;
        let mut bytes = [0; 64];
        let length = self
            .control
            .0
            .receive_until(&mut bytes, time::after(500)?)?;
        Ok((bytes, length))
    }
    pub fn refresh(&mut self) -> Result<bool, Error> {
        let (bytes, length) = self.request(session::STATUS)?;
        let status =
            session::Status::decode(&bytes[..length]).ok_or(Error(abi::IO_ERROR as i64))?;
        if status.display != self.status.display {
            self.display = None;
        }
        if status.input != self.status.input {
            self.input = None;
        }
        if status.storage != self.status.storage {
            self.storage = None;
        }
        let changed = status != self.status;
        self.status = status;
        Ok(changed)
    }
    pub fn pair(&mut self, index: u64) -> Result<(Handle, Handle), Error> {
        if let Some(pair) = match index {
            session::link::DISPLAY => self.display,
            session::link::INPUT => self.input,
            session::link::STORAGE => self.storage,
            _ => return Err(Error(abi::DENIED as i64)),
        } {
            return Ok(pair);
        }
        for _ in 0..3 {
            match Link::at(index)?.connect() {
                Ok(pair) => {
                    if index == session::link::DISPLAY {
                        self.display = Some(pair);
                    } else if index == session::link::INPUT {
                        self.input = Some(pair);
                    } else {
                        self.storage = Some(pair);
                    }
                    return Ok(pair);
                }
                Err(_) => {
                    self.refresh()?;
                }
            }
        }
        Err(Error(abi::PEER_CLOSED as i64))
    }
}
