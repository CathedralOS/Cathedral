//! Scoped atomic image publication for any caller-authored view. Region nodes
//! are chosen by the app to fit its granted backing; removed regions retire
//! their surfaces in the same commit. No distribution code is imported here.
use crate::{Error, Node, Rectangle, Stats, Ui};
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{ipc::Handle, memory::Private, resources};
type Pair = (Handle, Handle);
pub struct Presenter {
    parent: Pair,
    epoch: u64,
    width: u64,
    height: u64,
    names: [u64; wire::MAX_SURFACES],
    count: usize,
    revision: u64,
    painted: Option<(Stats, usize, usize, usize)>,
    painted_nodes: [Node; wire::MAX_SURFACES],
}
impl Presenter {
    /// The supplied interaction binding owns this presenter exclusively. A new
    /// view epoch retires its surfaces; create a new presenter for that epoch.
    pub fn new(parent: Pair, view: wire::Packet) -> Result<Self, Error> {
        if view[0] != 0 || view[1] == 0 || view[2] == 0 || view[3] == 0 {
            return Err(Error::Rectangle);
        }
        Ok(Self {
            parent,
            epoch: view[1],
            width: view[2],
            height: view[3],
            names: [0; wire::MAX_SURFACES],
            count: 0,
            revision: 0,
            painted: None,
            painted_nodes: [Node::NONE; wire::MAX_SURFACES],
        })
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Render these nonempty regions from one completed frame. Origin is local
    /// to the supplied view. Budget refusal happens before starting a batch.
    pub fn present(
        &mut self,
        ui: &mut Ui<'_, '_>,
        nodes: &[Node],
        origin: (usize, usize),
    ) -> Result<bool, Error> {
        if nodes.len() >= wire::MAX_SURFACES {
            return Err(Error::Capacity);
        }
        let signature = (ui.stats(), origin.0, origin.1, nodes.len());
        if self.painted == Some(signature) && self.painted_nodes[..nodes.len()] == *nodes {
            return Ok(false);
        }
        let mut rectangles = [Rectangle::default(); wire::MAX_SURFACES];
        let mut pages = 0;
        for (rectangle, node) in rectangles.iter_mut().zip(nodes) {
            *rectangle = ui.bounds(*node)?;
            if rectangle.width == 0
                || rectangle.height == 0
                || origin
                    .0
                    .checked_add(rectangle.x)
                    .and_then(|x| x.checked_add(rectangle.width))
                    .is_none_or(|end| end > self.width as usize)
                || origin
                    .1
                    .checked_add(rectangle.y)
                    .and_then(|y| y.checked_add(rectangle.height))
                    .is_none_or(|end| end > self.height as usize)
            {
                return Err(Error::Rectangle);
            }
            let size = rectangle
                .width
                .checked_mul(rectangle.height)
                .and_then(|size| size.checked_mul(4))
                .ok_or(Error::Capacity)?
                .div_ceil(4096);
            if size > cathedral_kernel_contracts::memory::MAX_PAGES {
                return Err(Error::Capacity);
            }
            pages += size;
        }
        let budget = resources::budget().map_err(platform)?;
        let physical = resources::physical_budget().map_err(platform)?;
        if pages as u64
            > budget
                .limits
                .private_pages
                .saturating_sub(budget.used.private_pages)
            || pages as u64 > physical.reserved.saturating_sub(physical.charged)
        {
            return Err(Error::Capacity);
        }
        let desired = nodes.len() + 1;
        for rank in 0..desired {
            if self.names[rank] == 0 {
                let reply = call(
                    self.parent,
                    [wire::SURFACE_CREATE, self.epoch, rank as u64, 0, 0, 0, 0, 0],
                )?;
                self.names[rank] = reply[1];
                self.count = self.count.max(rank + 1);
            }
        }
        self.revision = self.revision.checked_add(1).ok_or(Error::Capacity)?;
        call(
            self.parent,
            [
                wire::PRESENT_BEGIN,
                self.epoch,
                self.revision,
                0,
                0,
                0,
                0,
                0,
            ],
        )?;
        let staged = self.stage(ui, &rectangles[..nodes.len()], origin);
        self.finish(staged)?;
        self.names[desired..].fill(0);
        self.count = desired;
        self.painted = Some(signature);
        self.painted_nodes[..nodes.len()].copy_from_slice(nodes);
        Ok(true)
    }
    /// Release every owned surface while the binding is still live. Teardown
    /// after view revocation belongs to the parent, not a blocking Drop call.
    pub fn retire(&mut self) -> Result<(), Error> {
        if self.count == 0 {
            return Ok(());
        }
        self.revision = self.revision.checked_add(1).ok_or(Error::Capacity)?;
        call(
            self.parent,
            [
                wire::PRESENT_BEGIN,
                self.epoch,
                self.revision,
                0,
                0,
                0,
                0,
                0,
            ],
        )?;
        let staged = self.names[..self.count].iter().try_for_each(|name| {
            call(self.parent, [wire::ERASE, self.epoch, *name, 0, 0, 0, 0, 0]).map(|_| ())
        });
        self.finish(staged)?;
        self.names.fill(0);
        self.count = 0;
        self.painted = None;
        self.painted_nodes.fill(Node::NONE);
        Ok(())
    }
    fn finish(&self, staged: Result<(), Error>) -> Result<(), Error> {
        let result = staged.and_then(|()| {
            call(
                self.parent,
                [
                    wire::PRESENT_COMMIT,
                    self.epoch,
                    self.revision,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
            )
            .map(|_| ())
        });
        if let Err(error) = result {
            call(
                self.parent,
                [
                    wire::PRESENT_ABORT,
                    self.epoch,
                    self.revision,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
            )?;
            return Err(error);
        }
        Ok(())
    }
    fn stage(
        &self,
        ui: &mut Ui<'_, '_>,
        rectangles: &[Rectangle],
        origin: (usize, usize),
    ) -> Result<(), Error> {
        call(
            self.parent,
            [
                wire::PAINT,
                self.epoch,
                self.names[0],
                0,
                0,
                self.width,
                self.height,
                ui.stats().background as u64,
            ],
        )?;
        for (index, rectangle) in rectangles.iter().enumerate() {
            let mut image =
                Private::allocate((rectangle.width * rectangle.height * 4).div_ceil(4096))
                    .map_err(platform)?;
            for y in 0..rectangle.height {
                let row = ui.row(rectangle.y + y)?;
                for (x, pixel) in row[rectangle.x..rectangle.x + rectangle.width]
                    .iter()
                    .enumerate()
                {
                    let at = (y * rectangle.width + x) * 4;
                    image.bytes_mut()[at..at + 4].copy_from_slice(&pixel.to_le_bytes());
                }
            }
            let mut image = image.freeze().map_err(platform)?;
            image.unmap().map_err(platform)?;
            self.parent
                .0
                .send_memory(
                    &wire::encode([
                        wire::SURFACE_IMAGE,
                        self.epoch,
                        self.names[index + 1],
                        (origin.0 + rectangle.x) as u64,
                        (origin.1 + rectangle.y) as u64,
                        rectangle.width as u64,
                        rectangle.height as u64,
                        rectangle.width as u64,
                    ]),
                    &image,
                )
                .map_err(platform)?;
            receive(self.parent, wire::SURFACE_IMAGE)?;
        }
        for name in &self.names[rectangles.len() + 1..self.count] {
            call(self.parent, [wire::ERASE, self.epoch, *name, 0, 0, 0, 0, 0])?;
        }
        Ok(())
    }
}
fn platform(error: cathedral_user_runtime::Error) -> Error {
    Error::Platform(error.0)
}
fn call(pair: Pair, packet: wire::Packet) -> Result<wire::Packet, Error> {
    pair.0.send(&wire::encode(packet)).map_err(platform)?;
    receive(pair, packet[0])
}
fn receive(pair: Pair, operation: u64) -> Result<wire::Packet, Error> {
    let mut bytes = [0; 64];
    let length =
        cathedral_user_runtime::matrix::upstream::receive(pair.1, &mut bytes, operation, 300)
            .map_err(platform)?;
    let reply = wire::decode(&bytes[..length]).ok_or(Error::Layout)?;
    if reply[0] != 0 {
        return Err(Error::Platform(reply[0] as i64));
    }
    Ok(reply)
}
