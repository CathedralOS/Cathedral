#![no_std]
#![forbid(unsafe_code)]
//! Lab status protocol. This reports observations, never grants provider control.
pub const STATUS: &[u8] = b"status";
pub const READY: &[u8] = b"ready";
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Status {
    pub display: u64,
    pub input: u64,
    pub application: u64,
    pub last: u64,
    pub storage: u64,
}
impl Status {
    pub fn encode(self) -> [u8; 40] {
        let mut bytes = [0; 40];
        for (word, chunk) in [
            self.display,
            self.input,
            self.application,
            self.last,
            self.storage,
        ]
        .into_iter()
        .zip(bytes.chunks_exact_mut(8))
        {
            chunk.copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 40 {
            return None;
        }
        let mut words = [0; 5];
        for (word, chunk) in words.iter_mut().zip(bytes.chunks_exact(8)) {
            *word = u64::from_le_bytes(chunk.try_into().ok()?);
        }
        if words[3] > 4 {
            return None;
        }
        Some(Self {
            display: words[0],
            input: words[1],
            application: words[2],
            last: words[3],
            storage: words[4],
        })
    }
}
