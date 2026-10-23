//! List the supplied realm's executables, hide the template (by name) or this
//! launcher itself (by the object its launch argument named), and copy the
//! template to create a named user.
use super::*;
use cathedral_platform_contracts::storage as store;
use cathedral_user_runtime::storage::Scope;
pub struct Realm {
    pub(super) scope: Scope,
    pub ids: [u64; ENTRIES],
    pub labels: [[u8; 13]; ENTRIES],
    pub count: usize,
    pub(super) user_picker: bool,
    /// This program's own object, from its launch argument.
    own: u64,
}
impl Realm {
    pub fn new(pair: Pair, user_picker: bool, own: u64) -> Self {
        Self {
            scope: Scope::new(pair),
            ids: [0; ENTRIES],
            labels: [[0; 13]; ENTRIES],
            count: 0,
            user_picker,
            own,
        }
    }
    /// One listing names the realm: tiles are its executables in number order,
    /// the hidden template and the background are known by name, never by
    /// listing position, and a launcher hides the object its launch argument
    /// named as itself. Row zero is presentation only.
    pub fn reload(&mut self) -> Result<(), u64> {
        self.scope.bind()?;
        let realm = self.scope.realm();
        let mut executables = [(0, [0; store::metadata::FIELD]); store::ENTRIES_PER_PAGE];
        let mut found = 0;
        let (mut template, mut background) = (None, None);
        let mut after = 0;
        loop {
            let mut listing = self.scope.entries(realm, after)?;
            for index in 0..listing.len() {
                let row = listing.row(index)?;
                match store::metadata::text(&row.metadata.name) {
                    b"Template" => template = Some(row.object),
                    b"Background" => background = Some(row.object),
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
        } else if executables[..found].iter().any(|(id, _)| *id == self.own) {
            self.own
        } else {
            // A launcher must be one of the realm's executables; an owner that
            // named no object, or another one, has not configured a launcher.
            return Err(store::MISSING);
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
