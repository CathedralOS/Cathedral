//! Present only visible choices, then a short message when an action fails.
use super::*;
use cathedral_user_runtime::memory::Private;
pub struct Scene {
    pub epoch: u64,
    names: [u64; ENTRIES + 2],
    message: usize,
}
impl Scene {
    pub fn new(parent: Pair, epoch: u64, picker: bool) -> Result<Self, u64> {
        let mut names = [0; ENTRIES + 2];
        for (rank, name) in names
            .iter_mut()
            .take(if picker {
                USER_ENTRIES + 2
            } else {
                ENTRIES + 2
            })
            .enumerate()
        {
            let reply = call(
                parent,
                [wire::SURFACE_CREATE, epoch, rank as u64, 0, 0, 0, 0, 0],
            );
            accepted(reply[0])?;
            *name = reply[1];
        }
        Ok(Self {
            epoch,
            names,
            message: if picker { USER_ENTRIES + 1 } else { ENTRIES },
        })
    }
    pub fn paint(
        &self,
        parent: Pair,
        extent: wire::Packet,
        realm: &realm::Realm,
        selected: usize,
        status: &str,
        managed: bool,
    ) -> Result<(), u64> {
        let picker = realm.user_picker;
        self.background(parent, extent)?;
        let (items, n) = layout::entries(realm.count, picker);
        for (i, &index) in items[..n].iter().enumerate() {
            let title: &[u8] = if index == NEW {
                b"NEW USER"
            } else {
                &realm.labels[index]
            };
            let pixels = tiles::paint(index, picker, selected == index, title);
            let (x, y) = layout::origin(extent, realm.count, picker, i);
            let (width, height) = layout::dimensions(picker);
            self.image(
                parent,
                self.names[i + 1],
                pixels,
                [x, y, width as u64, height as u64],
            )?;
        }
        for i in n..self.message - 1 {
            accepted(
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
                        tiles::BACKGROUND as u64,
                    ],
                )[0],
            )?;
        }
        self.footer(
            parent,
            extent,
            status,
            if picker { "" } else { "SETTINGS" },
            picker,
            managed,
        )
    }
    pub fn tasks(
        &self,
        parent: Pair,
        extent: wire::Packet,
        tasks: &tasks::Tasks,
        realm: &realm::Realm,
    ) -> Result<(), u64> {
        self.image(
            parent,
            self.names[ENTRIES + 1],
            tasks.paint(realm),
            [
                layout::tasks_left(realm.count),
                tasks::top(extent),
                tasks::WIDTH as u64,
                tasks::ROW,
            ],
        )
    }
    pub fn management(
        &self,
        parent: Pair,
        extent: wire::Packet,
        controls: &management::Controls,
    ) -> Result<(), u64> {
        self.background(parent, extent)?;
        for index in (4..ENTRIES).chain(core::iter::once(ENTRIES + 1)) {
            if self.names[index] == 0 {
                continue;
            }
            accepted(
                call(
                    parent,
                    [wire::PAINT, self.epoch, self.names[index], 0, 0, 0, 0, 0],
                )[0],
            )?;
        }
        for row in 0..3 {
            let (x, y) = management::view::origin(extent, row);
            self.image(
                parent,
                self.names[row + 1],
                management::view::paint(controls, row),
                [
                    x,
                    y,
                    management::view::WIDTH as u64,
                    management::view::height(row) as u64,
                ],
            )?;
        }
        self.footer(parent, extent, controls.status, "", false, false)
    }
    pub fn settings(
        &self,
        parent: Pair,
        extent: wire::Packet,
        editor: &settings::Editor,
    ) -> Result<(), u64> {
        self.background(parent, extent)?;
        for index in (4..ENTRIES).chain(core::iter::once(ENTRIES + 1)) {
            if self.names[index] == 0 {
                continue;
            }
            accepted(
                call(
                    parent,
                    [wire::PAINT, self.epoch, self.names[index], 0, 0, 0, 0, 0],
                )[0],
            )?;
        }
        for row in 0..3 {
            let (x, y) = settings::view::origin(extent, row);
            self.image(
                parent,
                self.names[row + 1],
                settings::view::paint(editor, row),
                [
                    x,
                    y,
                    settings::view::WIDTH as u64,
                    settings::view::HEIGHT as u64,
                ],
            )?;
        }
        self.footer(parent, extent, editor.status, "", false, false)
    }
    fn background(&self, parent: Pair, extent: wire::Packet) -> Result<(), u64> {
        accepted(
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
                    tiles::BACKGROUND as u64,
                ],
            )[0],
        )?;
        // Release the previous message before allocating replacement tiles.
        accepted(
            call(
                parent,
                [
                    wire::PAINT,
                    self.epoch,
                    self.names[self.message],
                    0,
                    0,
                    1,
                    1,
                    tiles::BACKGROUND as u64,
                ],
            )[0],
        )?;
        Ok(())
    }
    fn footer(
        &self,
        parent: Pair,
        extent: wire::Packet,
        status: &str,
        button: &str,
        picker: bool,
        managed: bool,
    ) -> Result<(), u64> {
        if !status.is_empty() || !button.is_empty() || managed {
            let mut pixels = Private::allocate(if managed { 12 } else { 8 }).unwrap();
            for pixel in pixels.bytes_mut().chunks_exact_mut(4) {
                pixel.copy_from_slice(&tiles::BACKGROUND.to_le_bytes());
            }
            let offset = if managed { 24 } else { 0 };
            if managed {
                tiles::label(pixels.bytes_mut(), 180, 48, 0, b"END APP");
            }
            tiles::label(pixels.bytes_mut(), 180, 0, offset, status.as_bytes());
            tiles::label(pixels.bytes_mut(), 180, 42, offset + 24, button.as_bytes());
            let x = if picker {
                extent[2].saturating_sub(180) / 2
            } else {
                extent[2].saturating_sub(184)
            };
            let y = if picker {
                extent[3] / 2 + 88
            } else {
                extent[3].saturating_sub(48 + offset as u64)
            };
            self.image(
                parent,
                self.names[self.message],
                pixels,
                [x, y, 180, 40 + offset as u64],
            )?;
        }
        Ok(())
    }
    fn image(
        &self,
        parent: Pair,
        surface: u64,
        pixels: Private,
        [x, y, width, height]: [u64; 4],
    ) -> Result<(), u64> {
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
        let n = cathedral_user_runtime::matrix::upstream::receive(
            parent.1,
            &mut bytes,
            wire::SURFACE_IMAGE,
            300,
        )
        .unwrap();
        accepted(wire::decode(&bytes[..n]).unwrap()[0])
    }
}

fn accepted(status: u64) -> Result<(), u64> {
    if status == 0 { Ok(()) } else { Err(status) }
}
