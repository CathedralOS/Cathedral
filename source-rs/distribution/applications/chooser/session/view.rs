//! Present only visible choices, then a short message when an action fails.
use super::*;
use cathedral_user_runtime::memory::Private;
pub struct Scene {
    pub epoch: u64,
    names: [u64; 5],
}
impl Scene {
    pub fn new(parent: Pair, epoch: u64) -> Self {
        let mut names = [0; 5];
        for (rank, name) in names.iter_mut().enumerate() {
            let reply = call(
                parent,
                [wire::SURFACE_CREATE, epoch, rank as u64, 0, 0, 0, 0, 0],
            );
            assert_eq!(reply[0], 0);
            *name = reply[1];
        }
        Self { epoch, names }
    }
    pub fn paint(
        &self,
        parent: Pair,
        extent: wire::Packet,
        realm: &realm::Realm,
        selected: usize,
        status: &str,
        picker: bool,
    ) {
        assert_eq!(
            call(
                parent,
                [
                    wire::PAINT,
                    self.epoch,
                    self.names[0],
                    0,
                    0,
                    extent[2],
                    extent[3],
                    tiles::BACKGROUND as u64
                ]
            )[0],
            0
        );
        // Release the previous message before allocating replacement tiles.
        assert_eq!(
            call(
                parent,
                [
                    wire::PAINT,
                    self.epoch,
                    self.names[4],
                    0,
                    0,
                    1,
                    1,
                    tiles::BACKGROUND as u64
                ]
            )[0],
            0
        );
        let (items, n) = layout::entries(realm.count, picker);
        for (i, &index) in items[..n].iter().enumerate() {
            let title: &[u8] = if index == NEW {
                b"NEW USER"
            } else {
                &realm.labels[index]
            };
            let pixels = tiles::paint(index, picker, selected == index, title);
            let (x, y) = layout::origin(extent, n, picker, i);
            self.image(
                parent,
                self.names[i + 1],
                pixels,
                [x, y, layout::WIDTH as u64, layout::HEIGHT as u64],
            );
        }
        for i in n..ENTRIES - 1 {
            assert_eq!(
                call(
                    parent,
                    [
                        wire::PAINT,
                        self.epoch,
                        self.names[i + 1],
                        0,
                        0,
                        1,
                        1,
                        tiles::BACKGROUND as u64
                    ]
                )[0],
                0
            );
        }
        if !status.is_empty() {
            let mut pixels = Private::allocate(3).unwrap();
            for pixel in pixels.bytes_mut().chunks_exact_mut(4) {
                pixel.copy_from_slice(&tiles::BACKGROUND.to_le_bytes());
            }
            tiles::label(pixels.bytes_mut(), 180, 0, 0, status.as_bytes());
            let x = extent[2].saturating_sub(180) / 2;
            let y = if picker {
                extent[3] / 2 + 88
            } else {
                extent[3].saturating_sub(32)
            };
            self.image(parent, self.names[4], pixels, [x, y, 180, 16]);
        }
    }
    fn image(&self, parent: Pair, surface: u64, pixels: Private, [x, y, width, height]: [u64; 4]) {
        let mut image = pixels.freeze().unwrap();
        image.unmap().unwrap();
        parent
            .0
            .send_memory(
                &wire::encode([
                    wire::SURFACE_IMAGE,
                    self.epoch,
                    surface,
                    x,
                    y,
                    width,
                    height,
                    width,
                ]),
                &image,
            )
            .unwrap();
        let mut bytes = [0; 64];
        let n = parent
            .1
            .receive_until(&mut bytes, time::after(300).unwrap())
            .unwrap();
        assert_eq!(wire::decode(&bytes[..n]).unwrap()[0], 0);
    }
}
