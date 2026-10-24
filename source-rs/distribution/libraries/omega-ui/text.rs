// An app owns each field value and cursor; only the Omega editor changes them.
use crate::Error;
#[derive(Clone)]
pub struct TextInput {
    pub(crate) bytes: [u8; 128],
    pub(crate) length: usize,
    pub(crate) caret: usize,
    pub(crate) anchor: usize,
    pub(crate) scroll: i32,
}
impl Default for TextInput {
    fn default() -> Self {
        Self {
            bytes: [0; 128],
            length: 0,
            caret: 0,
            anchor: 0,
            scroll: 0,
        }
    }
}
impl TextInput {
    pub fn new(text: &str) -> Result<Self, Error> {
        let mut input = Self::default();
        input.set(text)?;
        Ok(input)
    }
    pub fn set(&mut self, text: &str) -> Result<(), Error> {
        if text.len() > 128 || !text.bytes().all(|byte| (32..=126).contains(&byte)) {
            return Err(Error::Text);
        }
        self.bytes.fill(0);
        self.bytes[..text.len()].copy_from_slice(text.as_bytes());
        self.length = text.len();
        self.caret = self.length;
        self.anchor = self.length;
        self.scroll = 0;
        Ok(())
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn as_str(&self) -> &str {
        // SAFETY: Construction and native read-back validate every byte as ASCII;
        // fields are private and the returned slice cannot mutate them.
        unsafe { core::str::from_utf8_unchecked(&self.bytes[..self.length]) }
    }
    pub(crate) fn read(packet: &[i32; 4096]) -> Result<Self, Error> {
        let length = usize::try_from(packet[4040]).map_err(|_| Error::Text)?;
        let caret = usize::try_from(packet[4041]).map_err(|_| Error::Text)?;
        let anchor = usize::try_from(packet[4042]).map_err(|_| Error::Text)?;
        if length > 128 || caret > length || anchor > length || packet[4043] < 0 {
            return Err(Error::Text);
        }
        let mut next = Self {
            length,
            caret,
            anchor,
            scroll: packet[4043],
            ..Self::default()
        };
        for (target, value) in next.bytes[..length].iter_mut().zip(&packet[..length]) {
            if !(32..=126).contains(value) {
                return Err(Error::Text);
            }
            *target = *value as u8;
        }
        Ok(next)
    }
}
