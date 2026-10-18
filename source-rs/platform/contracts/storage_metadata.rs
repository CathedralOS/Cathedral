//! Bounded hierarchy metadata codec shared by storage clients and provider.
//! Empty fields represent legacy/unknown metadata. Names are exact UTF-8 bytes;
//! no slash, control character, dot or parent component is a valid name.
pub const FIELD: usize = 32;
pub const BYTES: usize = 2 * FIELD;
pub const PATH_BYTES: usize = 256;
pub const EXECUTABLE: &[u8] = b"application/x-executable";
pub const TEXT: &[u8] = b"text/plain";
pub const MATRIX_CONFIG: &[u8] = b"application/x-matrix-config";
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    pub name: [u8; FIELD],
    pub media_type: [u8; FIELD],
}
pub fn field(bytes: &[u8]) -> Option<[u8; FIELD]> {
    if bytes.len() > FIELD
        || bytes.iter().any(|b| *b < 32 || *b == 127)
        || core::str::from_utf8(bytes).is_err()
    {
        return None;
    }
    let mut result = [0; FIELD];
    result[..bytes.len()].copy_from_slice(bytes);
    Some(result)
}
pub fn text(bytes: &[u8; FIELD]) -> &[u8] {
    &bytes[..bytes.iter().position(|b| *b == 0).unwrap_or(FIELD)]
}
impl Metadata {
    pub fn new(name: &[u8], media_type: &[u8]) -> Option<Self> {
        if name.contains(&b'/') || name == b"." || name == b".." {
            return None;
        }
        Some(Self {
            name: field(name)?,
            media_type: field(media_type)?,
        })
    }
    pub fn encode(self) -> [u8; BYTES] {
        let mut bytes = [0; BYTES];
        bytes[..FIELD].copy_from_slice(&self.name);
        bytes[FIELD..].copy_from_slice(&self.media_type);
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return Some(Self::default());
        }
        if bytes.len() != BYTES {
            return None;
        }
        let name = bytes[..FIELD].try_into().ok()?;
        let media_type = bytes[FIELD..].try_into().ok()?;
        let result = Self::new(text(&name), text(&media_type))?;
        (result.name == name && result.media_type == media_type).then_some(result)
    }
}
