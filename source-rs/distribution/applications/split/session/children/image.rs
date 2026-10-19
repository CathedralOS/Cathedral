//! Read a supplied executable and validate its declared service boundary.
use super::*;
use cathedral_user_runtime::memory::Object;
pub(super) fn read(scope: &mut Scope, id: u64) -> Result<(Object, u64), Error> {
    let head = scope
        .call(storage::Request::new(storage::HEAD, id, 0, 0, 0))
        .map_err(|c| Error(c as i64))?;
    let (reply, object) = scope
        .request(
            storage::Request::new(storage::BULK, id, head.revision, 0, head.length),
            None,
        )
        .map_err(|c| Error(c as i64))?;
    storage::Reply::decode(&reply).map_err(|c| Error(c as i64))?;
    let mut code = object.ok_or(Error(abi::IO_ERROR as i64))?;
    let declaration = imports::from_elf(code.map()?)
        .map_err(|_| Error(abi::BAD_EXECUTABLE as i64))?
        .ok_or(Error(abi::BAD_EXECUTABLE as i64))?;
    code.unmap()?;
    let granted = imports::bind(declaration.entries(), imports::ALL)
        .map_err(|_| Error(imports::MISSING as i64))?;
    if !declaration.brokered || granted != imports::ALL {
        return Err(Error(abi::DENIED as i64));
    }
    Ok((code, granted))
}
