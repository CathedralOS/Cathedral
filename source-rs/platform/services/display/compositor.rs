//! Retained compositor entrance: receive lifecycle/request, authorize, commit, redraw.
mod delivery;
mod input;
mod requests;
mod session;
use crate::surface::Surface;
use cathedral_contracts::{composition as wire, display, user as abi};
use cathedral_display_service::pointer::Router;
use cathedral_display_service::scene::{Client, Rect, Scene};
use cathedral_user_runtime::{
    Error,
    server::{Event, Server},
};
struct Compositor {
    clients: &'static mut [Client; 3],
    scratch: &'static mut Scene,
    labels: &'static mut cathedral_display_service::labels::Labels,
    background: u32,
    output: Surface,
    pointers: Router,
    source: u64,
    focus: cathedral_display_service::focus::Focus,
    tree: cathedral_display_service::hosting::Tree,
    action: u64,
    pending: [Option<cathedral_user_runtime::server::Request>; 3],
    stats: [u64; 4],
    wake: [bool; 3],
}
pub fn run() -> u64 {
    static mut CLIENTS: [Client; 3] = [const { Client::empty() }; 3];
    static mut SCRATCH: Scene = Scene::empty();
    static mut LABELS: cathedral_display_service::labels::Labels =
        cathedral_display_service::labels::Labels::empty();
    let scratch_address = &raw mut SCRATCH;
    let clients_address = &raw mut CLIENTS;
    // SAFETY: This single-threaded process enters run exactly once. Neither static
    // has another reference; scratch and retained state are separate allocations.
    let (scratch, clients) = unsafe { (&mut *scratch_address, &mut *clients_address) };
    let labels_address = &raw mut LABELS;
    // SAFETY: Same single-entry ownership as the retained scene statics above.
    let labels = unsafe { &mut *labels_address };
    let mut compositor = Compositor::open(clients, scratch, labels);
    // The optional fifth connection has no scene/input authority; it exercises
    // denied requests and reply backpressure without adding an application slot.
    let mut server = Server::open_clients(5).unwrap();
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
        labels: &'static mut cathedral_display_service::labels::Labels,
    ) -> Self {
        let output = Surface::open();
        Self {
            clients,
            scratch,
            labels,
            background: 0,
            pointers: Router::new(output.width, output.height),
            output,
            source: 0,
            focus: Default::default(),
            tree: Default::default(),
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
                row.fill(self.background);
                self.labels.paint_row(x, y, row);
                for client in self.clients.iter() {
                    client.paint_row(x, y, row);
                }
                for cursor in self.pointers.cursor_bounds() {
                    if y < cursor.y || y - cursor.y >= cursor.h {
                        continue;
                    }
                    for px in x.max(cursor.x)..(x + len as u64).min(cursor.x + cursor.w) {
                        if let Some(color) = self.pointers.pixel(px, y, self.clients) {
                            row[(px - x) as usize] = color;
                        }
                    }
                }
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
