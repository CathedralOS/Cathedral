//! A bounded duplex mailbox over two independently funded single-writer buffers.
//! Try operations never park: drain/check, then wait on an explicitly held
//! notification. The peer owns all input; copied bytes still require app validation.
use crate::{
    Error, abi,
    memory::shared::{Reader, Writer},
};
pub const MAX_BYTES: usize = 256;
const WORDS: usize = 3 + MAX_BYTES / 8;
pub struct Mailbox {
    output: Writer,
    input: Reader,
    sent: u64,
    received: u64,
}
impl Mailbox {
    /// Requires fresh, otherwise unused buffers. Check our own headers; the peer
    /// may already have published its first message. This does not authenticate
    /// a session or prove the history of a caller-supplied peer buffer.
    pub fn new(output: Writer, input: Reader) -> Result<Self, Error> {
        if output.words() < WORDS
            || input.words() < WORDS
            || output.load(0)? != 0
            || output.load(1)? != 0
        {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        Ok(Self {
            output,
            input,
            sent: 0,
            received: 0,
        })
    }
    pub fn try_send(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if bytes.len() > MAX_BYTES {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        self.send_ready()?;
        let next = self
            .sent
            .checked_add(1)
            .ok_or(Error(abi::IO_ERROR as i64))?;
        for (i, chunk) in bytes.chunks(8).enumerate() {
            let mut word = [0; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            self.output.store(3 + i, u64::from_le_bytes(word))?;
        }
        self.output.store(2, bytes.len() as u64)?;
        self.output.store(0, next)?; // Release publication follows complete payload.
        self.sent = next;
        Ok(())
    }
    /// Check backpressure before transferring an attachment. The sole writer
    /// owns `sent`; a compliant peer can only advance its acknowledgement.
    pub(crate) fn send_ready(&self) -> Result<(), Error> {
        let acknowledged = self.input.load(1)?;
        if acknowledged > self.sent {
            return Err(Error(abi::IO_ERROR as i64));
        }
        if acknowledged != self.sent {
            return Err(Error(abi::WOULD_BLOCK as i64));
        }
        Ok(())
    }
    /// Insufficient output space does not acknowledge or consume the message.
    /// A hostile peer can lie about sequence/content, but cannot select copy bounds.
    pub fn try_receive(&mut self, bytes: &mut [u8]) -> Result<Option<usize>, Error> {
        let sequence = self.input.load(0)?;
        if sequence == self.received {
            return Ok(None);
        }
        if self.received.checked_add(1) != Some(sequence) {
            return Err(Error(abi::IO_ERROR as i64));
        }
        let length = self.input.load(2)?;
        if length > MAX_BYTES as u64 {
            return Err(Error(abi::IO_ERROR as i64));
        }
        let length = length as usize;
        if length > bytes.len() {
            return Err(Error(abi::TOO_SMALL as i64));
        }
        // Copy into caller-owned storage, never expose a shared slice. The length
        // above is a local value and is never reread for indexing.
        for (i, chunk) in bytes[..length].chunks_mut(8).enumerate() {
            let word = self.input.load(3 + i)?.to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }
        if self.input.load(0)? != sequence {
            return Err(Error(abi::IO_ERROR as i64));
        }
        self.received = sequence;
        self.output.store(1, sequence)?; // Release ack only after reads have completed.
        Ok(Some(length))
    }
}
