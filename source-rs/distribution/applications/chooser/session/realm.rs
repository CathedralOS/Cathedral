//! Query executable entries in the supplied realm, resolve the hidden template
//! or launcher by path, and copy the template to create a named user.
use super::*;
use cathedral_platform_contracts::storage as store;
use cathedral_user_runtime::storage::Scope;
pub struct Realm {
    pub(super) scope: Scope,
    pub ids: [u64; ENTRIES],
    pub labels: [[u8; 13]; ENTRIES],
    pub count: usize,
    pub(super) user_picker: bool,
}
impl Realm {
    pub fn new(pair: Pair, user_picker: bool) -> Self {
        Self {
            scope: Scope::new(pair),
            ids: [0; ENTRIES],
            labels: [[0; 13]; ENTRIES],
            count: 0,
            user_picker,
        }
    }
    /// Query executable entries; the hidden template/launcher is identified by
    /// its path, never its listing position. Row zero is presentation only.
    pub fn reload(&mut self) -> Result<(), u64> {
        self.scope.bind()?;
        let hidden = if self.user_picker {
            self.scope.lookup(b"Template")?
        } else {
            let config = self
                .scope
                .lookup(cathedral_desktop_matrix_contracts::matrix::config::NAME)?;
            let head = self
                .scope
                .call(store::Request::new(store::HEAD, config, 0, 0, 0))?;
            let (reply, extra) = self.scope.request(
                store::Request::new(store::READ, config, head.revision, 8, 32),
                None,
            )?;
            drop(extra);
            store::Reply::decode(&reply)?;
            let name: [u8; 32] = reply[32..].try_into().unwrap();
            self.scope.lookup(store::metadata::text(&name))?
        };
        let background = if self.user_picker {
            None
        } else {
            self.scope.lookup(b"Background").ok()
        };
        self.ids = [0; ENTRIES];
        self.ids[0] = hidden;
        self.count = 1;
        let mut after = 0;
        loop {
            let mut found = [0; store::LIST_IDS];
            let count = self
                .scope
                .filter(store::metadata::EXECUTABLE, after, &mut found)?;
            if count == 0 {
                break;
            }
            for &id in &found[..count] {
                after = id;
                if id == hidden || Some(id) == background {
                    continue;
                }
                if self.count
                    == if self.user_picker {
                        USER_ENTRIES
                    } else {
                        ENTRIES
                    }
                {
                    return Ok(());
                }
                // The filter already proved the type; only the name is needed.
                let name = self.scope.name(id)?;
                let name = store::metadata::text(&name);
                let mut label = [0; 13];
                for (to, from) in label.iter_mut().zip(name) {
                    *to = from.to_ascii_uppercase();
                }
                self.ids[self.count] = id;
                self.labels[self.count] = label;
                self.count += 1;
            }
        }
        Ok(())
    }
    /// A new entry: the resolved template copied whole, beside it. A user realm made
    /// this way shares every byte of the template until it diverges.
    pub fn copy_first(&mut self) -> Result<(), u64> {
        if self.count == 0 || self.count == USER_ENTRIES {
            return Err(abi::DENIED);
        }
        let template = self.ids[0];
        let head = self
            .scope
            .call(store::Request::new(store::HEAD, template, 0, 0, 0))?;
        let label = labels::numbered(b"User ", self.count);
        let len = label.iter().position(|b| *b == 0).unwrap_or(label.len());
        self.scope.attached(
            store::Request::new(
                store::COPY_TREE,
                template,
                head.revision,
                self.scope.realm(),
                len as u64,
            ),
            &label[..len],
        )?;
        self.reload()
    }
}
