//! App-owned surface lifecycle and adversarial local-handle/allowance probes.
use super::*;
pub(super) const ACCENT: u64 = 0xe5e7eb;
pub(crate) struct Scene {
    pub epoch: u64,
    base: u64,
    accent: Option<u64>,
    revision: u64,
    #[cfg(feature = "shell")]
    close: u64,
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
        let retained = if cfg!(feature = "shell") { 4 } else { 3 };
        assert!(info[1] >= retained as u64);
        for &name in &names[retained..info[1] as usize] {
            assert_eq!(
                call(parent, [wire::ERASE, view[1], name, 0, 0, 0, 0, 0]),
                [0; 8]
            );
        }
        let mut scene = Self {
            epoch: view[1],
            base: names[0],
            accent: Some(names[1]),
            revision: 2,
            #[cfg(feature = "shell")]
            close: names[3],
        };
        scene.remove_accent(parent);
        scene.restore_accent(parent);
        image::submit(parent, view[1], names[2]);
        scene
    }
    #[cfg(feature = "shell")]
    pub fn close_button(&self, parent: Pair, view: wire::Packet, confirm: bool) {
        closing::paint(parent, view, self.close, confirm);
    }
    pub fn paint(&mut self, parent: Pair, view: wire::Packet, color: u64, error: u64) {
        self.revision += 1;
        let base = [
            wire::PAINT,
            self.epoch,
            self.base,
            0,
            0,
            view[2] + 99,
            view[3] + 99,
            color,
        ];
        let accent = [
            wire::PAINT,
            self.epoch,
            self.accent.unwrap_or(0),
            30,
            40,
            40,
            18,
            if error == 0 { ACCENT } else { 0xe96f6f },
        ];
        let updates = [base, accent];
        let packet = wire::rectangles(
            self.epoch,
            self.revision,
            &updates[..if self.accent.is_some() { 2 } else { 1 }],
        )
        .unwrap();
        assert_eq!(call(parent, packet), [0; 8]);
    }

    #[cfg(not(feature = "shell"))]
    pub fn compact_probe(&mut self, parent: Pair) {
        self.revision += 1;
        let valid = [wire::PAINT, self.epoch, self.base, 0, 0, 200, 130, 0];
        let foreign = [wire::PAINT, self.epoch, u64::MAX, 0, 0, 1, 1, 0];
        let packet = wire::rectangles(self.epoch, self.revision, &[valid, foreign]).unwrap();
        assert_eq!(call(parent, packet), wire::error(abi::BAD_HANDLE));
    }
    #[cfg(not(feature = "shell"))]
    fn begin(&mut self, parent: Pair) {
        self.revision += 1;
        assert_eq!(
            call(
                parent,
                [
                    wire::PRESENT_BEGIN,
                    self.epoch,
                    self.revision,
                    0,
                    0,
                    0,
                    0,
                    0
                ]
            ),
            [0; 8]
        );
    }
    #[cfg(not(feature = "shell"))]
    pub fn commit(&self, parent: Pair) {
        {
            assert_eq!(
                call(
                    parent,
                    [
                        wire::PRESENT_COMMIT,
                        self.epoch,
                        self.revision,
                        0,
                        0,
                        0,
                        0,
                        0
                    ]
                ),
                [0; 8]
            );
        }
    }
    #[cfg(not(feature = "shell"))]
    pub fn staged_probe(&mut self, parent: Pair) {
        self.begin(parent);
        assert_eq!(
            call(
                parent,
                [wire::PAINT, self.epoch, self.base, 0, 0, 200, 130, 0x884422]
            ),
            [0; 8]
        );
        assert_eq!(
            call(
                parent,
                [
                    wire::PAINT,
                    self.epoch,
                    self.accent.unwrap(),
                    30,
                    40,
                    40,
                    18,
                    0x224488
                ]
            ),
            [0; 8]
        );
    }
    #[cfg(not(feature = "shell"))]
    pub fn rejected_probe(&mut self, parent: Pair) {
        self.begin(parent);
        assert_eq!(
            call(
                parent,
                [wire::PAINT, self.epoch, self.base, 0, 0, 200, 130, 0]
            ),
            [0; 8]
        );
        assert_eq!(
            call(parent, [wire::ERASE, self.epoch, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::BAD_HANDLE)
        );
        assert_eq!(
            call(
                parent,
                [
                    wire::PRESENT_COMMIT,
                    self.epoch,
                    self.revision,
                    0,
                    0,
                    0,
                    0,
                    0
                ]
            ),
            wire::error(abi::BAD_HANDLE)
        );
        assert_eq!(
            call(
                parent,
                [
                    wire::PRESENT_ABORT,
                    self.epoch,
                    self.revision,
                    0,
                    0,
                    0,
                    0,
                    0
                ]
            ),
            [0; 8]
        );
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
