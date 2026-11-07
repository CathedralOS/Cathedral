//! Replace the ancestor view during a held scene, then explicitly re-place its
//! children after settlement. Regrant drawing allowances before placement:
//! a new ancestor epoch never restores either permission implicitly.
use super::children::Children;
use super::*;
use cathedral_user_runtime::time;

pub(super) struct Regrant {
    slot: usize,
    incarnation: u64,
    view: wire::Packet,
    until: u64,
    stage: u8,
    asked: bool,
}
impl Regrant {
    pub fn new(world: &mut World, slot: usize, incarnation: u64, held_until: u64) -> Self {
        let view = world.request(slot, incarnation, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(view[0], 0);
        assert!(view[2] > 32 && view[3] > 104);
        let surfaces = world.request(slot, incarnation, [wire::SURFACE_INFO, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(surfaces[0], 0);
        assert!(surfaces[1] >= 13);
        assert_eq!(
            world.owner_command([wire::REVOKE, slot as u64, 0, 0, 0, 0, 0, 0]),
            [0; 8]
        );
        world
            .surface_limit(slot, surfaces[1] as usize)
            .expect("restore the ancestor's drawing allowance");
        assert_eq!(
            world.owner_command([wire::PLACE, slot as u64, 0, 0, view[2], view[3], view[4], 0]),
            [0; 8]
        );
        let after = world.request(slot, incarnation, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(after[0], 0);
        assert_ne!(after[1], view[1]);
        assert_eq!(after[2..5], view[2..5]);
        world.notify(slot);
        time::storage_sample("layout-view-regranted", &[view[1], after[1]]);
        Self {
            slot,
            incarnation,
            view,
            until: held_until.wrapping_add(300),
            stage: 0,
            asked: false,
        }
    }
    pub fn step(
        &mut self,
        server: &Server,
        children: &mut Children,
        now: u64,
        held: bool,
    ) -> Result<bool, Error> {
        assert_eq!(server.incarnation(self.slot), Some(self.incarnation));
        assert!(
            !time::reached(now, self.until),
            "view restoration timed out"
        );
        if held {
            return Ok(false);
        }
        let (width, height, keys) = (self.view[2], self.view[3], self.view[4]);
        let words = match self.stage {
            0 => [wire::SURFACE_LIMIT, 4, 1, 0, 0, 0, 0, 0],
            1 => [wire::SURFACE_LIMIT, 1, 4, 0, 0, 0, 0, 0],
            2 => [wire::SURFACE_LIMIT, 0, 8, 0, 0, 0, 0, 0],
            3 => [wire::PLACE, 4, 0, 0, width, height, 0, 0],
            4 => [wire::PLACE, 1, 8, 8, width - 16, height - 104, keys, 0],
            5 => [wire::PLACE, 0, 8, height - 88, width - 16, 80, keys, 0],
            _ => return Ok(true),
        };
        let Some(reply) = children.poll_control_probe(
            self.slot,
            (!self.asked).then_some(words),
            now,
            self.until,
        )?
        else {
            self.asked = true;
            return Ok(false);
        };
        assert_eq!(reply, [0; 8], "explicit child regrant refused: {words:?}");
        self.asked = false;
        self.stage += 1;
        if self.stage == 6 {
            time::storage_sample("layout-view-restored", &[]);
            return Ok(true);
        }
        Ok(false)
    }
}
