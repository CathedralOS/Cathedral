//! Bounded virtual-input hints. Labels are untrusted display text, never identity
//! credentials or authority. Unknown remains distinct from an empty device.
//! Controls are advisory categories, not an enforced filter on later events.
//! Each description is immutable for its live channel; replace the channel to
//! change it. This lab encoding is not a complete HID/control vocabulary.
pub const POINTER: u64 = 1;
pub const KEYS: u64 = 2;
pub const LABEL_BYTES: usize = 16;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Description {
    controls: u64,
    label: [u8; LABEL_BYTES],
}
impl Description {
    pub const UNKNOWN: Self = Self {
        controls: 0,
        label: [0; LABEL_BYTES],
    };
    pub fn new(controls: u64, label: &str) -> Option<Self> {
        if !(1..=3).contains(&controls)
            || label.is_empty()
            || label.len() > LABEL_BYTES
            || label.chars().any(char::is_control)
        {
            return None;
        }
        let mut result = Self::UNKNOWN;
        result.controls = controls;
        result.label[..label.len()].copy_from_slice(label.as_bytes());
        Some(result)
    }
    pub fn from_words([a, b, controls]: [u64; 3]) -> Option<Self> {
        if [a, b, controls] == [0; 3] {
            return Some(Self::UNKNOWN);
        }
        let mut bytes = [0; LABEL_BYTES];
        bytes[..8].copy_from_slice(&a.to_le_bytes());
        bytes[8..].copy_from_slice(&b.to_le_bytes());
        let len = bytes.iter().position(|b| *b == 0).unwrap_or(LABEL_BYTES);
        if bytes[len..].iter().any(|b| *b != 0) {
            return None;
        }
        Self::new(controls, core::str::from_utf8(&bytes[..len]).ok()?)
    }
    pub fn words(self) -> [u64; 3] {
        [
            u64::from_le_bytes(self.label[..8].try_into().unwrap()),
            u64::from_le_bytes(self.label[8..].try_into().unwrap()),
            self.controls,
        ]
    }
    pub fn label(&self) -> &str {
        let len = self
            .label
            .iter()
            .position(|b| *b == 0)
            .unwrap_or(LABEL_BYTES);
        core::str::from_utf8(&self.label[..len]).unwrap()
    }
    pub fn controls(self) -> u64 {
        self.controls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_utf8_canonical_padding_and_control_hints() {
        for label in ["Pointer", "Clavier é", "1234567890123456"] {
            let d = Description::new(POINTER | KEYS, label).unwrap();
            assert_eq!(Description::from_words(d.words()), Some(d));
            assert_eq!(d.label(), label);
        }
        assert_eq!(Description::from_words([0; 3]), Some(Description::UNKNOWN));
        for label in ["", "12345678901234567", "hello\n", "a\0b"] {
            assert!(Description::new(POINTER, label).is_none());
        }
        assert!(Description::new(4, "Unknown").is_none());
        assert!(Description::from_words([0xff, 0, POINTER]).is_none());
        assert!(Description::from_words([0x610062, 0, POINTER]).is_none());
    }
}
