//! This program chooses two columns. The platform World clips and names surfaces;
//! shared retained custody forwards sealed images without mapping child pixels.
use super::*;
use cathedral_user_runtime::{
    matrix::upstream::Images,
    memory::{Object, Private},
    server::Request,
};
pub struct Output {
    pair: Pair,
    grant: Grant,
    images: Images,
    header: u64,
    capacity: usize,
}
impl Output {
    pub fn new(pair: Pair) -> Self {
        Self {
            pair,
            grant: Grant::default(),
            images: Images::empty(),
            header: 0,
            capacity: 0,
        }
    }
    pub fn arrange(&mut self, grant: Grant, world: &mut World) -> Result<(), Error> {
        if self.grant == grant && self.capacity == world.backed_surfaces() {
            return Ok(());
        }
        self.grant = grant;
        self.capacity = world.backed_surfaces();
        self.header = 0;
        self.flush(world)?;
        if !grant.valid() {
            return Ok(());
        }
        if grant.width < 80 || grant.height < 64 || self.capacity < 9 {
            for slot in 0..2 {
                world.revoke(slot).map_err(|c| Error(c as i64))?;
            }
            return self.flush(world);
        }
        let width = (grant.width - 12) / 2;
        for slot in 0..2 {
            world
                .surface_range(slot, slot * 4, 4)
                .map_err(|c| Error(c as i64))?;
            world
                .place(
                    slot,
                    Rect {
                        x: slot as u64 * (width + 12),
                        y: 32,
                        width,
                        height: grant.height - 32,
                    },
                    grant.keys,
                )
                .map_err(|c| Error(c as i64))?;
        }
        let reply = upstream::call(
            self.pair,
            [wire::SURFACE_CREATE, grant.epoch, 8, 0, 0, 0, 0, 0],
        )?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        self.header = reply[1];
        self.header(false)
    }
    pub fn header(&mut self, dirty: bool) -> Result<(), Error> {
        if self.header == 0 {
            return Ok(());
        }
        let width = (self.grant.width as usize).min(256);
        let mut pixels = Private::allocate((width * 28 * 4).div_ceil(4096))?;
        for p in pixels.bytes_mut()[..width * 28 * 4].chunks_exact_mut(4) {
            p.copy_from_slice(&0x202d43u32.to_le_bytes());
        }
        for (left, text) in [
            (
                8,
                if dirty {
                    b"UNSAVED NOTES".as_slice()
                } else {
                    b"SPLIT"
                },
            ),
            (width.saturating_sub(20), b"X"),
        ] {
            for (i, ch) in text.iter().enumerate() {
                for (y, bits) in cathedral_bitmap_font::glyph(*ch).into_iter().enumerate() {
                    for x in 0..5 {
                        if bits & (1 << (4 - x)) != 0 {
                            for dy in 0..2 {
                                for dx in 0..2 {
                                    let px = left + i * 12 + x * 2 + dx;
                                    let py = 6 + y * 2 + dy;
                                    if px < width {
                                        let at = (py * width + px) * 4;
                                        pixels.bytes_mut()[at..at + 4]
                                            .copy_from_slice(&0xffffffu32.to_le_bytes());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut object = pixels.freeze()?;
        object.unmap()?;
        let reply = upstream::exchange(
            self.pair,
            [
                wire::SURFACE_IMAGE,
                self.grant.epoch,
                self.header,
                self.grant.width - width as u64,
                0,
                width as u64,
                28,
                width as u64,
            ],
            Some(&object),
        )?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        Ok(())
    }
    pub fn flush(&mut self, world: &mut World) -> Result<(), Error> {
        while let Some(words) = world.take_retired_surface() {
            let reply = upstream::call(self.pair, words)?;
            if !matches!(reply[0], 0 | abi::DENIED | abi::BAD_HANDLE) {
                return Err(Error(reply[0] as i64));
            }
            self.images.commit(words[2], None);
        }
        Ok(())
    }
    pub fn surface(
        &mut self,
        world: &mut World,
        request: &Request,
        words: wire::Packet,
    ) -> Result<wire::Packet, Error> {
        let slot = request.client.unwrap();
        if words[0] == wire::SURFACE_CREATE {
            if words[3..] != [0; 5] {
                return Ok(wire::error(abi::DENIED));
            }
            let rank = match world.prepare_surface(slot, request.incarnation, words[1], words[2]) {
                Ok(r) => r,
                Err(c) => return Ok(wire::error(c)),
            };
            let reply = upstream::call(
                self.pair,
                [wire::SURFACE_CREATE, self.grant.epoch, rank, 0, 0, 0, 0, 0],
            )?;
            if reply[0] != 0 {
                return Ok(wire::error(reply[0]));
            }
            return Ok([
                0,
                world.create_surface(slot, words[2] as usize, reply[1]),
                0,
                0,
                0,
                0,
                0,
                0,
            ]);
        }
        let prepared = match world.preview_surface(slot, request.incarnation, words) {
            Ok(p) => p,
            Err(c) => return Ok(wire::error(c)),
        };
        let reply = upstream::call(self.pair, prepared)?;
        if reply[0] == 0 {
            world.surface(slot, request.incarnation, words).unwrap();
            self.images.commit(prepared[2], None);
        }
        Ok(reply)
    }
    pub fn image(
        &mut self,
        world: &mut World,
        request: &Request,
        object: Object,
    ) -> Result<wire::Packet, Error> {
        let Some(words) = wire::decode(&request.bytes[..request.len]) else {
            return Ok(wire::error(abi::INVALID_ARGUMENT));
        };
        if request.control || words[0] != wire::SURFACE_IMAGE {
            return Ok(wire::error(abi::DENIED));
        }
        if !cathedral_platform_contracts::display::composition::valid_image(
            words[5],
            words[6],
            words[7],
            object.len(),
        ) {
            return Ok(wire::error(abi::INVALID_ARGUMENT));
        }
        let slot = request.client.unwrap();
        let prepared = match world.preview_surface(slot, request.incarnation, words) {
            Ok(p) => p,
            Err(c) => return Ok(wire::error(c)),
        };
        self.images.room(prepared[2])?;
        let reply = upstream::exchange(self.pair, prepared, Some(&object))?;
        if reply[0] == 0 {
            world.surface(slot, request.incarnation, words).unwrap();
            self.images.commit(prepared[2], Some(object));
        }
        Ok(reply)
    }
}
