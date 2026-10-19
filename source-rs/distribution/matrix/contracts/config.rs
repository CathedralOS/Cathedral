//! The stock matrix's bounded persisted arrangement, not a universal manifest.
//! Paths resolve in its supplied realm. Editing this data does not apply it live.
use cathedral_platform_contracts::storage::metadata::{field, text};
pub const NAME: &[u8] = b"Matrix";
pub const SIZE: usize = 104;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub launcher: [u8; 32],
    pub launcher_pages: u64,
    pub app_pages: u64,
    pub app_readers: u64,
    pub margin: u64,
    pub launcher_width: u64,
    pub gap: u64,
    pub app_surfaces: u64,
    pub keys: u64,
}
impl Config {
    pub fn encode(self) -> [u8; SIZE] {
        let mut bytes = [0; SIZE];
        bytes[..8].copy_from_slice(b"CTHMAT01");
        bytes[8..40].copy_from_slice(&self.launcher);
        for (slot, word) in bytes[40..].chunks_exact_mut(8).zip([
            self.launcher_pages,
            self.app_pages,
            self.app_readers,
            self.margin,
            self.launcher_width,
            self.gap,
            self.app_surfaces,
            self.keys,
        ]) {
            slot.copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE || &bytes[..8] != b"CTHMAT01" {
            return None;
        }
        let launcher: [u8; 32] = bytes[8..40].try_into().ok()?;
        if field(text(&launcher))? != launcher || text(&launcher).is_empty() {
            return None;
        }
        let words: [u64; 8] = core::array::from_fn(|i| {
            u64::from_le_bytes(bytes[40 + i * 8..48 + i * 8].try_into().unwrap())
        });
        let result = Self {
            launcher,
            launcher_pages: words[0],
            app_pages: words[1],
            app_readers: words[2],
            margin: words[3],
            launcher_width: words[4],
            gap: words[5],
            app_surfaces: words[6],
            keys: words[7],
        };
        if result.launcher_pages > 128
            || result.app_pages > 1024
            || result.app_readers > 1536
            || result.margin > 4096
            || result.launcher_width > 4096
            || result.gap > 4096
            || result.app_surfaces > crate::matrix::MAX_SURFACES as u64
            || result.keys & !crate::matrix::VALID_KEY_MASK != 0
        {
            return None;
        }
        Some(result)
    }
    pub fn rectangles(self, width: u64, height: u64) -> Option<[[u64; 4]; 2]> {
        rectangles(self.margin, self.launcher_width, self.gap, width, height)
    }
}
/// Checked geometry shared by saved startup and explicitly authorized live layout.
pub fn rectangles(
    margin: u64,
    launcher_width: u64,
    gap: u64,
    width: u64,
    height: u64,
) -> Option<[[u64; 4]; 2]> {
    let h = height.checked_sub(margin.checked_mul(2)?)?;
    let x = margin.checked_add(launcher_width)?.checked_add(gap)?;
    let w = width.checked_sub(x.checked_add(margin)?)?;
    if h == 0 || w == 0 || launcher_width == 0 {
        return None;
    }
    Some([[margin, margin, launcher_width, h], [x, margin, w, h]])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_configuration_cannot_wrap_layout_or_expand_key_mask() {
        let mut c = Config {
            launcher: field(b"./Desktop").unwrap(),
            launcher_pages: 48,
            app_pages: 32,
            app_readers: 4,
            margin: 24,
            launcher_width: 192,
            gap: 16,
            app_surfaces: 6,
            keys: crate::matrix::VALID_KEY_MASK,
        };
        assert_eq!(Config::decode(&c.encode()), Some(c));
        assert_eq!(
            c.rectangles(1024, 768),
            Some([[24, 24, 192, 720], [232, 24, 768, 720]])
        );
        assert!(c.rectangles(200, 40).is_none());
        assert!(Config::decode(&c.encode()[..SIZE - 1]).is_none());
        c.keys = u64::MAX;
        assert!(Config::decode(&c.encode()).is_none());
        c.margin = u64::MAX;
        assert!(c.rectangles(1024, 768).is_none());
    }
}
