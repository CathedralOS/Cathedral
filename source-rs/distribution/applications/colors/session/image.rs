//! One immutable bitmap: submit once, drop the producer handle, let matrices replay.
use super::*;
use cathedral_user_runtime::memory::{Object, Private};
fn send(parent: Pair, words: wire::Packet, object: &Object) -> wire::Packet {
    parent.0.send_memory(&wire::encode(words), object).unwrap();
    let mut bytes = [0; 64];
    let len = parent.1.receive(&mut bytes).unwrap();
    wire::decode(&bytes[..len]).unwrap()
}
pub(super) fn probe(parent: Pair, epoch: u64, surface: u64) -> Result<(), u64> {
    let mut pixels = Private::allocate(1).unwrap();
    for (index, pixel) in pixels.bytes_mut().chunks_exact_mut(4).enumerate() {
        let color: u32 = if ((index % 32 / 8) ^ (index / 32 / 8)) & 1 == 0 {
            0xffcc44
        } else {
            0x7755dd
        };
        pixel.copy_from_slice(&color.to_le_bytes());
    }
    let mut object = pixels.freeze().unwrap();
    object.unmap().unwrap();
    let image = [wire::SURFACE_IMAGE, epoch, surface, 180, 110, 32, 32, 32];
    // Aborted staging must release its reader. Then forward the same sealed
    // backing through a complete recursive transaction without copying pixels.
    assert_eq!(
        call(parent, [wire::PRESENT_BEGIN, epoch, 1, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(send(parent, image, &object), [0; 8]);
    assert_eq!(
        call(parent, [wire::PRESENT_ABORT, epoch, 1, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(
        call(parent, [wire::PRESENT_BEGIN, epoch, 2, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(send(parent, image, &object), [0; 8]);
    assert_eq!(
        call(parent, [wire::PRESENT_COMMIT, epoch, 2, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(call(parent, image), wire::error(abi::DENIED));
    // Rejected replacement cannot disturb retained geometry or the existing image.
    let mut invalid = image;
    invalid[7] = 1;
    assert_eq!(
        send(parent, invalid, &object),
        wire::error(abi::INVALID_ARGUMENT)
    );
    invalid = image;
    invalid[2] = 0;
    assert_eq!(send(parent, invalid, &object), wire::error(abi::BAD_HANDLE));
    // Forwarders now own the read rights; the producing app keeps no mapping/handle.
    Ok(())
}

/// Ordinary presentation has no adversarial probes. A resized binding returns
/// to the app loop so it can acquire fresh surfaces without losing state.
pub(super) fn submit(parent: Pair, epoch: u64, surface: u64) -> Result<(), u64> {
    let mut pixels = Private::allocate(1).unwrap();
    for (index, pixel) in pixels.bytes_mut().chunks_exact_mut(4).enumerate() {
        let color: u32 = if ((index % 32 / 8) ^ (index / 32 / 8)) & 1 == 0 {
            0xffcc44
        } else {
            0x7755dd
        };
        pixel.copy_from_slice(&color.to_le_bytes());
    }
    let mut object = pixels.freeze().unwrap();
    object.unmap().unwrap();
    let reply = send(
        parent,
        [wire::SURFACE_IMAGE, epoch, surface, 180, 110, 32, 32, 32],
        &object,
    );
    if reply[0] == 0 { Ok(()) } else { Err(reply[0]) }
}
