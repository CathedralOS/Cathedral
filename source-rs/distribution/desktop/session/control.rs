//! One root connection per admitted session. Ordinary F5 borrows the retained
//! shared client; positional labs explicitly keep their copied protocol.
use crate::{Error, Pair, abi, time, wire};
#[cfg(feature = "shell")]
use cathedral_user_runtime::{connection::Client, event};

#[cfg(feature = "shell")]
pub(crate) struct RootControl {
    client: Client,
}
#[cfg(not(feature = "shell"))]
pub(crate) type RootControl = Pair;

#[cfg(feature = "shell")]
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
    #[cfg(feature = "shell")]
    let result = matrix.exchange(words, deadline(words));
    #[cfg(not(feature = "shell"))]
    let result = copied_exchange(*matrix, words);
    result.unwrap_or_else(|error| {
        let root = crate::ROOT_TASK.load(core::sync::atomic::Ordering::Relaxed);
        panic!(
            "matrix request {words:?}: {error:?}; root {:?}",
            cathedral_user_runtime::task::Child::from_raw(root).wait_until(time::now().unwrap())
        )
    })
}
#[cfg(not(feature = "shell"))]
pub(crate) fn copied_call(mut pair: Pair, words: wire::Packet) -> wire::Packet {
    request(&mut pair, words)
}
#[cfg(not(feature = "shell"))]
fn copied_exchange(pair: Pair, words: wire::Packet) -> Result<wire::Packet, Error> {
    let mut bytes = [0; 64];
    pair.0.send(&wire::encode(words))?;
    let n = pair.1.receive_until(&mut bytes, deadline(words))?;
    wire::decode(&bytes[..n]).ok_or(Error(abi::IO_ERROR as i64))
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
