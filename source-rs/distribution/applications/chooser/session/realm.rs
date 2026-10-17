//! The realm as this program sees it: the first entries under its binding's
//! root, and the two ordinary object operations the chooser performs itself,
//! listing and the subtree copy that makes a new entry from the first.
use super::*;
use cathedral_platform_contracts::storage as store;
use cathedral_user_runtime::storage::Scope;
pub struct Realm {
    scope: Scope,
    pub ids: [u64; ENTRIES],
    pub count: usize,
}
impl Realm {
    pub fn new(pair: Pair) -> Self {
        Self {
            scope: Scope::new(pair),
            ids: [0; ENTRIES],
            count: 0,
        }
    }
    /// Acquire the current binding and list what it is rooted at.
    pub fn reload(&mut self) -> Result<(), u64> {
        self.scope.bind()?;
        let request = store::Request::new(store::LIST, self.scope.realm(), 0, 0, 0);
        let (reply, _) = self.scope.request(request, None)?;
        let words = store::unpack(&reply).ok_or(abi::IO_ERROR)?;
        if words[0] != 0 {
            return Err(words[0]);
        }
        self.count = (words[1] as usize).min(ENTRIES);
        self.ids[..self.count].copy_from_slice(&words[2..2 + self.count]);
        Ok(())
    }
    /// A new entry: the first one copied whole, beside it. A user realm made
    /// this way shares every byte of the template until it diverges.
    pub fn copy_first(&mut self) -> Result<(), u64> {
        if self.count == 0 {
            return Err(abi::DENIED);
        }
        let template = self.ids[0];
        let head = self
            .scope
            .call(store::Request::new(store::HEAD, template, 0, 0, 0))?;
        self.scope.call(store::Request::new(
            store::COPY_TREE,
            template,
            head.revision,
            self.scope.realm(),
            0,
        ))?;
        self.reload()
    }
}
