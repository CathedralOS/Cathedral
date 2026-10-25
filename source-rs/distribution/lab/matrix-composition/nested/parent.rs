//! Fixture parent owns the actual grant and services the matrix's supplied IPC.
use cathedral_kernel_contracts::user as abi;
use cathedral_matrix::world::{Grant, World};
use cathedral_platform_contracts::{imports, matrix as wire, services};
use cathedral_user_runtime::{
    Error,
    matrix::admission::{self, Admitted},
    resources,
    server::{Event, Server},
    task::Launch,
    time,
};
pub(super) struct Parent {
    pub world: &'static mut World<2>,
    server: Server,
    child: Option<Admitted>,
}
pub fn run() -> u64 {
    static mut WORLD: World<2> = World::empty();
    let address = &raw mut WORLD;
    // SAFETY: The single process entry exclusively owns this static arena.
    let world = unsafe { &mut *address };
    world.inherit(Grant {
        epoch: 1,
        width: 800,
        height: 800,
        keys: wire::KEY_MASK,
    });
    let mut server = Server::open_clients(1).unwrap();
    server.pause(1); // Root supervises lifetime; no fixture commands on this edge.
    let mut parent = Parent {
        world,
        server,
        child: None,
    };
    let before = resources::budget().unwrap();
    for _ in 0..3 {
        parent.start();
        super::behavior::check(&mut parent);
        parent.stop();
        assert_eq!(resources::budget().unwrap(), before);
    }
    // Return with a live matrix and peers. The kernel must reap the descendants
    // before our owner can restart this entire tree with the same launch loans.
    parent.start();
    assert_eq!(parent.view(0)[0], 0);
    0
}
impl Parent {
    fn start(&mut self) {
        assert_eq!(
            self.world
                .source_command([wire::POINTER_SOURCE, 0, 0, 0, 0, 0, 0, 0]),
            [0; 8]
        );
        let loans = [1, 2, 3].map(|i| Launch::at(i).unwrap());
        let child =
            admission::spawn_brokered(Launch::at(0).unwrap(), 0, &loans, imports::INTERACTION)
                .unwrap();
        self.world.disconnected(0);
        self.server.install(0, child.data).unwrap();
        self.child = Some(child);
        assert!(loans[0].spawn(0).is_err());
        self.place(240, 450, 16);
    }
    fn stop(&mut self) {
        self.server.remove(0);
        self.world.disconnected(0);
        let child = self.child.take().unwrap();
        child.task.cancel().unwrap();
        child.task.wait().unwrap();
        child.close_connections().unwrap();
    }
    pub fn place(&mut self, width: u64, height: u64, keys: u64) {
        assert_eq!(
            self.world
                .owner_command([wire::PLACE, 0, 100, 80, width, height, keys, 0]),
            [0; 8]
        );
    }
    pub fn call(&mut self, mut words: wire::Packet, slot: Option<u64>) -> wire::Packet {
        if let Some(slot) = slot {
            words[0] |= (slot + 1) << 32;
        }
        let control = self.child.as_ref().unwrap().control;
        control.0.send(&wire::encode(words)).unwrap();
        let until = time::after(300).unwrap();
        loop {
            let mut bytes = [0; 64];
            match control.1.receive_until(&mut bytes, time::now().unwrap()) {
                Ok(len) => return wire::decode(&bytes[..len]).unwrap(),
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                other => panic!("nested control: {other:?}"),
            }
            assert!(!time::reached(time::now().unwrap(), until));
            let Some(event) = self
                .server
                .next_event_or(&[control.1], Some(until))
                .unwrap()
            else {
                continue;
            };
            let Event::Request(request) = event else {
                panic!("unexpected upstream event");
            };
            let child = self.child.as_mut().unwrap();
            assert_eq!(request.client, Some(0));
            assert_eq!(request.incarnation, child.data.0.raw());
            let words = wire::decode(&request.bytes[..request.len]).unwrap();
            if words[0] == services::ACQUIRE {
                child
                    .services
                    .as_mut()
                    .unwrap()
                    .reply(&mut self.server, &request, words)
                    .unwrap();
            } else {
                assert_eq!(words, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
                let reply = self.world.request(0, request.incarnation, words);
                self.server.reply(&request, &wire::encode(reply)).unwrap();
            }
        }
    }
    pub fn view(&mut self, slot: u64) -> wire::Packet {
        self.call([wire::VIEW, 0, 0, 0, 0, 0, 0, 0], Some(slot))
    }
    pub fn next(&mut self, slot: u64, epoch: u64) -> wire::Packet {
        self.call([wire::NEXT, epoch, 0, 0, 0, 0, 0, 0], Some(slot))
    }
}
