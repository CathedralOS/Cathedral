//! Connection -> private root -> bounded request -> transaction -> reply.
//! Protocol lives in platform/contracts/database.rs. Root slot bindings are
//! explicit lab composition policy, stable across reconnection, not global IDs.
use cathedral_database_objects::{Error, Head, Stage, Store};
use cathedral_platform_contracts::database as wire;
use cathedral_user_runtime::{
    memory::Object,
    server::{Event, Server},
};

pub fn serve(store: &Store<'_>) {
    let mut server = Server::open_clients(2).unwrap();
    server.enable_memory();
    loop {
        let (request, mut object) = match server.next_event().unwrap() {
            Event::Request(request) => (request, None),
            Event::Memory(request, object) => (request, Some(object)),
            Event::Disconnected { .. } => continue,
        };
        let command = wire::Request::decode(&request.bytes[..request.len]);
        if request.control
            && object.is_none()
            && command.is_some_and(|r| {
                r.op == wire::CLOSE && r.expected == 0 && r.offset == 0 && r.len == 0
            })
        {
            server.reply(&request, &[0; 64]).unwrap();
            return;
        }
        let mut response = [0; 64];
        let result = if request.control {
            Err(Error::Invalid)
        } else {
            dispatch(
                store,
                request.client.unwrap() as u8,
                command,
                object.as_mut(),
                &mut response,
            )
        };
        // Release the reader grant before replying: synchronous callers can
        // release/reuse their pages immediately after observing completion.
        drop(object);
        let status = match result {
            Ok(()) => wire::OK,
            Err(Error::Invalid) => wire::INVALID,
            Err(Error::TooLarge) => wire::TOO_LARGE,
            Err(Error::Conflict) => wire::CONFLICT,
            Err(Error::Capacity) => wire::CAPACITY,
            // I/O/corruption isn't a clean refusal. Stop; reconciliation must
            // cold-open the engine. Never continue on possibly poisoned storage.
            Err(error) => panic!("private-object engine failed: {:?}", error),
        };
        if status != wire::OK {
            response.fill(0);
        }
        response[..8].copy_from_slice(&status.to_le_bytes());
        server.reply(&request, &response).unwrap();
    }
}

fn dispatch(
    store: &Store<'_>,
    root: u8,
    command: Option<wire::Request>,
    object: Option<&mut Object>,
    response: &mut [u8; 64],
) -> Result<(), Error> {
    let r = command.ok_or(Error::Invalid)?;
    let head = match r.op {
        wire::HEAD if object.is_none() && r.expected == 0 && r.offset == 0 && r.len == 0 => {
            let head = store.read(root, 0, 0, &mut [])?;
            response[24..56].copy_from_slice(&head.manifest);
            head
        }
        wire::READ if object.is_none() && r.expected != 0 && r.len <= wire::READ_BYTES as u64 => {
            let offset = usize::try_from(r.offset).map_err(|_| Error::Invalid)?;
            store.read(
                root,
                r.expected,
                offset,
                &mut response[24..24 + r.len as usize],
            )?
        }
        wire::PUBLISH if r.offset == 0 => {
            // Check logical and backing sizes before mapping or hashing input.
            if r.len > wire::MAX_BYTES as u64 {
                return Err(Error::TooLarge);
            }
            let object = object.ok_or(Error::Invalid)?;
            if object.len() > wire::MAX_BYTES || object.len() < r.len as usize {
                return Err(Error::Invalid);
            }
            let bytes = object.map().map_err(|_| Error::Invalid)?;
            let head =
                store.publish_observed(root, r.expected, &bytes[..r.len as usize], |stage| {
                    crate::metrics::report(match stage {
                        Stage::Prepared => "prepared",
                        Stage::Committed => "committed",
                    })
                })?;
            response[24..56].copy_from_slice(&head.manifest);
            head
        }
        _ => return Err(Error::Invalid),
    };
    describe(response, head);
    Ok(())
}
fn describe(response: &mut [u8; 64], head: Head) {
    response[8..16].copy_from_slice(&head.revision.to_le_bytes());
    response[16..24].copy_from_slice(&(head.len as u64).to_le_bytes());
}
