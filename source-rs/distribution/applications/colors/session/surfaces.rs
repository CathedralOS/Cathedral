//! App-owned surface lifecycle and adversarial local-handle/allowance probes.
use super::*;
pub(super) const ACCENT: u64 = 0xe5e7eb;
pub struct Scene {
    pub epoch: u64,
    base: u64,
    accent: Option<u64>,
    revision: u64,
    close: Option<u64>,
}
impl Scene {
    fn probe(parent: &mut Services, view: wire::Packet) -> Result<Self, u64> {
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
        let retained = 3;
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
            close: None,
        };
        scene.remove_accent(parent);
        scene.restore_accent(parent);
        image::probe(parent, view[1], names[2]).unwrap();
        Ok(scene)
    }
    pub fn new<const PROBES: bool>(parent: &mut Services, view: wire::Packet) -> Result<Self, u64> {
        if PROBES {
            return Self::probe(parent, view);
        }
        let mut names = [0; 4];
        for (rank, name) in names.iter_mut().enumerate() {
            let reply = call(
                parent,
                [wire::SURFACE_CREATE, view[1], rank as u64, 0, 0, 0, 0, 0],
            );
            accepted(reply[0])?;
            *name = reply[1];
        }
        image::submit(parent, view[1], names[2])?;
        Ok(Self {
            epoch: view[1],
            base: names[0],
            accent: Some(names[1]),
            revision: 2,
            close: Some(names[3]),
        })
    }
    pub fn close_button(
        &self,
        parent: &mut Services,
        view: wire::Packet,
        confirm: bool,
    ) -> Result<(), u64> {
        closing::paint(parent, view, self.close.unwrap(), confirm)
    }
    pub fn paint(
        &mut self,
        parent: &mut Services,
        view: wire::Packet,
        color: u64,
        error: u64,
    ) -> Result<(), u64> {
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
        accepted(call(parent, packet)[0])
    }

    pub fn compact_probe(&mut self, parent: &mut Services) {
        self.revision += 1;
        let valid = [wire::PAINT, self.epoch, self.base, 0, 0, 200, 130, 0];
        let foreign = [wire::PAINT, self.epoch, u64::MAX, 0, 0, 1, 1, 0];
        let packet = wire::rectangles(self.epoch, self.revision, &[valid, foreign]).unwrap();
        assert_eq!(call(parent, packet), wire::error(abi::BAD_HANDLE));
    }
    pub fn cursor_probe(&mut self, parent: &mut Services, view: wire::Packet, clipped: bool) {
        self.begin(parent);
        let (x, y) = if clipped { (view[2] - 2, 0) } else { (30, 40) };
        assert_eq!(
            call(
                parent,
                [
                    wire::SURFACE_CURSOR,
                    self.epoch,
                    self.accent.unwrap(),
                    x,
                    y,
                    7,
                    12,
                    0xffd166
                ]
            ),
            [0; 8]
        );
    }
    fn begin(&mut self, parent: &mut Services) {
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
    pub fn commit(&self, parent: &mut Services) {
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
    pub fn staged_probe(&mut self, parent: &mut Services) {
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
    pub fn rejected_probe(&mut self, parent: &mut Services) {
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
    fn paint_accent(&self, parent: &mut Services) {
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
    pub fn remove_accent(&mut self, parent: &mut Services) {
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
    pub fn restore_accent(&mut self, parent: &mut Services) {
        assert!(self.accent.is_none());
        let reply = call(parent, [wire::SURFACE_CREATE, self.epoch, 1, 0, 0, 0, 0, 0]);
        assert_eq!(reply[0], 0);
        self.accent = Some(reply[1]);
        self.paint_accent(parent);
    }
}

fn accepted(status: u64) -> Result<(), u64> {
    if status == 0 { Ok(()) } else { Err(status) }
}
