//! Synchronous private-catalog transport. The accepted link selects the root.
//! One outstanding request; errors never automatically retry mutations.
use crate::{Error, ipc::Handle, link::Link, time};
use cathedral_contracts::storage as wire;
pub struct Client {
    link: Link,
    pair: Option<(Handle, Handle)>,
}
impl Client {
    pub fn at(index: u64) -> Result<Self, Error> {
        Ok(Self {
            link: Link::at(index)?,
            pair: None,
        })
    }
    pub fn pair(&mut self) -> Result<(Handle, Handle), Error> {
        if let Some(pair) = self.pair {
            return Ok(pair);
        }
        let pair = self.link.connect()?;
        self.pair = Some(pair);
        Ok(pair)
    }
    pub fn call(
        &mut self,
        op: u64,
        object: u64,
        generation: u64,
        payload: &[u8],
    ) -> Result<wire::Record, Error> {
        let request = wire::request(op, object, generation, payload)
            .ok_or(Error(cathedral_contracts::user::INVALID_ARGUMENT as i64))?;
        let (send, receive) = self.pair()?;
        let result = (|| {
            send.send(&request)?;
            let mut reply = [0; 64];
            let length = receive.receive_until(&mut reply, time::after(100)?)?;
            Ok((reply, length))
        })();
        match result {
            Ok((reply, length)) => {
                wire::Record::decode(&reply[..length]).map_err(|error| Error(error as i64))
            }
            Err(error) => {
                self.pair = None;
                Err(error)
            }
        }
    }
}
