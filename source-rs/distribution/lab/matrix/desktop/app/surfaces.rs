//! App-owned surface lifecycle and adversarial local-handle/allowance probes.
use super::*;
pub(super) const ACCENT: u64 = 0xe5e7eb;
pub(super) struct Scene {
    pub epoch: u64,
    base: u64,
    accent: Option<u64>,
}
impl Scene {
    pub fn new(parent: Pair, view: wire::Packet) -> Self {
        assert_eq!(
            call(parent, [wire::RUN_MEMORY, 0, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::DENIED)
        );
        let info = call(parent, [wire::SURFACE_INFO, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(info[0], 0);
        assert_eq!(info[2], 0);
        assert!((3..=wire::MAX_SURFACES as u64).contains(&info[1]));
        let mut names = [0; wire::MAX_SURFACES];
        for rank in 0..info[1] {
            let reply = call(parent, [wire::SURFACE_CREATE, view[1], rank, 0, 0, 0, 0, 0]);
            assert_eq!(reply[0], 0);
            names[rank as usize] = reply[1];
        }
        assert_eq!(
            call(parent, [wire::SURFACE_CREATE, view[1], 0, 0, 0, 0, 0, 0]),
            wire::error(abi::NO_MEMORY)
        );
        assert_eq!(
            call(parent, [wire::PAINT, view[1], 0, 0, 0, 1, 1, 0]),
            wire::error(abi::BAD_HANDLE)
        );
        for &name in &names[3..info[1] as usize] {
            assert_eq!(
                call(parent, [wire::ERASE, view[1], name, 0, 0, 0, 0, 0]),
                [0; 8]
            );
        }
        let mut scene = Self {
            epoch: view[1],
            base: names[0],
            accent: Some(names[1]),
        };
        scene.remove_accent(parent);
        scene.restore_accent(parent);
        image::submit(parent, view[1], names[2]);
        scene
    }
    pub fn paint(&self, parent: Pair, view: wire::Packet, color: u64) {
        assert_eq!(
            call(
                parent,
                [
                    wire::PAINT,
                    self.epoch,
                    self.base,
                    0,
                    0,
                    view[2] + 99,
                    view[3] + 99,
                    color
                ]
            ),
            [0; 8]
        );
        self.paint_accent(parent);
    }
    fn paint_accent(&self, parent: Pair) {
        if let Some(name) = self.accent {
            assert_eq!(
                call(
                    parent,
                    [wire::PAINT, self.epoch, name, 30, 40, 40, 18, ACCENT]
                ),
                [0; 8]
            );
        }
    }
    pub fn remove_accent(&mut self, parent: Pair) {
        if let Some(stale) = self.accent.take() {
            assert_eq!(
                call(parent, [wire::ERASE, self.epoch, stale, 0, 0, 0, 0, 0]),
                [0; 8]
            );
            assert_eq!(
                call(parent, [wire::PAINT, self.epoch, stale, 0, 0, 1, 1, 0]),
                wire::error(abi::BAD_HANDLE)
            );
        }
    }
    pub fn restore_accent(&mut self, parent: Pair) {
        assert!(self.accent.is_none());
        let reply = call(parent, [wire::SURFACE_CREATE, self.epoch, 1, 0, 0, 0, 0, 0]);
        assert_eq!(reply[0], 0);
        self.accent = Some(reply[1]);
        self.paint_accent(parent);
    }
}
