use super::*;
use cathedral_object_store::protocol::Client;
use cathedral_storage_engine::redb::{Database, backends::InMemoryBackend};
fn packet(op: u64, object: u64, expected: u64, bytes: &[u8]) -> [u8; 64] {
    let mut out = [0; 64];
    for (chunk, value) in
        out[..32]
            .chunks_exact_mut(8)
            .zip([op, object, expected, bytes.len() as u64])
    {
        chunk.copy_from_slice(&value.to_le_bytes());
    }
    out[32..32 + bytes.len()].copy_from_slice(bytes);
    out
}
#[test]
fn catalog_transactions_keep_the_same_connection_and_root_semantics_on_content_storage() {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();
    let store = Store::open(&db).unwrap();
    let mut catalog = Catalog::open(&store).unwrap();
    let mut client = Client::default();
    for (op, obj, expected, bytes) in [
        (wire::BEGIN, 0, 0, &b""[..]),
        (wire::STAGE_CREATE, 1, 0, &b"one"[..]),
        (wire::STAGE_CREATE, 2, 0, &b"two"[..]),
        (wire::COMMIT, 0, 0, &b""[..]),
    ] {
        client
            .request(
                &mut catalog,
                0,
                1,
                &packet(op, obj, expected, bytes),
                |_| {},
            )
            .unwrap();
    }
    assert_eq!(&catalog.read(0, 1).unwrap().bytes[..3], b"one");
    assert_eq!(catalog.root(1).unwrap().generation, 0);
    assert_eq!(
        client
            .request(
                &mut catalog,
                0,
                1,
                &packet(wire::REPLACE, 1, 0, b"bad"),
                |_| {}
            )
            .err(),
        Some(abi::WOULD_BLOCK)
    );
    client
        .request(&mut catalog, 0, 1, &packet(wire::BEGIN, 0, 1, b""), |_| {})
        .unwrap();
    client
        .request(
            &mut catalog,
            0,
            1,
            &packet(wire::STAGE_DELETE, 2, 0, b""),
            |_| {},
        )
        .unwrap();
    assert_eq!(
        client
            .request(&mut catalog, 0, 2, &packet(wire::COMMIT, 0, 0, b""), |_| {})
            .err(),
        Some(abi::BAD_HANDLE)
    );
    assert_eq!(&catalog.read(0, 2).unwrap().bytes[..3], b"two");
    for revision in 2..=300 {
        client
            .request(
                &mut catalog,
                0,
                2,
                &packet(wire::REPLACE, 1, revision - 1, &revision.to_le_bytes()),
                |_| {},
            )
            .unwrap();
    }
    let reopened = Catalog::open(&store).unwrap();
    assert_eq!(reopened.root(0).unwrap().generation, 300);
    assert_eq!(
        &reopened.read(0, 1).unwrap().bytes[..8],
        &300u64.to_le_bytes()
    );
    assert_eq!(reopened.root(1).unwrap().generation, 0);
}
