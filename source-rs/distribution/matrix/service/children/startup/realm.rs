//! Build missing names inside the supplied realm; never copy an ancestor or
//! overwrite existing programs/data. Every operation is atomic and retryable.
use cathedral_desktop_matrix_contracts::matrix::config;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::storage as store;
use cathedral_user_runtime::storage::Scope;
const SEEDS: [&[u8]; 5] = [b"User", b"Chooser", b"Notes", b"Colors", b"Background"];
enum Source<'a> {
    Named(&'a [u8]),
    Realm,
}
pub fn prepare(scope: &mut Scope, keys: u64) -> Result<(u64, u64), u64> {
    // The same seed is copied into the workspace and its Nested template.
    // Resolve lazily, once in this preparation only. COPY still checks the
    // source revision, so a concurrent source edit refuses instead of drifting.
    let mut sources = [None; SEEDS.len()];
    let picker = scope.lookup(b"Chooser")?;
    let users = create(scope, b"Users", scope.realm(), b"Users", &[])?;
    let template = copy(
        scope,
        b"Users/Template",
        users,
        b"Template",
        Source::Named(b"User"),
        false,
        &mut sources,
    )?;
    for (source, name, path) in [
        (
            b"Chooser".as_slice(),
            b"Desktop".as_slice(),
            b"Users/Template/Desktop".as_slice(),
        ),
        (b"Background", b"Background", b"Users/Template/Background"),
        (b"Notes", b"Notes", b"Users/Template/Notes"),
        (b"Colors", b"Colors", b"Users/Template/Colors"),
    ] {
        copy(
            scope,
            path,
            template,
            name,
            Source::Named(source),
            true,
            &mut sources,
        )?;
    }
    let nested = copy(
        scope,
        b"Users/Template/Nested",
        template,
        b"Nested",
        Source::Realm,
        false,
        &mut sources,
    )?;
    for name in SEEDS {
        let mut path = [0; 64];
        let prefix = b"Users/Template/Nested/";
        path[..prefix.len()].copy_from_slice(prefix);
        path[prefix.len()..prefix.len() + name.len()].copy_from_slice(name);
        copy(
            scope,
            &path[..prefix.len() + name.len()],
            nested,
            name,
            Source::Named(name),
            true,
            &mut sources,
        )?;
    }
    let mut launcher = [0; 32];
    launcher[..7].copy_from_slice(b"Desktop");
    let config = config::Config {
        launcher,
        launcher_pages: 96,
        app_pages: 384,
        app_readers: 768,
        margin: 8,
        launcher_height: 80,
        gap: 8,
        app_surfaces: 64,
        keys,
    };
    create(
        scope,
        b"Users/Template/Matrix",
        template,
        b"Matrix",
        &config.encode(),
    )?;
    Ok((picker, users))
}
fn existing(scope: &mut Scope, path: &[u8]) -> Result<Option<u64>, u64> {
    match scope.lookup(path) {
        Ok(id) => Ok(Some(id)),
        Err(store::MISSING) => Ok(None),
        Err(c) => Err(c),
    }
}
fn copy(
    scope: &mut Scope,
    path: &[u8],
    parent: u64,
    name: &[u8],
    source: Source<'_>,
    tree: bool,
    sources: &mut [Option<store::Reply>; SEEDS.len()],
) -> Result<u64, u64> {
    if let Some(id) = existing(scope, path)? {
        return Ok(id);
    }
    // Existing templates need no source lookup. On a cold reopen each avoided
    // lookup also avoids walking the entire upstream confinement/IPC chain.
    let head = match source {
        Source::Named(path) => {
            let index = SEEDS.iter().position(|name| *name == path).unwrap();
            if let Some(head) = sources[index] {
                head
            } else {
                let id = scope.lookup(path)?;
                let head = scope.call(store::Request::new(store::HEAD, id, 0, 0, 0))?;
                sources[index] = Some(head);
                head
            }
        }
        Source::Realm => scope.call(store::Request::new(store::HEAD, scope.realm(), 0, 0, 0))?,
    };
    match scope.attached(
        store::Request::new(
            if tree { store::COPY_TREE } else { store::COPY },
            head.object,
            head.revision,
            parent,
            name.len() as u64,
        ),
        name,
    ) {
        Ok(bytes) => Ok(store::Reply::decode(&bytes)?.object),
        Err(abi::WOULD_BLOCK) => scope.lookup(path),
        Err(c) => Err(c),
    }
}
fn create(
    scope: &mut Scope,
    path: &[u8],
    parent: u64,
    name: &[u8],
    data: &[u8],
) -> Result<u64, u64> {
    if let Some(id) = existing(scope, path)? {
        return Ok(id);
    }
    let metadata = store::metadata::Metadata::new(name, b"")
        .ok_or(abi::INVALID_ARGUMENT)?
        .encode();
    let mut bytes = [0; store::metadata::BYTES + config::SIZE];
    bytes[..metadata.len()].copy_from_slice(&metadata);
    bytes[metadata.len()..metadata.len() + data.len()].copy_from_slice(data);
    match scope.attached(
        store::Request::new(
            store::CREATE_NAMED,
            0,
            0,
            parent,
            (metadata.len() + data.len()) as u64,
        ),
        &bytes[..metadata.len() + data.len()],
    ) {
        Ok(bytes) => Ok(store::Reply::decode(&bytes)?.object),
        Err(abi::WOULD_BLOCK) => scope.lookup(path),
        Err(c) => Err(c),
    }
}
