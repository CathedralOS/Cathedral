//! Stock catalog wire protocol over database content. Matrix delegation remains
//! in matrix/service/parent/storage; this mode never accepts private-object ops.
use cathedral_database_catalog::Catalog;
use cathedral_database_objects::Store;
use cathedral_kernel_contracts::user as abi;
use cathedral_object_store::protocol::Client;
use cathedral_platform_contracts::storage as wire;
use cathedral_user_runtime::{
    server::{Event, Server},
    write,
};
pub fn serve(store: &Store<'_>) {
    let mut catalog = Catalog::open(store).unwrap();
    let mut clients = core::array::from_fn::<_, { wire::ROOTS }, _>(|_| Client::default());
    let mut server = Server::open_clients(wire::ROOTS).unwrap();
    write(b"Cathedral: database catalog ready\n").unwrap();
    loop {
        let request = match server.next_event().unwrap() {
            Event::Request(r) => r,
            Event::Disconnected { client, .. } => {
                clients[client] = Client::default();
                continue;
            }
            Event::Memory(_, _) => unreachable!("memory attachments disabled"),
        };
        if request.control && &request.bytes[..request.len] == wire::HEALTH {
            server.reply(&request, b"ready").unwrap();
            continue;
        }
        let result = if request.control {
            Err(abi::INVALID_ARGUMENT)
        } else {
            let root = request.client.unwrap();
            clients[root].request(
                &mut catalog,
                root,
                request.incarnation,
                &request.bytes[..request.len],
                |_| {},
            )
        };
        assert!(
            !catalog.needs_reopen(),
            "database catalog requires recovery"
        );
        let reply = match result {
            Ok(record) => record.reply(),
            Err(code) => wire::error(code),
        };
        server.reply(&request, &reply).unwrap();
    }
}
