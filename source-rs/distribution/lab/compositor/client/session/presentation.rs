//! Small distribution UI: two explicit buttons, a counter and retained drag content.
use super::*;
use cathedral_contracts::composition::{self as c, Node};
impl App {
    pub(super) fn publish(&mut self) -> Result<(), Error> {
        let mut page = Private::allocate(1)?;
        self.encode(page.bytes_mut());
        let sealed = page.seal(self.link)?;
        self.revision += 1;
        let result = self.rpc([c::SUBMIT, self.revision, sealed.handle(), 0, 0, 0]);
        let _ = sealed.release();
        let reply = result?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        Ok(())
    }
    fn encode(&self, bytes: &mut [u8]) {
        let title = match self.role {
            1 => *b"APP ONE\0",
            2 => *b"HOST\0\0\0\0",
            _ => *b"CHILD\0\0\0",
        };
        let color = match self.role {
            1 => 0xe96f6f,
            2 => 0x79c99e,
            _ => 0x779bea,
        };
        let mut count = *b"N=000\0\0\0";
        count[2] = b'0' + (self.value / 100 % 10) as u8;
        count[3] = b'0' + (self.value / 10 % 10) as u8;
        count[4] = b'0' + (self.value % 10) as u8;
        let mut nodes = [
            Node([c::ROOT, c::RECT, 0, 0, 256, 200, color]),
            Node([
                c::ROOT,
                c::TEXT,
                16,
                12,
                (title.iter().position(|&b| b == 0).unwrap_or(8) * 6) as u64,
                7,
                u64::from_le_bytes(title),
            ]),
            Node([
                c::ROOT,
                c::RECT,
                16,
                32,
                64,
                24,
                if self.selected == 0 {
                    0xffd166
                } else {
                    0x101827
                },
            ]),
            Node([
                c::ROOT,
                c::TEXT,
                24,
                40,
                18,
                7,
                u64::from_le_bytes(*b"ADD\0\0\0\0\0"),
            ]),
            Node([
                c::ROOT,
                c::RECT,
                96,
                32,
                64,
                24,
                if self.selected == 1 {
                    0xffd166
                } else {
                    0x101827
                },
            ]),
            Node([
                c::ROOT,
                c::TEXT,
                104,
                40,
                30,
                7,
                u64::from_le_bytes(*b"CLEAR\0\0\0"),
            ]),
            Node([c::ROOT, c::TEXT, 16, 68, 30, 7, u64::from_le_bytes(count)]),
            Node([c::ROOT, c::INPUT, 16, 32, 64, 24, 0]),
            Node([c::ROOT, c::INPUT, 96, 32, 64, 24, 0]),
            Node([c::ROOT, c::INPUT, 16, 96, 224, 92, 0]),
            Node([c::ROOT, c::RECT, 0, 0, 12, 12, 0x101827]),
        ];
        if let Some((x, y)) = self.marker {
            nodes[10].0[2] = x;
            nodes[10].0[3] = y;
        }
        let encoded = c::encode(
            &nodes[..if self.marker.is_some() { 11 } else { 10 }],
            &[0; c::IMAGE_PIXELS],
        )
        .unwrap();
        bytes[..encoded.len()].copy_from_slice(&encoded);
    }
}
