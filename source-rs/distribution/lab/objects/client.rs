//! Ordinary synchronous client -> contracts/storage -> platform storage/database.
//! Deliberately no database/driver dependency. Distinct fixture bytes expose any
//! accidental root alias; revision checks prevent mixing concurrent range reads.
use cathedral_platform_contracts::storage as wire;
use cathedral_user_runtime::{ipc::Handle, memory::Object};
#[cfg(not(feature = "retention"))]
use cathedral_user_runtime::{link::Link, memory::Private, write};
pub(super) struct Client {
    pub(super) send: Handle,
    pub(super) receive: Handle,
}
pub(super) fn packet(op: u64, object: u64, expected: u64, offset: u64, len: u64) -> wire::Request {
    wire::Request::new(op, object, expected, offset, len)
}
impl Client {
    pub(super) fn call(&self, request: wire::Request, object: Option<&Object>) -> [u8; 64] {
        let bytes = request.encode();
        if let Some(object) = object {
            self.send.send_memory(&bytes, object).unwrap();
        } else {
            self.send.send(&bytes).unwrap();
        }
        let mut reply = [0; 64];
        assert_eq!(self.receive.receive(&mut reply).unwrap(), 64);
        reply
    }
    #[cfg(not(feature = "retention"))]
    fn verify(&self, revision: u64, root: u64) {
        for offset in (0..8192).step_by(wire::READ_BYTES) {
            let len = wire::READ_BYTES.min(8192 - offset);
            let reply = self.call(
                packet(wire::READ, 1, revision, offset as u64, len as u64),
                None,
            );
            let head = wire::Reply::decode(&reply).unwrap();
            assert_eq!(
                (head.object, head.revision, head.length),
                (1, revision, 8192)
            );
            for (i, byte) in reply[32..32 + len].iter().enumerate() {
                assert_eq!(*byte, fixture(root, revision, offset + i));
            }
        }
    }
}
#[cfg(not(feature = "retention"))]
fn fixture(root: u64, revision: u64, index: usize) -> u8 {
    let byte = ((index * 17 + index / 4096 * 71 + root as usize * 43) % 251) as u8;
    if revision == 2 && index == 41 {
        byte ^ 0x80
    } else {
        byte
    }
}
#[cfg(not(feature = "retention"))]
pub fn run(root: u64) -> u64 {
    use cathedral_kernel_contracts::user as abi;
    assert!(Link::at(1).is_err(), "client sees an ungranted connection");
    let (send, receive) = Link::at(0).unwrap().connect().unwrap();
    let client = Client { send, receive };
    let listing = wire::unpack(&client.call(packet(wire::LIST, 0, 0, 0, 0), None)).unwrap();
    assert_eq!(listing[0], 0);
    assert_eq!(
        wire::word(&client.call(packet(wire::CLOSE, 0, 0, 0, 0), None), 0),
        abi::INVALID_ARGUMENT
    );
    let old = listing[1];
    assert!(old <= 1);
    if old == 1 {
        client.verify(1, root);
    }
    let next = old + 1;
    let mut pages = Private::allocate(2).unwrap();
    for (i, byte) in pages.bytes_mut().iter_mut().enumerate() {
        *byte = fixture(root, next, i);
    }
    let object = pages.freeze().unwrap();
    let reply = client.call(packet(wire::PUBLISH, old, old, 0, 8192), Some(&object));
    let head = wire::Reply::decode(&reply).unwrap();
    assert_eq!((head.object, head.revision, head.length), (1, next, 8192));
    let accepted = client.call(packet(wire::HEAD, 1, 0, 0, 0), None);
    assert_eq!(reply, accepted);
    // A scope word only narrows: naming the object itself changes nothing,
    // naming anything else denies it, and so does naming nothing that exists.
    let mut scoped = packet(wire::HEAD, 1, 0, 0, 0);
    scoped.scopes[0] = 1;
    assert_eq!(client.call(scoped, None), accepted);
    scoped.scopes[1] = 2;
    assert_eq!(wire::word(&client.call(scoped, None), 0), abi::DENIED);
    assert_eq!(
        wire::word(
            &client.call(packet(wire::PUBLISH, 1, old, 0, 8192), Some(&object)),
            0
        ),
        abi::WOULD_BLOCK
    );
    assert_eq!(
        wire::word(
            &client.call(
                packet(wire::PUBLISH, 1, next, 0, wire::MAX_BYTES as u64 + 1),
                Some(&object)
            ),
            0
        ),
        wire::TOO_LARGE
    );
    assert_eq!(
        wire::word(
            &client.call(packet(wire::PUBLISH, 1, next, 0, 8193), Some(&object)),
            0
        ),
        abi::INVALID_ARGUMENT
    );
    // New hierarchy operations cannot enumerate an ancestor or mutate the
    // delegated realm's own entry, even with a well-formed attachment.
    for (op, subject) in [
        (wire::LOOKUP, 0),
        (wire::FILTER, 0),
        (wire::SET_METADATA, 1),
        (wire::METADATA, 2),
    ] {
        let mut request = packet(op, subject, 0, 0, 0);
        request.scopes[0] = 1;
        assert_eq!(
            wire::word(&client.call(request, Some(&object)), 0),
            abi::DENIED
        );
    }
    drop(object);
    let mut pages = Private::allocate(1).unwrap();
    pages.bytes_mut()[..wire::metadata::EXECUTABLE.len()]
        .copy_from_slice(wire::metadata::EXECUTABLE);
    let filter = pages.freeze().unwrap();
    let mut request = packet(
        wire::FILTER,
        1,
        0,
        0,
        wire::metadata::EXECUTABLE.len() as u64,
    );
    request.scopes[0] = 1;
    assert_eq!(
        wire::unpack(&client.call(request, Some(&filter))).unwrap(),
        [0; 8]
    );
    drop(filter);
    assert_eq!(client.call(packet(wire::HEAD, 1, 0, 0, 0), None), accepted);
    assert_eq!(
        wire::word(
            &client.call(packet(wire::READ, 1, next, u64::MAX, 1), None),
            0
        ),
        abi::INVALID_ARGUMENT
    );
    assert_eq!(
        wire::word(&client.call(packet(wire::READ, 1, next + 1, 0, 1), None), 0),
        abi::WOULD_BLOCK
    );
    assert_eq!(
        wire::word(&client.call(packet(wire::HEAD, 2, 0, 0, 0), None), 0),
        wire::MISSING
    );
    client.verify(next, root);
    write(b"Cathedral objects: client content and refusal checks passed\n").unwrap();
    next
}
