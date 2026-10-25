//! List the supplied realm's executables, hide the template (by name) or this
//! launcher itself (by the object its launch argument named), and copy the
//! template to create a named user.
use super::*;
use cathedral_platform_contracts::storage as store;
use cathedral_user_runtime::storage::Scope;
pub struct Realm {
    pub(super) scope: Scope<'static>,
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
        let mut executables = [(0, [0; 13]); store::ENTRIES_PER_PAGE];
        let mut found = 0;
        let (mut template, mut background) = (None, None);
        let mut after = 0;
        loop {
            let mut listing = self.scope.entries(realm, after)?;
            for index in 0..listing.len() {
                let row = listing.row(index)?;
                let (roles, label) = policy::classify(row.metadata);
                if roles & 2 != 0 {
                    template = Some(row.object);
                }
                if roles & 4 != 0 {
                    background = Some(row.object);
                }
                if roles & 1 != 0 && found < executables.len() {
                    executables[found] = (row.object, label);
                    found += 1;
                }
            }
            after = listing.last;
            if !listing.is_full() {
                break;
            }
        }
        let candidates: [u64; store::ENTRIES_PER_PAGE] = core::array::from_fn(|i| executables[i].0);
        let (ids, count) = policy::catalogue(
            self.user_picker,
            self.own,
            template.unwrap_or(0),
            background.unwrap_or(0),
            &candidates[..found],
        )
        .ok_or(store::MISSING)?;
        let mut labels = [[0; 13]; ENTRIES];
        for (label, id) in labels.iter_mut().zip(ids).take(count).skip(1) {
            *label = executables[..found]
                .iter()
                .find(|(object, _)| *object == id)
                .unwrap()
                .1;
        }
        self.ids = ids;
        self.labels = labels;
        self.count = count;
        Ok(())
    }
    /// A new entry: the resolved template copied whole, beside it. A user realm made
    /// this way shares every byte of the template until it diverges.
    pub fn copy_template(&mut self, template: u64) -> Result<(), u64> {
        let head = self
            .scope
            .call(store::Request::new(store::HEAD, template, 0, 0, 0))?;
        let label = policy::user_name(self.count);
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
