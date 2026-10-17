//! A deliberately bounded ASCII note. Encoding belongs to the app, not storage.
use cathedral_platform_contracts::input;
pub const CAPACITY: usize = 60;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Document {
    pub bytes: [u8; CAPACITY],
    pub len: usize,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            bytes: [0; CAPACITY],
            len: 0,
        }
    }
}
impl Document {
    pub fn edit(&mut self, cursor: &mut usize, key: u8) -> bool {
        match key {
            input::LEFT => *cursor = cursor.saturating_sub(1),
            input::RIGHT => *cursor = (*cursor + 1).min(self.len),
            input::BACKSPACE if *cursor > 0 => {
                self.bytes.copy_within(*cursor..self.len, *cursor - 1);
                self.len -= 1;
                *cursor -= 1;
                self.bytes[self.len] = 0;
            }
            _ => {
                let byte = match key {
                    input::A..=input::Z => b'a' + key - input::A,
                    input::DIGIT_0..=input::DIGIT_9 => b'0' + key - input::DIGIT_0,
                    input::SPACE => b' ',
                    input::ENTER => b'\n',
                    _ => return false,
                };
                if self.len == CAPACITY {
                    return false;
                }
                self.bytes.copy_within(*cursor..self.len, *cursor + 1);
                self.bytes[*cursor] = byte;
                *cursor += 1;
                self.len += 1;
            }
        }
        true
    }
    pub fn encode(&self) -> ([u8; 64], usize) {
        let mut bytes = [0; 64];
        bytes[..4].copy_from_slice(b"NTS1");
        bytes[4..4 + self.len].copy_from_slice(&self.bytes[..self.len]);
        (bytes, 4 + self.len)
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if !(4..=64).contains(&bytes.len())
            || &bytes[..4] != b"NTS1"
            || bytes[4..].iter().any(|b| {
                !b.is_ascii_lowercase() && !b.is_ascii_digit() && !matches!(b, b' ' | b'\n')
            })
        {
            return None;
        }
        let mut doc = Self {
            len: bytes.len() - 4,
            ..Self::default()
        };
        doc.bytes[..doc.len].copy_from_slice(&bytes[4..]);
        Some(doc)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn middle_edits_round_trip_at_record_boundary_and_capacity() {
        let mut doc = Document::default();
        let mut cursor = 0;
        for _ in 0..60 {
            assert!(doc.edit(&mut cursor, input::A));
        }
        assert!(!doc.edit(&mut cursor, input::SPACE));
        cursor = 28;
        doc.edit(&mut cursor, input::BACKSPACE);
        doc.edit(&mut cursor, input::Z);
        assert_eq!(doc.bytes[27], b'z');
        let (bytes, len) = doc.encode();
        assert_eq!(Document::decode(&bytes[..len]), Some(doc));
        assert!(Document::decode(b"NTS1bad\0").is_none());
        assert!(Document::decode(b"MC01").is_none());
    }
}
