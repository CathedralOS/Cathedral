//! Resolve Matrix within the supplied realm; check its codec and revision.
use super::*;
use cathedral_desktop_matrix_contracts::matrix::config;
use cathedral_platform_contracts::storage;
pub(super) fn load(scope: &mut Scope) -> Result<(u64, u64, Config), u64> {
    scope.bind()?;
    let object = scope.lookup(config::NAME)?;
    let head = scope.call(storage::Request::new(storage::HEAD, object, 0, 0, 0))?;
    if head.length != config::SIZE as u64 {
        return Err(abi::INVALID_ARGUMENT);
    }
    // The launcher has no reader-page grant. This small record uses bounded
    // inline reads, all pinned to the same revision, rather than adding authority.
    let mut bytes = [0; config::SIZE];
    for (index, chunk) in bytes.chunks_mut(storage::READ_BYTES).enumerate() {
        let (reply, extra) = scope.request(
            storage::Request::new(
                storage::READ,
                object,
                head.revision,
                (index * storage::READ_BYTES) as u64,
                chunk.len() as u64,
            ),
            None,
        )?;
        drop(extra);
        storage::Reply::decode(&reply)?;
        chunk.copy_from_slice(&reply[32..32 + chunk.len()]);
    }
    let saved = Config::decode(&bytes).ok_or(abi::INVALID_ARGUMENT)?;
    Ok((object, head.revision, saved))
}
pub(super) fn save(
    scope: &mut Scope,
    object: u64,
    revision: u64,
    config: Config,
) -> Result<u64, u64> {
    let bytes = config.encode();
    let reply = scope.attached(
        storage::Request::new(storage::PUBLISH, object, revision, 0, bytes.len() as u64),
        &bytes,
    )?;
    Ok(storage::Reply::decode(&reply)?.revision)
}
