//! Receive one authenticated event; retain reply custody while Omega decides.
use super::*;
use cathedral_user_runtime::server::Event;
impl Host {
    pub(super) fn next(&mut self, p: &mut [u64; PACKET_WORDS]) -> Result<(), Error> {
        if self.request.is_some() {
            return Err(Error(abi::BUSY as i64));
        }
        loop {
            if let Some(probe) = &self.pending
                && probe.progress(&mut self.server)?
            {
                self.pending = None;
            }
            self.server.pause(u32::from(self.pending.is_some()));
            let handles = self.pending.as_ref().map(|probe| [probe.wait().0]);
            let until = self.pending.as_ref().map(|probe| probe.wait().1);
            let Some(event) = self
                .server
                .next_event_or(handles.as_ref().map_or(&[], |h| &h[..]), until)?
            else {
                continue;
            };
            p[..14].fill(0);
            match event {
                Event::Disconnected {
                    client,
                    incarnation,
                } => {
                    p[10] = client as u64;
                    p[11] = incarnation;
                    p[12] = 2;
                }
                Event::Request(request) => self.received(p, request, 1),
                Event::Memory(request, object) => {
                    drop(object);
                    self.received(p, request, 3);
                }
            }
            return Ok(());
        }
    }
    fn received(&mut self, p: &mut [u64; PACKET_WORDS], request: Request, kind: u64) {
        if let Some(words) = wire::decode(&request.bytes[..request.len]) {
            p[..8].copy_from_slice(&words);
        }
        p[10] = request.client.map_or(3, |slot| slot as u64);
        p[11] = request.incarnation;
        p[12] = kind;
        p[13] = request.len as u64;
        self.request = Some(request);
    }
}
