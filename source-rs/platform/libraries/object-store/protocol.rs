//! Data protocol dispatch. The service supplies a trusted root and connection identity;
//! neither is deserialized from client bytes. Each connection stages at most two changes.
use super::{Store, catalog::Change};
use cathedral_contracts::{block::Device, storage as wire, user as abi};
#[derive(Default)]
pub struct Client {
    incarnation: u64,
    transaction: Option<Transaction>,
}
struct Transaction {
    expected: u64,
    changes: [Change; wire::CHANGES],
    length: usize,
}
impl Client {
    pub fn request<D: Device>(
        &mut self,
        store: &mut Store<D>,
        root: usize,
        incarnation: u64,
        bytes: &[u8],
        hook: impl FnMut(u8),
    ) -> Result<wire::Record, u64> {
        if self.incarnation != incarnation {
            self.incarnation = incarnation;
            self.transaction = None;
        }
        if bytes.len() != 64 {
            return Err(abi::INVALID_ARGUMENT);
        }
        let [op, object, expected, length] = core::array::from_fn(|i| wire::word(bytes, i));
        if length > 32 || bytes[32 + length as usize..].iter().any(|&b| b != 0) {
            return Err(abi::INVALID_ARGUMENT);
        }
        let payload = &bytes[32..32 + length as usize];
        match op {
            wire::READ if expected == 0 && length == 0 => store.read(root, object),
            wire::LIST if object == 0 && expected == 0 && length == 0 => {
                Ok(store.root(root)?.list())
            }
            wire::CREATE | wire::REPLACE | wire::DELETE => {
                if self.transaction.is_some() {
                    return Err(abi::BUSY);
                }
                let change = Change::value(op, object, payload)?;
                store.transact(root, expected, &[change], hook)?;
                if op == wire::DELETE {
                    Ok(store.root(root)?.list())
                } else {
                    store.read(root, object)
                }
            }
            wire::BEGIN if object == 0 && length == 0 => {
                let listing = store.root(root)?.list();
                if listing.generation != expected {
                    return Err(abi::WOULD_BLOCK);
                }
                if self.transaction.is_some() {
                    return Err(abi::BUSY);
                }
                self.transaction = Some(Transaction {
                    expected,
                    changes: [Change::Delete(0); wire::CHANGES],
                    length: 0,
                });
                Ok(listing)
            }
            wire::STAGE_CREATE | wire::STAGE_REPLACE | wire::STAGE_DELETE if expected == 0 => {
                let change = Change::value(op, object, payload)?;
                let transaction = self.transaction.as_mut().ok_or(abi::BAD_HANDLE)?;
                if transaction.length == wire::CHANGES {
                    return Err(abi::NO_MEMORY);
                }
                if transaction.changes[..transaction.length]
                    .iter()
                    .any(|old| old.object() == object)
                {
                    return Err(abi::INVALID_ARGUMENT);
                }
                transaction.changes[transaction.length] = change;
                transaction.length += 1;
                Ok(store.root(root)?.list())
            }
            wire::COMMIT if object == 0 && expected == 0 && length == 0 => {
                let transaction = self.transaction.take().ok_or(abi::BAD_HANDLE)?;
                store.transact(
                    root,
                    transaction.expected,
                    &transaction.changes[..transaction.length],
                    hook,
                )
            }
            wire::ABORT if object == 0 && expected == 0 && length == 0 => {
                self.transaction = None;
                Ok(store.root(root)?.list())
            }
            _ => Err(abi::INVALID_ARGUMENT),
        }
    }
}
