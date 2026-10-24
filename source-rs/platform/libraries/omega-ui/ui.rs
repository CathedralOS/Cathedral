// Frame lifecycle, checked commands and row access share one exclusive instance.
use crate::{Api, ButtonVariant, Error, Node, Rectangle, Response, Stats, TextInput};
use core::{
    mem,
    sync::atomic::{AtomicU64, Ordering},
};
// Reusing a buffer address must not make tokens from a destroyed instance valid.
static NEXT_INSTANCE: AtomicU64 = AtomicU64::new(1);
pub struct Ui<'memory, 'code> {
    api: Api<'code>,
    state: &'memory mut [u8],
    packet: &'memory mut [i32; 4096],
    stats: Stats,
    generation: u64,
    owner: u64,
    published: bool,
}
impl<'memory, 'code> Ui<'memory, 'code> {
    pub fn new(
        api: Api<'code>,
        state: &'memory mut [u8],
        packet: &'memory mut [u8],
    ) -> Result<Self, Error> {
        let (bytes, alignment) = api.layout()?;
        if state.len() < bytes
            || !(state.as_ptr() as usize).is_multiple_of(alignment)
            || packet.len() < 4096 * mem::size_of::<i32>()
            || !(packet.as_ptr() as usize).is_multiple_of(mem::align_of::<i32>())
        {
            return Err(Error::Storage);
        }
        state[..bytes].fill(0);
        let owner = NEXT_INSTANCE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .map_err(|_| Error::Capacity)?;
        // SAFETY: Exclusive, separate and aligned writable storage. i32 accepts
        // every bit pattern; the mutable borrow prevents concurrent row access.
        let packet = unsafe { &mut *packet.as_mut_ptr().cast::<[i32; 4096]>() };
        packet.fill(0);
        Ok(Self {
            api,
            state,
            packet,
            stats: Stats::default(),
            generation: 0,
            owner,
            published: false,
        })
    }
    fn invoke(&mut self, operation: i32, arguments: &[i32]) -> Result<Stats, Error> {
        if arguments.len() > 7 {
            return Err(Error::Layout);
        }
        self.packet[4080..4088].fill(0);
        self.packet[4080] = operation;
        self.packet[4081..4081 + arguments.len()].copy_from_slice(arguments);
        // SAFETY: Construction checks the opaque state and exclusive packet;
        // each public command checks its inputs before crossing this ABI.
        let status = unsafe { (self.api.apply)(self.state.as_mut_ptr(), self.packet.as_mut_ptr()) };
        if status != 0 {
            return Err(Error::Native(status));
        }
        if self.packet[4068] != 2 {
            return Err(Error::Layout);
        }
        let width = usize::try_from(self.packet[4076]).map_err(|_| Error::Rectangle)?;
        let height = usize::try_from(self.packet[4077]).map_err(|_| Error::Rectangle)?;
        if width > 3840 || height > 2160 {
            return Err(Error::Rectangle);
        }
        self.stats = Stats {
            animating: self.packet[4069] != 0,
            changed: self.packet[4070] != 0,
            focus: self.packet[4072],
            revision: self.packet[4073],
            background: self.packet[4075] as u32,
            width,
            height,
        };
        Ok(self.stats)
    }
    pub fn stats(&self) -> Stats {
        self.stats
    }
    pub fn resize(&mut self, width: i32, height: i32) -> Result<Stats, Error> {
        if !(1..=3840).contains(&width) || !(1..=2160).contains(&height) {
            return Err(Error::Rectangle);
        }
        self.published = false;
        self.generation = self.generation.checked_add(1).ok_or(Error::Capacity)?;
        self.invoke(0, &[width, height])
    }
    /// Stage one input edge, then build before staging another edge.
    pub fn pointer(
        &mut self,
        x: i32,
        y: i32,
        down: bool,
        pressed: bool,
        released: bool,
    ) -> Result<Stats, Error> {
        self.invoke(
            1,
            &[
                x,
                y,
                i32::from(down) | (i32::from(pressed) << 1) | (i32::from(released) << 2),
            ],
        )
    }
    pub fn key(
        &mut self,
        key: i32,
        character: i32,
        control: bool,
        shift: bool,
    ) -> Result<Stats, Error> {
        if character != 0 && !(32..=126).contains(&character) {
            return Err(Error::Text);
        }
        self.invoke(
            2,
            &[key, character, i32::from(control) | (i32::from(shift) << 1)],
        )
    }
    pub fn tick(&mut self, elapsed_ms: i32) -> Result<Stats, Error> {
        if !(0..=1000).contains(&elapsed_ms) {
            return Err(Error::Layout);
        }
        self.invoke(6, &[elapsed_ms])
    }
    /// Rebuild with the app's updated model until the shared frame settles.
    /// Closures must describe the whole view on every pass. Click edges occur
    /// once; model changes made by that click are visible on the following pass.
    pub fn build(
        &mut self,
        mut compose: impl FnMut(&mut Frame<'_, 'memory, 'code>) -> Result<(), Error>,
    ) -> Result<Stats, Error> {
        if self.stats.width == 0 || self.stats.height == 0 {
            return Err(Error::Layout);
        }
        self.published = false;
        self.invoke(10, &[])?;
        for _ in 0..4 {
            self.generation = self.generation.checked_add(1).ok_or(Error::Capacity)?;
            let result = compose(&mut Frame { ui: self }).and_then(|()| self.invoke(11, &[]));
            match result {
                Ok(stats) if self.packet[4064] == 0 => {
                    self.published = true;
                    return Ok(stats);
                }
                Ok(_) => (),
                Err(error) => {
                    self.invoke(12, &[])?;
                    return Err(error);
                }
            }
        }
        self.invoke(12, &[])?;
        Err(Error::Layout)
    }
    pub fn bounds(&mut self, node: Node) -> Result<Rectangle, Error> {
        if node.generation != self.generation || node.owner != self.owner {
            return Err(Error::StaleNode);
        }
        if !self.published || node.index <= 0 {
            return Err(Error::Layout);
        }
        let stats = self.invoke(5, &[node.index])?;
        let mut values = [0; 4];
        for (target, value) in values.iter_mut().zip(&self.packet[4000..4004]) {
            *target = usize::try_from(*value).map_err(|_| Error::Rectangle)?;
        }
        let [x, y, width, height] = values;
        if x.checked_add(width).is_none_or(|end| end > stats.width)
            || y.checked_add(height).is_none_or(|end| end > stats.height)
        {
            return Err(Error::Rectangle);
        }
        Ok(Rectangle {
            x,
            y,
            width,
            height,
        })
    }
    pub fn row(&mut self, y: usize) -> Result<&[i32], Error> {
        if !self.published || y >= self.stats.height {
            return Err(Error::Rectangle);
        }
        let stats = self.invoke(3, &[y as i32])?;
        Ok(&self.packet[..stats.width])
    }
}
pub struct Frame<'frame, 'memory, 'code> {
    ui: &'frame mut Ui<'memory, 'code>,
}

#[cfg(all(test, windows))]
#[test]
fn native_commands_reject_malformed_packets_independently_of_rust_input_checks() {
    crate::tests::with_ui(|ui| {
        assert_eq!(ui.invoke(3, &[0]), Err(Error::Native(3)));
        assert_eq!(ui.invoke(0, &[3841, 200]), Err(Error::Native(3)));
        ui.invoke(10, &[]).unwrap();
        ui.packet[4040] = 129;
        assert_eq!(ui.invoke(27, &[101, 0, 200]), Err(Error::Native(5)));
        ui.packet[4040] = 1;
        ui.packet[0] = 65;
        ui.packet[4041] = 2;
        ui.packet[4042] = 0;
        assert_eq!(ui.invoke(27, &[101, 0, 200]), Err(Error::Native(5)));
        ui.packet[4040] = -1;
        assert_eq!(ui.invoke(23, &[24, 400, -1]), Err(Error::Native(5)));
        ui.packet[4040] = 1;
        ui.packet[0] = i32::MAX;
        assert_eq!(ui.invoke(23, &[24, 400, -1]), Err(Error::Native(5)));
        ui.invoke(12, &[]).unwrap();
        assert!(
            ui.build(|view| {
                view.label("Recovered")?;
                Ok(())
            })
            .is_ok()
        );
    });
}
impl Frame<'_, '_, '_> {
    fn command(&mut self, operation: i32, arguments: &[i32]) -> Result<Response, Error> {
        self.ui.invoke(operation, arguments)?;
        Ok(Response {
            node: Node {
                index: self.ui.packet[4060],
                generation: self.ui.generation,
                owner: self.ui.owner,
            },
            clicked: self.ui.packet[4061] & 1 != 0,
            changed: self.ui.packet[4061] & 2 != 0,
        })
    }
    fn text_bytes(&mut self, text: &str) -> Result<(), Error> {
        if text.len() > 256 || !text.bytes().all(|byte| (32..=126).contains(&byte)) {
            return Err(Error::Text);
        }
        self.ui.packet[4040] = text.len() as i32;
        for (target, byte) in self.ui.packet[..text.len()].iter_mut().zip(text.bytes()) {
            *target = i32::from(byte);
        }
        Ok(())
    }
    pub fn theme(&mut self, dark: bool) -> Result<(), Error> {
        self.command(7, &[i32::from(dark)]).map(|_| ())
    }
    pub fn column(
        &mut self,
        gap: i32,
        padding: i32,
        fill_width: bool,
        fill_height: bool,
    ) -> Result<Node, Error> {
        Ok(self
            .command(
                20,
                &[
                    gap,
                    padding,
                    i32::from(fill_width) | (i32::from(fill_height) << 1),
                ],
            )?
            .node)
    }
    pub fn row(
        &mut self,
        gap: i32,
        padding: i32,
        fill_width: bool,
        center: bool,
    ) -> Result<Node, Error> {
        Ok(self
            .command(
                21,
                &[
                    gap,
                    padding,
                    i32::from(fill_width) | (i32::from(center) << 1),
                ],
            )?
            .node)
    }
    pub fn card(&mut self, gap: i32, padding: i32, fill_width: bool) -> Result<Node, Error> {
        Ok(self
            .command(32, &[gap, padding, i32::from(fill_width)])?
            .node)
    }
    pub fn end(&mut self) -> Result<(), Error> {
        self.command(22, &[]).map(|_| ())
    }
    pub fn spacer(&mut self) -> Result<Node, Error> {
        Ok(self.command(28, &[])?.node)
    }
    pub fn divider(&mut self) -> Result<Node, Error> {
        Ok(self.command(29, &[])?.node)
    }
    pub fn text(&mut self, text: &str, line_height: i32, weight: i32) -> Result<Node, Error> {
        self.text_bytes(text)?;
        Ok(self.command(23, &[line_height, weight, -1])?.node)
    }
    pub fn label(&mut self, text: &str) -> Result<Node, Error> {
        self.text(text, 24, 400)
    }
    pub fn heading(&mut self, text: &str) -> Result<Node, Error> {
        self.text(text, 32, 600)
    }
    pub fn button(
        &mut self,
        id: i32,
        label: &str,
        variant: ButtonVariant,
        width: i32,
        height: i32,
    ) -> Result<Response, Error> {
        self.text_bytes(label)?;
        self.command(24, &[id, variant as i32, width, height])
    }
    pub fn toggle(&mut self, id: i32, value: &mut bool) -> Result<Response, Error> {
        let response = self.command(25, &[id, i32::from(*value)])?;
        *value = self.ui.packet[4062] != 0;
        Ok(response)
    }
    pub fn checkbox(&mut self, id: i32, value: &mut bool) -> Result<Response, Error> {
        let response = self.command(30, &[id, i32::from(*value)])?;
        *value = self.ui.packet[4062] != 0;
        Ok(response)
    }
    pub fn slider(&mut self, id: i32, value: &mut i32, width: i32) -> Result<Response, Error> {
        let response = self.command(26, &[id, *value, width])?;
        *value = self.ui.packet[4062];
        Ok(response)
    }
    pub fn progress(&mut self, id: i32, value: i32, width: i32) -> Result<Node, Error> {
        Ok(self.command(31, &[id, value, width])?.node)
    }
    pub fn text_field(
        &mut self,
        id: i32,
        value: &mut TextInput,
        width: i32,
    ) -> Result<Response, Error> {
        self.ui.packet[4040] = value.length as i32;
        self.ui.packet[4041] = value.caret as i32;
        self.ui.packet[4042] = value.anchor as i32;
        self.ui.packet[4043] = value.scroll;
        for (target, byte) in self.ui.packet[..value.length]
            .iter_mut()
            .zip(&value.bytes[..value.length])
        {
            *target = i32::from(*byte);
        }
        let response = self.command(27, &[id, 0, width])?;
        *value = TextInput::read(self.ui.packet)?;
        Ok(response)
    }
}

#[cfg(all(test, windows))]
#[test]
fn nodes_from_a_destroyed_instance_do_not_match_reused_storage() {
    crate::tests::with_ui(|ui| {
        let mut previous = Node::NONE;
        ui.build(|view| {
            previous = view.label("Old instance")?;
            Ok(())
        })
        .unwrap();
        let api = ui.api;
        // SAFETY: The old wrapper is exclusively borrowed and unused while its
        // two disjoint buffers are reborrowed by the replacement instance.
        let packet =
            unsafe { core::slice::from_raw_parts_mut(ui.packet.as_mut_ptr().cast::<u8>(), 16384) };
        let mut replacement = Ui::new(api, &mut *ui.state, packet).unwrap();
        replacement.resize(420, 400).unwrap();
        replacement
            .build(|view| {
                view.label("Replacement")?;
                Ok(())
            })
            .unwrap();
        assert_eq!(replacement.bounds(previous), Err(Error::StaleNode));
    });
}
