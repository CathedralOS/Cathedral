use cathedral_contracts::{storage as wire, user as abi};
use cathedral_user_runtime::{Error, server::Server, write};
pub fn run() -> Result<(), Error> {
    #[cfg(feature = "recovery-lab")]
    super::recovery::transport()?;
    let disk = cathedral_ata_pio::Ata::open().map_err(|e| Error(e as i64))?;
    let mut store = cathedral_object_store::Store::open(disk).map_err(|e| Error(e as i64))?;
    let mut server = Server::open()?;
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
        let [op, object, expected, length] = core::array::from_fn(|i| wire::word(bytes, i));
        let result = match op {
            wire::READ if expected == 0 && length == 0 && bytes[32..].iter().all(|&b| b == 0) => {
                store.read(object)
            }
            wire::REPLACE
                if length <= 32 && bytes[32 + length as usize..].iter().all(|&b| b == 0) =>
            {
                let result = store.replace(
                    object,
                    expected,
                    &bytes[32..32 + length as usize],
                    |_phase| {
                        #[cfg(feature = "recovery-lab")]
                        super::recovery::checkpoint(_phase, cut);
                    },
                );
                // Any device error makes completion ambiguous. Retire this instance;
                // reopening the journal and client reconciliation decide what committed.
                if store.needs_reopen() {
                    return Err(Error(abi::IO_ERROR as i64));
                }
                result
            }
            _ => Err(abi::INVALID_ARGUMENT),
        };
        let reply = match result {
            Ok(record) => record.reply(),
            Err(error) => wire::error(error),
        };
        server.reply(&request, &reply)?;
    }
}
