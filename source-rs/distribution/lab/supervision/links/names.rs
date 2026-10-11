//! Exercise real clock/link lookup across process replacement and unrelated restarts.
use super::*;
pub(super) fn inspect(pair: (Handle, Handle), retired: [u64; 2]) -> [u64; 2] {
    let mut packet = [0; 24];
    for (value, bytes) in [255, retired[0], retired[1]]
        .into_iter()
        .zip(packet.chunks_exact_mut(8))
    {
        bytes.copy_from_slice(&value.to_le_bytes());
    }
    pair.0.send(&packet).unwrap();
    let mut reply = [0; 64];
    assert_eq!(pair.1.receive(&mut reply).unwrap(), 16);
    core::array::from_fn(|i| u64::from_le_bytes(reply[i * 8..i * 8 + 8].try_into().unwrap()))
}
pub(super) fn probe(packet: &[u8]) -> Option<[u8; 16]> {
    if packet.len() != 24 || packet[..8] != 255u64.to_le_bytes() {
        return None;
    }
    let old_clock = u64::from_le_bytes(packet[8..16].try_into().unwrap());
    let old_link = u64::from_le_bytes(packet[16..24].try_into().unwrap());
    let clock = raw(abi::CLOCK_HANDLE, 0, 0, 0);
    let link = Link::at(0).unwrap().raw();
    assert!((clock as i64) > 0);
    if old_clock != 0 {
        let mut ticks = 0u64;
        assert_ne!(clock, old_clock);
        assert_ne!(link, old_link);
        assert_eq!(
            raw(abi::CLOCK_READ, old_clock, (&raw mut ticks) as u64, 8),
            abi::BAD_HANDLE
        );
        assert_eq!(raw(abi::LINK_CONNECT, old_link, 0, 0), abi::BAD_HANDLE);
    }
    let mut reply = [0; 16];
    reply[..8].copy_from_slice(&clock.to_le_bytes());
    reply[8..].copy_from_slice(&link.to_le_bytes());
    Some(reply)
}
fn raw(number: u64, first: u64, second: u64, third: u64) -> u64 {
    let result;
    // SAFETY: Integer handle probes enter checked syscalls; the only pointer is a
    // live writable local tick buffer. No user mapping is accessed directly here.
    unsafe {
        core::arch::asm!("int 0x80", inlateout("rax") number => result,
            in("rdi") first, in("rsi") second, in("rdx") third,
            options(nostack, preserves_flags));
    }
    result
}
