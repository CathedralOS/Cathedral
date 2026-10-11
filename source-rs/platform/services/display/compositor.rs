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
    let scratch_address = &raw mut SCRATCH;
    let clients_address = &raw mut CLIENTS;
    // SAFETY: This single-threaded process enters run exactly once. Neither static
    // has another reference; scratch and retained state are separate allocations.
    let (scratch, clients) = unsafe { (&mut *scratch_address, &mut *clients_address) };
    let output = Surface::open();
    let mut compositor = Compositor {
        clients,
        scratch,
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
    };
    let mut server = Server::open_clients(4).unwrap();
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
    fn redraw(&mut self, rect: Rect) {
        self.stats[3] += 1;
        for y in rect.y..rect.y + rect.h {
            for x in rect.x..rect.x + rect.w {
                let color = self
                    .clients
                    .iter()
                    .filter_map(|client| client.pixel(x, y))
                    .next_back()
                    .unwrap_or(self.background);
                self.output.put(
                    x,
                    y,
                    self.pointers.pixel(x, y, self.clients).unwrap_or(color),
                );
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
