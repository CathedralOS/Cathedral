//! Persistent names belong to Split's realm. Children never receive its code
//! object, parent realm, or their sibling's root.
use cathedral_platform_contracts::storage as wire;
use cathedral_user_runtime::{Error, storage::Scope};
pub fn open(scope: &mut Scope) -> Result<[u64; 2], Error> {
    let mut roots = [0; 2];
    for (slot, name) in [b"Left".as_slice(), b"Right"].into_iter().enumerate() {
        roots[slot] = match scope.lookup(name) {
            Ok(id) => id,
            Err(wire::MISSING) => {
                let created = scope
                    .call(wire::Request::new(wire::PUBLISH, 0, 0, scope.realm(), 0))
                    .map_err(|c| Error(c as i64))?;
                let metadata = wire::metadata::Metadata::new(name, b"").unwrap().encode();
                scope
                    .attached(
                        wire::Request::new(
                            wire::SET_METADATA,
                            created.object,
                            created.revision,
                            0,
                            metadata.len() as u64,
                        ),
                        &metadata,
                    )
                    .map_err(|c| Error(c as i64))?;
                created.object
            }
            Err(code) => return Err(Error(code as i64)),
        };
    }
    Ok(roots)
}
