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
                let metadata = wire::metadata::Metadata::new(name, b"").unwrap();
                match scope.create_named(metadata, &[]) {
                    Ok(created) => created.object,
                    // Another creator won. Resolve the standing name, without
                    // overwriting it. A lost reply instead aborts this launch;
                    // the next launch starts with LOOKUP on a fresh stream.
                    Err(cathedral_kernel_contracts::user::WOULD_BLOCK) => {
                        scope.lookup(name).map_err(|c| Error(c as i64))?
                    }
                    Err(code) => return Err(Error(code as i64)),
                }
            }
            Err(code) => return Err(Error(code as i64)),
        };
    }
    Ok(roots)
}
