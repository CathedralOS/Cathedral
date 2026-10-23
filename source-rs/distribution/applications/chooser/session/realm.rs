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
    /// One listing names the realm: tiles are its executables in number order,
    /// the hidden template or launcher and the background are known by name,
    /// never by listing position. The launcher's name lives in the matrix
    /// configuration, read once that entry is found. Row zero is presentation
    /// only.
    pub fn reload(&mut self) -> Result<(), u64> {
        self.scope.bind()?;
        let realm = self.scope.realm();
        let mut executables = [(0, [0; store::metadata::FIELD]); store::ENTRIES_PER_PAGE];
        let mut found = 0;
        let (mut template, mut background, mut config) = (None, None, None);
        let mut after = 0;
        loop {
            let mut listing = self.scope.entries(realm, after)?;
            for index in 0..listing.len() {
                let row = listing.row(index)?;
                match store::metadata::text(&row.metadata.name) {
                    b"Template" => template = Some(row.object),
                    b"Background" => background = Some(row.object),
                    name if name == cathedral_desktop_matrix_contracts::matrix::config::NAME => {
                        config = Some((row.object, row.revision))
                    }
                    _ => (),
                }
                if store::metadata::text(&row.metadata.media_type) == store::metadata::EXECUTABLE
                    && found < executables.len()
                {
                    executables[found] = (row.object, row.metadata.name);
                    found += 1;
                }
            }
            after = listing.last;
            if !listing.is_full() {
                break;
            }
        }
        let hidden = if self.user_picker {
            template.ok_or(store::MISSING)?
        } else {
            let (object, revision) = config.ok_or(store::MISSING)?;
            let (reply, extra) = self.scope.request(
                store::Request::new(store::READ, object, revision, 8, 32),
                None,
            )?;
            drop(extra);
            store::Reply::decode(&reply)?;
            let launcher: [u8; 32] = reply[32..].try_into().unwrap();
            executables[..found]
                .iter()
                .find(|(_, name)| *name == launcher)
                .map(|(id, _)| *id)
                .ok_or(store::MISSING)?
        };
        let limit = if self.user_picker {
            USER_ENTRIES
        } else {
            ENTRIES
        };
        self.ids = [0; ENTRIES];
        self.labels = [[0; 13]; ENTRIES];
        self.ids[0] = hidden;
        self.count = 1;
        for (id, name) in &executables[..found] {
            if *id == hidden || Some(*id) == background {
                continue;
            }
            if self.count == limit {
                break;
            }
            let mut label = [0; 13];
            for (to, from) in label.iter_mut().zip(store::metadata::text(name)) {
                *to = from.to_ascii_uppercase();
            }
            self.ids[self.count] = *id;
            self.labels[self.count] = label;
            self.count += 1;
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
