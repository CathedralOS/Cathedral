use cathedral_contracts::{storage as wire, user as abi};
use cathedral_user_runtime::{Error, server::Server, write};
pub fn run() -> Result<(), Error> {
    #[cfg(feature = "recovery-lab")]
    super::recovery::transport()?;
    let disk = cathedral_ata_pio::Ata::open().map_err(|e| Error(e as i64))?;
    let mut store = cathedral_object_store::Store::open(disk).map_err(|e| Error(e as i64))?;
    let mut server = Server::open_clients(wire::ROOTS)?;
    let mut clients = core::array::from_fn::<_, { wire::ROOTS }, _>(|_| {
        cathedral_object_store::protocol::Client::default()
    });
    #[cfg(feature = "recovery-lab")]
    let mut cut = 0;
    write(b"Cathedral: storage ready\n")?;
    loop {
        let request = server.next_request()?;
        let bytes = &request.bytes[..request.len];
        #[cfg(feature = "recovery-lab")]
        if request.control && super::recovery::request(bytes, &mut cut, &mut server, &request)? {
            continue;
        }
        if request.control && bytes == wire::HEALTH {
            server.reply(&request, b"ready")?;
            continue;
        }
        if request.control || bytes.len() != 64 {
            server.reply(&request, &wire::error(abi::INVALID_ARGUMENT))?;
            continue;
        }
        // Link position is the boot-approved root binding, stable across replacement.
        let root = request.client.unwrap();
        let result =
            clients[root].request(&mut store, root, request.incarnation, bytes, |_phase| {
                #[cfg(feature = "recovery-lab")]
                super::recovery::checkpoint(_phase, cut);
            });
        if store.needs_reopen() {
            return Err(Error(abi::IO_ERROR as i64));
        }
        let reply = match result {
            Ok(record) => record.reply(),
            Err(error) => wire::error(error),
        };
        server.reply(&request, &reply)?;
    }
}
