//! Experimental local shared request/reply transport. Connection authority comes
//! from supplied endpoints; these bytes neither locate nor authenticate a peer.
pub const OPEN: &[u8] = b"cathedral.shared.v1";
pub const ACK: &[u8] = b"shared.ready.v1";
pub const COMMIT: &[u8] = b"shared.commit.v1";
pub const MAX_PAYLOAD: usize = 64;
pub const FRAME_BYTES: usize = 8 + MAX_PAYLOAD;
pub fn encode(id: u64, payload: &[u8]) -> Option<([u8; FRAME_BYTES], usize)> {
    if id == 0 || payload.len() > MAX_PAYLOAD {
        return None;
    }
    let mut frame = [0; FRAME_BYTES];
    frame[..8].copy_from_slice(&id.to_le_bytes());
    frame[8..8 + payload.len()].copy_from_slice(payload);
    Some((frame, 8 + payload.len()))
}
pub fn decode(frame: &[u8]) -> Option<(u64, &[u8])> {
    if !(8..=FRAME_BYTES).contains(&frame.len()) {
        return None;
    }
    let id = u64::from_le_bytes(frame[..8].try_into().ok()?);
    (id != 0).then_some((id, &frame[8..]))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames_preserve_exact_id_and_extent() {
        for n in 0..=MAX_PAYLOAD {
            let (f, len) = encode(u64::MAX, &[7; MAX_PAYLOAD][..n]).unwrap();
            assert_eq!(decode(&f[..len]), Some((u64::MAX, &[7; MAX_PAYLOAD][..n])));
        }
        assert!(encode(0, b"x").is_none());
        assert!(encode(1, &[0; 65]).is_none());
        assert!(decode(&[0; 8]).is_none());
        assert!(decode(&[1; 7]).is_none());
        assert!(decode(&[1; 73]).is_none());
    }
}
