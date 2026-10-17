//! Ordinary synchronous client -> contracts/database -> platform storage/database.
//! Deliberately no database/driver dependency. Distinct fixture bytes expose any
//! accidental root alias; revision checks prevent mixing concurrent range reads.
use cathedral_platform_contracts::database as wire;
use cathedral_user_runtime::{ipc::Handle, memory::Object};
#[cfg(not(feature = "retention"))]
use cathedral_user_runtime::{link::Link, memory::Private, write};
pub(super) struct Client {
    pub(super) send: Handle,
    pub(super) receive: Handle,
}
impl Client {
    pub(super) fn call(
        &self,
        op: u64,
        expected: u64,
        offset: u64,
        len: u64,
        object: Option<&Object>,
    ) -> [u8; 64] {
        let request = wire::Request {
            op,
            expected,
            offset,
            len,
        }
        .encode();
        if let Some(object) = object {
            self.send.send_memory(&request, object).unwrap();
        } else {
            self.send.send(&request).unwrap();
        }
        let mut reply = [0; 64];
        assert_eq!(self.receive.receive(&mut reply).unwrap(), 64);
        reply
    }
    #[cfg(not(feature = "retention"))]
    fn verify(&self, revision: u64, root: u64) {
        for offset in (0..8192).step_by(wire::READ_BYTES) {
            let len = wire::READ_BYTES.min(8192 - offset);
            let reply = self.call(wire::READ, revision, offset as u64, len as u64, None);
            assert_eq!(wire::word(&reply, 0), wire::OK);
            assert_eq!(wire::word(&reply, 8), revision);
            assert_eq!(wire::word(&reply, 16), 8192);
            for (i, byte) in reply[24..24 + len].iter().enumerate() {
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
    assert!(Link::at(1).is_err(), "client sees an ungranted connection");
    let (send, receive) = Link::at(0).unwrap().connect().unwrap();
    let client = Client { send, receive };
    let head = client.call(wire::HEAD, 0, 0, 0, None);
    assert_eq!(wire::word(&head, 0), wire::OK);
    assert_eq!(
        wire::word(&client.call(wire::CLOSE, 0, 0, 0, None), 0),
        wire::INVALID
    );
    // Extra root-selection bytes are malformed, not a second namespace API.
    let mut forged = [0; 40];
    forged[..32].copy_from_slice(
        &wire::Request {
            op: wire::HEAD,
            expected: 0,
            offset: 0,
            len: 0,
        }
        .encode(),
    );
    forged[32] = 1;
    client.send.send(&forged).unwrap();
    let mut rejected = [0; 64];
    assert_eq!(client.receive.receive(&mut rejected).unwrap(), 64);
    assert_eq!(wire::word(&rejected, 0), wire::INVALID);
    let old = wire::word(&head, 8);
    assert!(old <= 1);
    if old == 1 {
        client.verify(old, root);
    }
    let next = old + 1;
    let mut pages = Private::allocate(2).unwrap();
    for (i, byte) in pages.bytes_mut().iter_mut().enumerate() {
        *byte = fixture(root, next, i);
    }
    let object = pages.freeze().unwrap();
    let reply = client.call(wire::PUBLISH, old, 0, 8192, Some(&object));
    assert_eq!(wire::word(&reply, 0), wire::OK);
    assert_eq!(wire::word(&reply, 8), next);
    let accepted = client.call(wire::HEAD, 0, 0, 0, None);
    assert_eq!(reply, accepted);
    assert_eq!(
        wire::word(&client.call(wire::PUBLISH, old, 0, 8192, Some(&object)), 0),
        wire::CONFLICT
    );
    assert_eq!(
        wire::word(
            &client.call(
                wire::PUBLISH,
                next,
                0,
                wire::MAX_BYTES as u64 + 1,
                Some(&object)
            ),
            0
        ),
        wire::TOO_LARGE
    );
    assert_eq!(
        wire::word(&client.call(wire::PUBLISH, next, 0, 8193, Some(&object)), 0),
        wire::INVALID
    );
    drop(object);
    // Distinct chunks exceed the retained-root quota, after some staging has
    // already happened. The abort must discard those chunks as well as metadata.
    let mut pages = Private::allocate(4).unwrap();
    for (i, byte) in pages.bytes_mut().iter_mut().enumerate() {
        *byte = (i / 4096 + 200) as u8;
    }
    let object = pages.freeze().unwrap();
    assert_eq!(
        wire::word(
            &client.call(wire::PUBLISH, next, 0, 16384, Some(&object)),
            0
        ),
        wire::CAPACITY
    );
    drop(object);
    assert_eq!(client.call(wire::HEAD, 0, 0, 0, None), accepted);
    assert_eq!(
        wire::word(&client.call(wire::READ, next, u64::MAX, 1, None), 0),
        wire::INVALID
    );
    assert_eq!(
        wire::word(&client.call(wire::READ, next + 1, 0, 1, None), 0),
        wire::CONFLICT
    );
    client.verify(next, root);
    write(b"Cathedral objects: client content and refusal checks passed\n").unwrap();
    next
}
