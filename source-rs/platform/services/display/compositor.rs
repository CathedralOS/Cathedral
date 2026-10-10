//! Retained compositor entrance: receive lifecycle/request, authorize, commit, redraw.
mod requests;
use crate::surface::Surface;
use cathedral_contracts::{composition as wire, display, user as abi};
use cathedral_display_service::scene::{Client, Rect, Scene};
use cathedral_user_runtime::{
    Error,
    server::{Event, Server},
};
struct Compositor {
    clients: &'static mut [Client; 2],
    scratch: &'static mut Scene,
    background: u32,
    output: Surface,
}
pub fn run() -> u64 {
    static mut CLIENTS: [Client; 2] = [const { Client::empty() }; 2];
    static mut SCRATCH: Scene = Scene::empty();
    let scratch_address = &raw mut SCRATCH;
    let clients_address = &raw mut CLIENTS;
    // SAFETY: This single-threaded process enters run exactly once. Neither static
    // has another reference; scratch and retained state are separate allocations.
    let (scratch, clients) = unsafe { (&mut *scratch_address, &mut *clients_address) };
    let mut compositor = Compositor {
        clients,
        scratch,
        background: 0,
        output: Surface::open(),
    };
    let mut server = Server::open_clients(2).unwrap();
    loop {
        match server.next_event() {
            Ok(Event::Disconnected {
                client,
                incarnation,
            }) => {
                compositor.clients[client].disconnect(incarnation);
                compositor.redraw(compositor.clients[client].scope);
            }
            Ok(Event::Request(request)) => {
                let reply = compositor.request(&request);
                if server.reply(&request, &display::encode(reply)).is_err() {
                    return 1;
                }
            }
            Err(Error(code)) if code == abi::PEER_CLOSED as i64 => return 0,
            Err(_) => return 1,
        }
    }
}
impl Compositor {
    fn redraw(&mut self, rect: Rect) {
        for y in rect.y..rect.y + rect.h {
            for x in rect.x..rect.x + rect.w {
                let color = self
                    .clients
                    .iter()
                    .filter_map(|client| client.pixel(x, y))
                    .next_back()
                    .unwrap_or(self.background);
                self.output.put(x, y, color);
            }
        }
    }
}
