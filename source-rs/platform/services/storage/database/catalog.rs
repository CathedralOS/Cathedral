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
    #[cfg(feature = "recovery-lab")]
    let mut gate = super::gate::Gate::default();
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
        #[cfg(feature = "recovery-lab")]
        if request.control {
            if gate.arm(&mut server, &request) {
                continue;
            }
            if request.bytes[..request.len] == [0xf4] {
                use cathedral_object_store::protocol::Backend;
                let mut reply = [0; 16];
                for root in 0..2 {
                    reply[root * 8..root * 8 + 8]
                        .copy_from_slice(&catalog.root(root).unwrap().generation.to_le_bytes());
                }
                server.reply(&request, &reply).unwrap();
                continue;
            }
        }
        if request.control && &request.bytes[..request.len] == wire::HEALTH {
            server.reply(&request, b"ready").unwrap();
            continue;
        }
        let result = if request.control {
            Err(abi::INVALID_ARGUMENT)
        } else {
            let root = request.client.unwrap();
            let op = if request.len == 64 {
                wire::word(&request.bytes, 0)
            } else {
                0
            };
            #[cfg(feature = "trace-storage")]
            cathedral_user_runtime::time::storage_trace("provider-begin", root as u64, op);
            #[cfg(feature = "trace-storage")]
            let io_before = cathedral_ata_pio::metrics::snapshot();
            #[cfg(feature = "recovery-lab")]
            gate.wait(1, op, root);
            let result = clients[root].request(
                &mut catalog,
                root,
                request.incarnation,
                &request.bytes[..request.len],
                |_phase| {
                    #[cfg(feature = "trace-storage")]
                    cathedral_user_runtime::time::storage_trace(
                        [
                            "collect-begin",
                            "publish-begin",
                            "prepared",
                            "commit-begin",
                            "committed",
                        ][_phase as usize],
                        root as u64,
                        op,
                    );
                },
            );
            #[cfg(feature = "recovery-lab")]
            gate.wait(2, op, root);
            #[cfg(feature = "trace-storage")]
            {
                let after = cathedral_ata_pio::metrics::snapshot();
                let delta: [u64; 5] = core::array::from_fn(|i| after[i].wrapping_sub(io_before[i]));
                cathedral_user_runtime::time::storage_sample(
                    "ata-io",
                    &[
                        root as u64,
                        op,
                        delta[0],
                        delta[1],
                        delta[2],
                        delta[3],
                        delta[4],
                    ],
                );
                cathedral_user_runtime::time::storage_trace("provider-end", root as u64, op);
            }
            let _ = op;
            result
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
