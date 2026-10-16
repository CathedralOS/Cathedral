//! One deferred event receive per app. Control/RPC work continues while apps sleep.
use super::*;
use cathedral_platform_contracts::display::pointer as p;
use cathedral_user_runtime::server::Request;
impl Compositor {
    pub(super) fn defer(&mut self, request: Request, server: &mut Server) -> bool {
        if display::decode(&request.bytes[..request.len]) != Some([p::WAIT, 0, 0, 0, 0, 0]) {
            return false;
        }
        let client = request.client.and_then(app_index).filter(|&c| {
            !request.control
                && self.composition.clients[c].visible
                && self.composition.clients[c].incarnation == request.incarnation
        });
        let error = match client {
            Some(c) if self.pending[c].is_none() => {
                self.pending[c] = Some(request);
                self.stats[1] += 1;
                return true;
            }
            Some(_) => abi::BUSY,
            None => abi::DENIED,
        };
        let _ = server.reply(&request, &display::encode([error, 0, 0, 0, 0, 0]));
        true
    }
    pub(super) fn deliver(&mut self, server: &mut Server) {
        for client in 0..3 {
            let Some(request) = self.pending[client] else {
                continue;
            };
            let words = if let Some(key) = self.composition.focus.next(client) {
                key
            } else {
                let event = self.composition.pointers.next(client);
                if event.kind == p::NONE && !self.wake[client] {
                    continue;
                }
                event.words()
            };
            self.pending[client] = None;
            self.wake[client] = false;
            self.stats[2] += 1;
            let _ = server.reply(&request, &display::encode(words));
        }
    }
}
