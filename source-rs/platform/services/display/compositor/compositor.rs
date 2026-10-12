// Retained compositor entrance: receive lifecycle/request, authorize, commit, redraw.
use crate::surface::Surface;
use cathedral_contracts::{composition as wire, display, user as abi};
use cathedral_display_service::composition::{
    Composition,
    scene::{Client, Rect, Scene, labels::Labels},
};
use cathedral_user_runtime::{
    Error,
    server::{Event, Server},
};
struct Compositor {
    composition: Composition<'static>,
    output: Surface,
    source: u64,
    action: u64,
    pending: [Option<cathedral_user_runtime::server::Request>; 3],
    stats: [u64; 4],
    wake: [bool; 3],
}
pub fn run(environment: cathedral_user_runtime::task::Environment) -> u64 {
    static mut CLIENTS: [Client; 3] = [const { Client::empty() }; 3];
    static mut SCRATCH: Scene = Scene::empty();
    static mut LABELS: Labels = Labels::empty();
    static mut TILES: cathedral_display_service::composition::scene::tiles::Tiles =
        cathedral_display_service::composition::scene::tiles::Tiles::empty();
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
    let mut compositor = Compositor::open(clients, scratch, labels, tiles);
    // The optional fifth connection has no scene/input authority; it exercises
    // denied requests and reply backpressure without adding an application slot.
    let mut server = Server::open_clients(5).unwrap();
    if environment.len() == 2 {
        server.set_control((
            environment.endpoint(0).unwrap(),
            environment.endpoint(1).unwrap(),
        ));
    } else {
        assert!(environment.is_empty());
    }
    loop {
        match server.next_event() {
            Ok(Event::Disconnected {
                client,
                incarnation,
            }) => {
                compositor.disconnected(client, incarnation);
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
    ) -> Self {
        let output = Surface::open();
        Self {
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
            wake: [false; 3],
        }
    }
    fn redraw(&mut self, rect: Rect) {
        if !rect.valid() {
            return;
        }
        self.stats[3] += 1;
        let mut row = [0u32; 256];
        for y in rect.y..rect.y + rect.h {
            for x in (rect.x..rect.x + rect.w).step_by(row.len()) {
                let len = (rect.x + rect.w - x).min(row.len() as u64) as usize;
                let row = &mut row[..len];
                self.composition.paint_row(x, y, row);
                self.output.row(x, y, row);
            }
        }
    }
}

fn app_index(link: usize) -> Option<usize> {
    match link {
        0 | 1 => Some(link),
        3 => Some(2),
        _ => None,
    }
}
