// Classify the two outstanding exchanges on a stock matrix interaction link.
// Presentation status is a sign-extended i32; polls use the other high words.
use cathedral_platform_contracts::matrix as wire;

/// This client reserves the two high words used by untagged status replies.
/// The public poll protocol still permits u32::MAX for serial clients.
pub const LAST_POLL: u32 = u32::MAX - 1;

#[derive(Debug, PartialEq, Eq)]
pub enum Reply {
    Poll(wire::Packet),
    Presentation(u64),
    Unmatched(u32),
    Invalid,
}

pub fn classify(words: wire::Packet, poll: Option<u32>, presenting: bool) -> Reply {
    let tag = (words[0] >> 32) as u32;
    if (1..=LAST_POLL).contains(&tag) {
        return if poll == Some(tag) {
            Reply::Poll(wire::untag_poll(words, tag).unwrap())
        } else {
            Reply::Unmatched(tag)
        };
    }
    if presenting
        && words[1..] == [0; 7]
        && (words[0] as i64) <= 0
        && words[0] == words[0] as u32 as i32 as i64 as u64
    {
        Reply::Presentation(words[0])
    } else {
        Reply::Invalid
    }
}
