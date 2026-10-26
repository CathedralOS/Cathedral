//! Read a bounded revision-consistent directory from the supplied interaction
//! connection. BUSY means discard this attempt; the caller chooses when to retry.
//! At most INPUT_STREAMS rows and their descriptions plus one terminal read;
//! ordinary input events never require these lookup exchanges.
use crate::services::Services;
use crate::{Error, abi};
use cathedral_platform_contracts::input::description::Description;
use cathedral_platform_contracts::matrix as wire;
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Channel {
    pub name: u32,
    pub group: u16,
    pub retiring: bool,
    /// Receiver view-wide key ceiling, not this source's physical capabilities.
    pub keys: u64,
    pub description: Description,
}
pub struct Directory {
    pub epoch: u64,
    pub revision: u64,
    entries: [Channel; wire::INPUT_STREAMS],
    len: usize,
}
impl Directory {
    pub fn entries(&self) -> &[Channel] {
        &self.entries[..self.len]
    }
    pub fn read(services: &mut Services, epoch: u64) -> Result<Self, Error> {
        let mut snapshot = Self {
            epoch,
            revision: 0,
            entries: [Channel::default(); wire::INPUT_STREAMS],
            len: 0,
        };
        let mut after = 0;
        for _ in 0..=wire::INPUT_STREAMS {
            let reply = services.request_interaction(
                [
                    wire::INPUT_DIRECTORY,
                    epoch,
                    snapshot.revision,
                    after,
                    0,
                    0,
                    0,
                    0,
                ],
                None,
            )?;
            if reply[0] != 0 {
                return Err(Error(reply[0] as i64));
            }
            if reply[1] != epoch
                || reply[2] == 0
                || (snapshot.revision != 0 && reply[2] != snapshot.revision)
                || reply[7] != 0
            {
                return Err(Error(abi::IO_ERROR as i64));
            }
            snapshot.revision = reply[2];
            if reply[3..] == [0; 5] {
                return Ok(snapshot);
            }
            if reply[3] <= after
                || reply[3] > u32::MAX as u64
                || !(1..=0x7fff).contains(&reply[4])
                || !matches!(reply[5], wire::CHANNEL_LIVE | wire::CHANNEL_RETIRING)
                || reply[6] & !wire::VALID_KEY_MASK != 0
                || snapshot.len == wire::INPUT_STREAMS
            {
                return Err(Error(abi::IO_ERROR as i64));
            }
            snapshot.entries[snapshot.len] = Channel {
                name: reply[3] as u32,
                group: reply[4] as u16,
                retiring: reply[5] == wire::CHANNEL_RETIRING,
                keys: reply[6],
                description: Self::describe(services, epoch, snapshot.revision, reply[3])?,
            };
            snapshot.len += 1;
            after = reply[3];
        }
        Err(Error(abi::IO_ERROR as i64))
    }
    fn describe(
        services: &mut Services,
        epoch: u64,
        revision: u64,
        name: u64,
    ) -> Result<Description, Error> {
        let reply = services.request_interaction(
            [wire::INPUT_DESCRIPTION, epoch, revision, name, 0, 0, 0, 0],
            None,
        )?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        if reply[1..4] != [epoch, revision, name] || reply[7] != 0 {
            return Err(Error(abi::IO_ERROR as i64));
        }
        Description::from_words([reply[4], reply[5], reply[6]]).ok_or(Error(abi::IO_ERROR as i64))
    }
}
