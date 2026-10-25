//! Experimental local shared request/reply transport. Connection authority comes
//! from supplied endpoints; these bytes neither locate nor authenticate a peer.
pub const OPEN: &[u8] = b"cathedral.shared.v2";
/// Opt in to bounded independent requests and consumption notifications.
pub const OPEN_PIPELINE: &[u8] = b"cathedral.shared.pipeline.v2";
pub const ACK: &[u8] = b"shared.ready.v2";
pub const COMMIT: &[u8] = b"shared.commit.v2";
/// Bounded requests retained by one service connection, independent of mailbox capacity.
pub const MAX_IN_FLIGHT: usize = 4;
pub const MAX_PAYLOAD: usize = 64;
pub const FRAME_BYTES: usize = 16 + MAX_PAYLOAD;
/// A typed grant carries only this correlation ID, never the application payload.
pub fn grant_id(bytes: &[u8]) -> Option<u64> {
    let id = u64::from_le_bytes(bytes.try_into().ok()?);
    (id != 0).then_some(id)
}
pub fn encode(id: u64, payload: &[u8], memory: bool) -> Option<([u8; FRAME_BYTES], usize)> {
    if id == 0 || payload.len() > MAX_PAYLOAD {
        return None;
    }
    let mut frame = [0; FRAME_BYTES];
    frame[..8].copy_from_slice(&id.to_le_bytes());
    frame[8..16].copy_from_slice(&u64::from(memory).to_le_bytes());
    frame[16..16 + payload.len()].copy_from_slice(payload);
    Some((frame, 16 + payload.len()))
}
pub fn decode(frame: &[u8]) -> Option<(u64, &[u8], bool)> {
    if !(16..=FRAME_BYTES).contains(&frame.len()) {
        return None;
    }
    let id = u64::from_le_bytes(frame[..8].try_into().ok()?);
    let memory = u64::from_le_bytes(frame[8..16].try_into().ok()?);
    (id != 0 && memory <= 1).then_some((id, &frame[16..], memory == 1))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames_bind_exact_id_extent_and_attachment_kind() {
        for memory in [false, true] {
            for n in 0..=MAX_PAYLOAD {
                let (f, len) = encode(u64::MAX, &[7; MAX_PAYLOAD][..n], memory).unwrap();
                assert_eq!(
                    decode(&f[..len]),
                    Some((u64::MAX, &[7; MAX_PAYLOAD][..n], memory))
                );
            }
        }
        assert!(encode(0, b"x", false).is_none());
        assert!(encode(1, &[0; 65], false).is_none());
        assert!(decode(&[0; 16]).is_none());
        assert!(decode(&[1; 15]).is_none());
        assert!(decode(&[1; 81]).is_none());
        let (mut frame, n) = encode(1, b"x", false).unwrap();
        frame[8] = 2;
        assert!(decode(&frame[..n]).is_none());
        assert_eq!(grant_id(&1u64.to_le_bytes()), Some(1));
        assert_eq!(grant_id(&[0; 8]), None);
        assert_eq!(grant_id(&[1; 9]), None);
    }
}
