//! Persistent private realm and funding for the same matrix one level down.
use super::*;
impl Children {
    /// Run this same matrix beneath a private copy of its realm. No ambient
    /// executable lookup: the readable realm root is our supplied code object.
    pub fn deepen(
        &mut self,
        storage: &mut crate::session::storage::Storage,
        server: &mut Server,
    ) -> Result<(), Error> {
        let mut scope = storage.idle_scope().ok_or(Error(abi::BUSY as i64))?;
        scope.bind().map_err(|c| Error(c as i64))?;
        let root = match scope.lookup(b"Nested") {
            Ok(id) => id,
            Err(storage::MISSING) => {
                let head = scope
                    .call(storage::Request::new(storage::HEAD, scope.realm(), 0, 0, 0))
                    .map_err(|c| Error(c as i64))?;
                let reply = scope
                    .attached(
                        storage::Request::new(
                            storage::COPY,
                            scope.realm(),
                            head.revision,
                            scope.realm(),
                            6,
                        ),
                        b"Nested",
                    )
                    .map_err(|c| Error(c as i64))?;
                storage::Reply::decode(&reply)
                    .map_err(|c| Error(c as i64))?
                    .object
            }
            Err(code) => return Err(Error(code as i64)),
        };
        // Populate only missing names. A stopped setup resumes by path without
        // overwriting a nested program or its saved notes. Each copy is atomic.
        for name in [b"Notes".as_slice(), b"Left", b"Right"] {
            let mut path = [0; 32];
            path[..7].copy_from_slice(b"Nested/");
            path[7..7 + name.len()].copy_from_slice(name);
            match scope.lookup(&path[..7 + name.len()]) {
                Ok(_) => continue,
                Err(storage::MISSING) => (),
                Err(code) => return Err(Error(code as i64)),
            }
            let source = scope.lookup(name).map_err(|c| Error(c as i64))?;
            let head = scope
                .call(storage::Request::new(storage::HEAD, source, 0, 0, 0))
                .map_err(|c| Error(c as i64))?;
            scope
                .attached(
                    storage::Request::new(
                        storage::COPY_TREE,
                        source,
                        head.revision,
                        root,
                        name.len() as u64,
                    ),
                    name,
                )
                .map_err(|c| Error(c as i64))?;
        }
        let (code, granted) = image::read(&mut scope, root)?;
        let launch = Executable::from_object(&code, 0)?.reserve(
            Limits {
                // Two 40-page Notes grants plus two 14-page header versions
                // during replacement. The enclosing 144-page grant still funds
                // its own retained/replacement headers alongside this loan.
                private_pages: 112,
                reader_pages: 64,
                channels: 15,
            },
            2,
            abi::MAX_ADMISSION_FRAMES as u64,
        )?;
        let child = match admission::spawn_brokered(launch, 0, &[], granted) {
            Ok(child) => child,
            Err(error) => {
                launch.release()?;
                return Err(error);
            }
        };
        server.install(0, child.data)?;
        server.install(2, child.storage.unwrap())?;
        self.launches[0] = Some(launch);
        self.slots[0] = Some(child);
        self.uncertain = [false; 2];
        let realm = scope.realm();
        drop(scope);
        storage.reset_roots(realm, [root, 0]);
        storage.install(0, server.incarnation(2).unwrap());
        cathedral_user_runtime::write(
            b"Cathedral Split: nested matrix admitted in separate realm\n",
        )?;
        Ok(())
    }
}
