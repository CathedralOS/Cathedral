//! Chrome-local running instances. Tickets are scoped to this matrix, never PIDs.
use super::*;
pub const CELL: usize = 144;
pub const WIDTH: usize = CELL * 3;
pub const ROW: u64 = 32;
pub fn top(view: wire::Packet) -> u64 {
    view[3].saturating_sub(ROW) / 2
}
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
        clicked: bool,
        left: u64,
    ) -> Option<&'static str> {
        if event[2] != wire::POINTER
            || !clicked
            || !(top(view)..top(view) + ROW).contains(&event[4])
            || !(left..left + WIDTH as u64).contains(&event[3])
        {
            return None;
        }
        let index = ((event[3] - left) / CELL as u64) as usize;
        let entry = self.entries[index];
        if entry[0] != 0 || entry[1] == 0 {
            return Some("");
        }
        let reply = call(parent, [wire::SELECT_APP, entry[1], 0, 0, 0, 0, 0, 0]);
        if reply[0] != 0 {
            return Some("TRY AGAIN");
        }
        let left = left + index as u64 * CELL as u64;
        if event[3] >= left + 124 {
            let reply = call(parent, [wire::CLOSE, 0, 0, 0, 0, 0, 0, 0]);
            return Some(if reply[0] == 0 { "" } else { "UNSAVED WORK" });
        }
        Some("")
    }
    pub fn paint(&self, realm: &realm::Realm) -> cathedral_user_runtime::memory::Private {
        let mut pixels = cathedral_user_runtime::memory::Private::allocate(
            (WIDTH * ROW as usize * 4).div_ceil(4096),
        )
        .unwrap();
        for (i, entry) in self.entries.iter().enumerate() {
            let color: u32 = if entry[0] == 0 && entry[1] != 0 {
                if entry[3] != 0 { 0x31506c } else { tiles::CARD }
            } else {
                tiles::BACKGROUND
            };
            for y in 0..ROW as usize {
                for x in i * CELL..(i + 1) * CELL {
                    let offset = (y * WIDTH + x) * 4;
                    pixels.bytes_mut()[offset..offset + 4].copy_from_slice(&color.to_le_bytes());
                }
            }
            if entry[0] == 0 && entry[1] != 0 {
                let label = realm
                    .ids
                    .iter()
                    .position(|id| *id == entry[2])
                    .map_or(b"APP".as_slice(), |index| &realm.labels[index]);
                tiles::label(
                    pixels.bytes_mut(),
                    WIDTH,
                    i * CELL + 4,
                    8,
                    &label[..label.len().min(8)],
                );
                tiles::label(
                    pixels.bytes_mut(),
                    WIDTH,
                    i * CELL + 108,
                    8,
                    &[b'1' + i as u8],
                );
                tiles::label(pixels.bytes_mut(), WIDTH, i * CELL + 128, 8, b"X");
            }
        }
        pixels
    }
}
