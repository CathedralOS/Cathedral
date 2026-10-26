//! One retained shared root connection per admitted desktop or lab session.
use crate::{Error, Pair, abi, time, wire};
use cathedral_user_runtime::{connection::Client, event};

pub(crate) struct RootControl {
    client: Client,
}

impl RootControl {
    /// The boot owner serves no requests needed by the new root. Its display
    /// peer is already running, so this bounded setup can park here.
    /// Takes endpoint custody even on failure.
    pub fn open(pair: Pair, deadline: u64) -> Result<Self, Error> {
        let mut control = Self {
            client: Client::connect(pair, deadline)?,
        };
        let reply = control.exchange([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0], deadline)?;
        if reply != [0; 8] {
            return Err(Error(abi::IO_ERROR as i64));
        }
        cathedral_user_runtime::write(b"Cathedral desktop: shared root control ready\n")?;
        Ok(control)
    }
    pub fn exchange(&mut self, words: wire::Packet, deadline: u64) -> Result<wire::Packet, Error> {
        let (bytes, n) = self.client.call(&wire::encode(words), deadline)?;
        wire::decode(&bytes[..n]).ok_or(Error(abi::IO_ERROR as i64))
    }
    pub fn idle(&self, deadline: u64) -> Result<usize, Error> {
        assert!(!self.client.pending());
        // No reply is outstanding. Watch the lifetime/grant endpoint only;
        // notification hints never mean unsolicited application work arrived.
        // The next request drains any coalesced hint through the shared client.
        event::wait(&self.client.interests()?[..1], deadline)
    }
}

pub(crate) fn request(matrix: &mut RootControl, words: wire::Packet) -> wire::Packet {
    let result = matrix.exchange(words, deadline(words));
    result.unwrap_or_else(|error| {
        let root = crate::ROOT_TASK.load(core::sync::atomic::Ordering::Relaxed);
        panic!(
            "matrix request {words:?}: {error:?}; root {:?}",
            cathedral_user_runtime::task::Child::from_raw(root).wait_until(time::now().unwrap())
        )
    })
}
fn deadline(words: wire::Packet) -> u64 {
    time::after(match words[0] & 0xffff {
        wire::CONFIGURE | wire::RESOLVE_PATH => wire::CONFIGURE_DEADLINE_TICKS + 100,
        wire::RUN_OBJECT => 3 * wire::STORAGE_DEADLINE_TICKS + 200,
        // Storage may own the root's parent stream while a picker starts.
        wire::HEALTH => wire::STORAGE_DEADLINE_TICKS + 200,
        _ => 300,
    })
    .unwrap()
}
