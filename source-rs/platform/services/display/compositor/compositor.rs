// Retained compositor entrance: receive lifecycle/request, authorize, commit, redraw.
use crate::surface::Surface;
use cathedral_display_service::composition::{
    Composition,
    scene::{Client, Rect, Scene, labels::Labels},
};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display;
use cathedral_platform_contracts::display::composition as wire;
use cathedral_user_runtime::{
    Error,
    server::{Event, Server},
};
// Only explicitly installed bootstrap endpoints occupy this slot; ordinary
// scene/source links keep their existing indices and cannot claim it.
const BOOTSTRAP: usize = 5;
struct Compositor {
    presentation: &'static mut presentation::Presentation,
    composition: Composition<'static>,
    output: Surface,
    source: u64,
    action: u64,
    pending: [Option<cathedral_user_runtime::server::Request>; 3],
    stats: [u64; 4],
    images: [Option<cathedral_user_runtime::memory::Object>; wire::MATRIX_TILES],
    wake: [bool; 3],
}
pub fn run(environment: cathedral_user_runtime::task::Environment) -> u64 {
    static mut CLIENTS: [Client; 3] = [const { Client::empty() }; 3];
    static mut SCRATCH: Scene = Scene::empty();
    static mut LABELS: Labels = Labels::empty();
    static mut TILES: cathedral_display_service::composition::scene::tiles::Tiles =
        cathedral_display_service::composition::scene::tiles::Tiles::empty();
    static mut PRESENTATION: presentation::Presentation = presentation::Presentation::empty();
    let scratch_address = &raw mut SCRATCH;
    let clients_address = &raw mut CLIENTS;
    // SAFETY: This single-threaded process enters run exactly once. Neither static
    // has another reference; scratch and retained state are separate allocations.
    let (scratch, clients) = unsafe { (&mut *scratch_address, &mut *clients_address) };
    let labels_address = &raw mut LABELS;
    // SAFETY: Same single-entry ownership as the retained scene statics above.
    let labels = unsafe { &mut *labels_address };
    let tiles_address = &raw mut TILES;
    // SAFETY: Single entry, disjoint retained storage, like the scene/label arrays.
    let tiles = unsafe { &mut *tiles_address };
    let presentation_address = &raw mut PRESENTATION;
    // SAFETY: Single entry and exclusive process-owned storage, as for retained tiles.
    let presentation = unsafe { &mut *presentation_address };
    let mut compositor = Compositor::open(clients, scratch, labels, tiles, presentation);
    // The optional fifth connection has no scene/input authority; it exercises
    // denied requests and reply backpressure without adding an application slot.
    let mut server = Server::open_clients(if environment.len() == 2 { 6 } else { 5 }).unwrap();
    server.enable_memory();
    if environment.len() == 2 {
        // Keep bootstrap configuration separate from the root's presentation
        // stream. set_control replaces an unopened port; the installed slot
        // takes that port's custody, never aliases an active shared client.
        server
            .install(
                BOOTSTRAP,
                (
                    cathedral_user_runtime::ipc::Handle::bootstrap(0).unwrap(),
                    cathedral_user_runtime::ipc::Handle::bootstrap(1).unwrap(),
                ),
            )
            .unwrap();
        static mut BOOT: cathedral_user_runtime::server::SharedSlot =
            cathedral_user_runtime::server::SharedSlot::empty();
        let boot = &raw mut BOOT;
        // SAFETY: Single entry lends this separate slot until service exit.
        server
            .enable_shared(BOOTSTRAP, unsafe { &mut *boot })
            .unwrap();
        server.set_control((
            environment.endpoint(0).unwrap(),
            environment.endpoint(1).unwrap(),
        ));
        static mut SHARED: cathedral_user_runtime::server::SharedSlot =
            cathedral_user_runtime::server::SharedSlot::empty();
        let slot = &raw mut SHARED;
        // SAFETY: Single-entry service owns this connection storage until exit.
        server.enable_shared_control(unsafe { &mut *slot }).unwrap();
    } else {
        assert!(environment.is_empty());
        static mut CONTROL: cathedral_user_runtime::server::SharedSlot =
            cathedral_user_runtime::server::SharedSlot::empty();
        static mut CLIENT_TRANSPORTS: [cathedral_user_runtime::server::SharedSlot; 5] =
            [const { cathedral_user_runtime::server::SharedSlot::empty() }; 5];
        let control = &raw mut CONTROL;
        let clients = &raw mut CLIENT_TRANSPORTS;
        // SAFETY: Single service entry exclusively lends distinct process-local slots.
        server
            .enable_shared_control(unsafe { &mut *control })
            .unwrap();
        // SAFETY: Disjoint array from control and the ordinary bootstrap slots above.
        for (index, slot) in unsafe { &mut *clients }.iter_mut().enumerate() {
            server.enable_pipeline(index, slot).unwrap();
        }
    }
    loop {
        match server.next_event() {
            Ok(Event::Disconnected {
                client,
                incarnation,
            }) => {
                compositor.disconnected(client, incarnation);
            }
            Ok(Event::Memory(request, object)) => {
                let reply = compositor
                    .matrix_image(&request, object)
                    .unwrap_or_else(|status| [status, 0, 0, 0, 0, 0]);
                if server.reply(&request, &display::encode(reply)).is_err() {
                    return 1;
                }
            }
            Ok(Event::Request(request)) => {
                compositor.stats[0] += 1;
                if !compositor.defer(request, &mut server) {
                    let reply = compositor.request(&request);
                    if server.reply(&request, &display::encode(reply)).is_err() {
                        return 1;
                    }
                }
            }
            Err(Error(code)) if code == abi::PEER_CLOSED as i64 => return 0,
            Err(_) => return 1,
        }
        compositor.deliver(&mut server);
    }
}
impl Compositor {
    // Finish construction before dispatch so its temporary defaults do not stay
    // on the guarded user stack while rasterization and IPC nest below the loop.
    fn open(
        clients: &'static mut [Client; 3],
        scratch: &'static mut Scene,
        labels: &'static mut Labels,
        tiles: &'static mut cathedral_display_service::composition::scene::tiles::Tiles,
        presentation: &'static mut presentation::Presentation,
    ) -> Self {
        let output = Surface::open();
        Self {
            presentation,
            composition: Composition::new(
                output.width,
                output.height,
                clients,
                scratch,
                labels,
                tiles,
            ),
            output,
            source: 0,
            action: 0,
            pending: [None; 3],
            stats: [0; 4],
            images: [const { None }; wire::MATRIX_TILES],
            wake: [false; 3],
        }
    }
    fn redraw(&mut self, rect: Rect) {
        if !rect.valid() {
            return;
        }
        #[cfg(feature = "trace-input")]
        cathedral_user_runtime::time::trace("compositor-redraw-begin");
        self.stats[3] += 1;
        let images = core::array::from_fn(|i| {
            self.images[i]
                .as_ref()
                .and_then(|o| o.bytes())
                .unwrap_or(&[])
        });
        let mut row = [0u32; 256];
        for y in rect.y..rect.y + rect.h {
            for x in (rect.x..rect.x + rect.w).step_by(row.len()) {
                let len = (rect.x + rect.w - x).min(row.len() as u64) as usize;
                let row = &mut row[..len];
                self.composition.paint_images(x, y, row, &images);
                self.output.row(x, y, row);
            }
        }
        #[cfg(feature = "trace-input")]
        cathedral_user_runtime::time::trace("compositor-redraw-end");
    }
}

fn app_index(link: usize) -> Option<usize> {
    match link {
        0 | 1 => Some(link),
        3 => Some(2),
        _ => None,
    }
}
