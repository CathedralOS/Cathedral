// This program chooses two columns. The platform World clips and names surfaces;
// shared retained custody forwards sealed images without mapping child pixels.
use super::*;
use cathedral_user_runtime::{
    matrix::upstream::Images,
    memory::{Object, Private},
    server::Request,
};
pub struct Output {
    pub(super) services: Services,
    pub(super) grant: Grant,
    images: Images,
    cursors: upstream::Cursors,
    header: u64,
    pub(super) capacity: usize,
    pub automatic: bool,
    pub capturing: bool,
    hidden_right: bool,
    pub nested: bool,
    pub failure: Option<&'static [u8]>,
}
impl Output {
    pub fn new(services: Services) -> Self {
        Self {
            services,
            grant: Grant::default(),
            images: Images::empty(),
            cursors: upstream::Cursors::empty(),
            header: 0,
            capacity: 0,
            automatic: false,
            capturing: false,
            hidden_right: false,
            nested: false,
            failure: None,
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
        let needed = if self.nested { 10 } else { 9 };
        if grant.width < 80 || grant.height < 64 || self.capacity < needed {
            for slot in 0..2 {
                world.revoke(slot).map_err(|c| Error(c as i64))?;
            }
            return self.flush(world);
        }
        let width = if self.nested {
            grant.width
        } else {
            (grant.width - 12) / 2
        };
        for slot in 0..if self.nested { 1 } else { 2 } {
            world
                .surface_range(slot, slot * 4, if self.nested { 9 } else { 4 })
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
        let reply = self.services.request_interaction(
            [
                wire::SURFACE_CREATE,
                grant.epoch,
                needed as u64 - 1,
                0,
                0,
                0,
                0,
                0,
            ],
            None,
        )?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        self.header = reply[1];
        self.header(false)
    }
    pub fn deepen_hit(&self, x: u64, y: u64) -> bool {
        let left = self.grant.width.saturating_sub(256) + 100;
        !self.nested && self.failure.is_none() && y < 28 && (left..left + 80).contains(&x)
    }
    pub fn automatic_hit(&self, x: u64, y: u64) -> bool {
        let left = self.grant.width.saturating_sub(256) + 180;
        !self.nested && self.failure.is_none() && y < 28 && (left..left + 48).contains(&x)
    }
    pub fn can_deepen(&self) -> bool {
        self.capacity >= 10
    }
    pub fn nested_layout(&mut self, world: &mut World) -> Result<(), Error> {
        self.nested = true;
        let grant = self.grant;
        // Retire our former header; child surfaces were already retired.
        if self.header != 0 {
            self.services.request_interaction(
                [wire::ERASE, grant.epoch, self.header, 0, 0, 0, 0, 0],
                None,
            )?;
        }
        self.capacity = 0;
        self.arrange(grant, world)
    }
    pub fn flush(&mut self, world: &mut World) -> Result<(), Error> {
        while let Some(words) = world.take_retired_surface() {
            let reply = self.services.request_interaction(words, None)?;
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
        #[cfg(feature = "render-demand-probe")]
        time::storage_sample("split-publication", &[slot as u64, words[0]]);
        if words[0] == wire::SURFACE_CREATE {
            if words[3..] != [0; 5] {
                return Ok(wire::error(abi::DENIED));
            }
            let rank = match world.prepare_surface(slot, request.incarnation, words[1], words[2]) {
                Ok(r) => r,
                Err(c) => return Ok(wire::error(c)),
            };
            let reply = self.services.request_interaction(
                [wire::SURFACE_CREATE, self.grant.epoch, rank, 0, 0, 0, 0, 0],
                None,
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
        let reply = self.services.request_interaction(prepared, None)?;
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
        #[cfg(feature = "render-demand-probe")]
        time::storage_sample("split-publication", &[slot as u64, words[0]]);
        let prepared = match world.preview_surface(slot, request.incarnation, words) {
            Ok(p) => p,
            Err(c) => return Ok(wire::error(c)),
        };
        self.images.room(prepared[2])?;
        let reply = self.services.request_interaction(prepared, Some(&object))?;
        if reply[0] == 0 {
            world.surface(slot, request.incarnation, words).unwrap();
            self.images.commit(prepared[2], Some(object));
        }
        Ok(reply)
    }
}
