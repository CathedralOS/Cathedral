//! Experimental retained-scene snapshots. Client identity comes from IPC, never a field.
//! SUBMIT [op, revision, sealed page, 0, 0, 0] copies a bounded immutable snapshot;
//! completion means accepted and drawn, not a vblank/presentation guarantee.
//! Control-only: BACKGROUND [op,rgb,0,0,0,0], PLACE [op,client,x,y,w,h],
//! PIXEL [op,x,y,0,0,0] (lab readback), STATUS [op,client,0,0,0,0].
//! STATUS replies [status, incarnation, revision, visible, 0, 0].
pub const MODE: u64 = 0x434f4d50;
pub const SUBMIT: u64 = 16;
pub const BACKGROUND: u64 = 17;
pub const PLACE: u64 = 18;
pub const PIXEL: u64 = 19;
pub const STATUS: u64 = 20;
pub const MAX_NODES: usize = 8;
pub const IMAGE_SIDE: usize = 16;
pub const IMAGE_PIXELS: usize = IMAGE_SIDE * IMAGE_SIDE;
pub const SNAPSHOT_BYTES: usize = 16 + MAX_NODES * 56 + IMAGE_PIXELS * 4;
pub const MAGIC: u64 = 0x5343454e450001;
pub const ROOT: u64 = u64::MAX;
pub const GROUP: u64 = 0;
pub const RECT: u64 = 1;
pub const TEXT: u64 = 2;
pub const IMAGE: u64 = 3;
/// Parent index (earlier GROUP or ROOT), kind, local x/y, width/height, payload.
/// RECT payload is RGB; TEXT is up to eight ASCII bytes, zero padded, scale one;
/// IMAGE uses the snapshot's one 16x16 RGB asset and requires matching dimensions.
#[derive(Clone, Copy, Debug, Default)]
pub struct Node(pub [u64; 7]);
pub fn encode(nodes: &[Node], pixels: &[u32; IMAGE_PIXELS]) -> Option<[u8; SNAPSHOT_BYTES]> {
    if nodes.len() > MAX_NODES {
        return None;
    }
    let mut out = [0; SNAPSHOT_BYTES];
    out[..8].copy_from_slice(&MAGIC.to_le_bytes());
    out[8..16].copy_from_slice(&(nodes.len() as u64).to_le_bytes());
    for (node, bytes) in nodes
        .iter()
        .zip(out[16..16 + MAX_NODES * 56].chunks_exact_mut(56))
    {
        for (word, dst) in node.0.iter().zip(bytes.chunks_exact_mut(8)) {
            dst.copy_from_slice(&word.to_le_bytes());
        }
    }
    for (pixel, dst) in pixels
        .iter()
        .zip(out[16 + MAX_NODES * 56..].chunks_exact_mut(4))
    {
        dst.copy_from_slice(&pixel.to_le_bytes());
    }
    Some(out)
}
