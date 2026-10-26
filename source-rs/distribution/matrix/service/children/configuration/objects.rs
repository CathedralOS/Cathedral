// Restore remembered bindings and read immutable realm objects.
use super::*;
impl Children {
    pub(in crate::service::children) fn restore_storage(
        &mut self,
        parent: &mut Parent,
        server: &Server,
    ) -> Result<(), Error> {
        for slot in 0..CHILDREN {
            if self.slots[slot].is_some()
                && let Some((source, scope)) = self.saved_storage[slot]
            {
                match parent.storage_grant(server, slot, source, scope)? {
                    Ok(()) => {}
                    // storage_grant refreshes upstream state before checking.
                    // A matching live grant needs no new binding incarnation.
                    Err(code)
                        if code == abi::BUSY
                            && parent.storage_binding(server, slot) == Some((source, scope)) => {}
                    Err(code) => return Err(Error(code as i64)),
                }
            }
        }
        Ok(())
    }
}
/// Read a realm object whole in one request. `expected` is the exact length
/// a fixed record must have; a longer or shorter object is refused.
pub(in crate::service::children) fn read(
    scope: &mut Scope,
    id: u64,
    expected: Option<usize>,
) -> Result<cathedral_user_runtime::memory::Object, Error> {
    let len = expected.map_or(0, |n| n as u64);
    let (reply, object) = scope
        .request(storage::Request::new(storage::BULK, id, 0, 0, len), None)
        .map_err(|c| Error(c as i64))?;
    let reply = storage::Reply::decode(&reply).map_err(|c| Error(c as i64))?;
    if expected.is_some_and(|n| reply.length != n as u64) {
        return Err(Error(abi::INVALID_ARGUMENT as i64));
    }
    object.ok_or(Error(abi::IO_ERROR as i64))
}
