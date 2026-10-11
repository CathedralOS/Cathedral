//! Distribution appearance. The platform knows only scene nodes and connection scopes.
use cathedral_contracts::composition::{self as w, Node};
pub fn snapshot(role: u64, invalid: bool) -> [u8; w::SNAPSHOT_BYTES] {
    interactive(role, invalid, None)
}
pub fn interactive(
    role: u64,
    invalid: bool,
    marker: Option<(u64, u64)>,
) -> [u8; w::SNAPSHOT_BYTES] {
    let title = if role == 1 {
        *b"APP ONE\0"
    } else {
        *b"APP TWO\0"
    };
    let color = if role == 1 { 0xe96f6f } else { 0x79c99e };
    let mut nodes = [
        Node([w::ROOT, w::RECT, 0, 0, 256, 256, color]),
        Node([w::ROOT, w::TEXT, 16, 12, 42, 7, u64::from_le_bytes(title)]),
        Node([w::ROOT, w::GROUP, 32, 48, 80, 64, 0]),
        Node([2, w::GROUP, 16, 8, 96, 96, 0]),
        Node([3, w::RECT, 0, 0, 96, 96, 0x779bea]),
        Node([3, w::IMAGE, 8, 8, 16, 16, 0]),
        Node([w::ROOT, w::RECT, 0, 0, 12, 12, 0x101827]),
    ];
    if invalid {
        nodes[5].0[2] = u64::MAX;
    }
    if let Some((x, y)) = marker {
        nodes[6].0[2] = x;
        nodes[6].0[3] = y;
    }
    w::encode(
        &nodes[..if marker.is_some() { 7 } else { 6 }],
        &[0x59d9cc; w::IMAGE_PIXELS],
    )
    .unwrap()
}
