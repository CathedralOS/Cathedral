//! Chrome-local running instances. Tickets are scoped to this matrix, never PIDs.
use super::*;
pub const TOP: u64 = 384;
pub const ROW: u64 = 32;
#[derive(Default)]
pub struct Tasks {
    pub entries: [wire::Packet; 3],
    next_refresh: u64,
}
impl Tasks {
    pub fn refresh(&mut self, parent: Pair, force: bool) -> bool {
        let now = time::now().unwrap();
        if !force && !time::reached(now, self.next_refresh) {
            return false;
        }
        self.next_refresh = now.wrapping_add(10);
        let old = self.entries;
        for (i, entry) in self.entries.iter_mut().enumerate() {
            *entry = call(parent, [wire::APP_ENTRY, i as u64, 0, 0, 0, 0, 0, 0]);
        }
        old != self.entries
    }
    pub fn click(
        &self,
        parent: Pair,
        view: wire::Packet,
        event: wire::Packet,
        pressed: bool,
    ) -> Option<&'static str> {
        if event[2] != wire::POINTER
            || event[5] & 1 == 0
            || pressed
            || !(TOP..TOP + 3 * ROW).contains(&event[4])
        {
            return None;
        }
        let entry = self.entries[((event[4] - TOP) / ROW) as usize];
        if entry[0] != 0 || entry[1] == 0 {
            return Some("");
        }
        let reply = call(parent, [wire::SELECT_APP, entry[1], 0, 0, 0, 0, 0, 0]);
        if reply[0] != 0 {
            return Some("TRY AGAIN");
        }
        let left = view[2].saturating_sub(180) / 2;
        if event[3] >= left + 156 {
            let reply = call(parent, [wire::CLOSE, 0, 0, 0, 0, 0, 0, 0]);
            return Some(if reply[0] == 0 { "" } else { "UNSAVED WORK" });
        }
        Some("")
    }
    pub fn paint(&self, realm: &realm::Realm) -> cathedral_user_runtime::memory::Private {
        let mut pixels = cathedral_user_runtime::memory::Private::allocate(17).unwrap();
        for (i, entry) in self.entries.iter().enumerate() {
            let color: u32 = if entry[0] == 0 && entry[1] != 0 {
                if entry[3] != 0 { 0x31506c } else { tiles::CARD }
            } else {
                tiles::BACKGROUND
            };
            for pixel in
                pixels.bytes_mut()[i * 180 * 32 * 4..(i + 1) * 180 * 32 * 4].chunks_exact_mut(4)
            {
                pixel.copy_from_slice(&color.to_le_bytes());
            }
            if entry[0] == 0 && entry[1] != 0 {
                let label = realm
                    .ids
                    .iter()
                    .position(|id| *id == entry[2])
                    .map_or(b"APP".as_slice(), |index| &realm.labels[index]);
                tiles::label(pixels.bytes_mut(), 180, 4, i * 32 + 8, label);
                tiles::label(pixels.bytes_mut(), 180, 136, i * 32 + 8, &[b'1' + i as u8]);
                tiles::label(pixels.bytes_mut(), 180, 160, i * 32 + 8, b"X");
            }
        }
        pixels
    }
}
