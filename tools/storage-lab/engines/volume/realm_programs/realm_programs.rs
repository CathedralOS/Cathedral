// Add named entries to recognized stock realms without reserving object IDs:
// programs, empty folders, a stock matrix configuration, or a subtree copy of
// an entry installed earlier in the same realm. Existing customized entries
// block that path and its descendants. Only program copies matching the
// installer receipt's previous/current bytes are refreshed; folders,
// configurations and copies are made once and never replaced. One transaction
// and the realm-program backup cover code, metadata and the old stock budget upgrade.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use cathedral_desktop_matrix_contracts::matrix::config::Config;
use cathedral_platform_contracts::storage::metadata::{self, Metadata};
use std::collections::BTreeMap;
type Records = BTreeMap<Vec<u8>, Vec<u8>>;
use entries::{named, put, resolve};
/// What a realm entry's source names: a program file, or a directive.
enum Source {
    Program { data: Vec<u8>, current: Vec<u8> },
    Folder,
    MatrixConfig,
    Copy(String),
}
impl Source {
    fn parse(argument: &str) -> Result<Self> {
        Ok(match argument {
            "@folder" => Self::Folder,
            "@matrix-config" => Self::MatrixConfig,
            copy if copy.starts_with("@copy=") => Self::Copy(copy["@copy=".len()..].to_owned()),
            directive if directive.starts_with('@') => {
                return Err(format!("unknown realm directive {directive}").into());
            }
            file => {
                let data = std::fs::read(file)?;
                if !data.starts_with(b"\x7fELF") {
                    return Err("realm addition is not a program".into());
                }
                let mut content = Store::new(Policy {
                    chunk_bytes: cathedral_database_objects::CHUNK_BYTES,
                    inline_limit: 128,
                    layout: Layout::Flat,
                });
                let version = content.put(&data);
                let current = content.records(&version)[b"head".as_slice()].clone();
                Self::Program { data, current }
            }
        })
    }
    /// The receipt names a program by its content manifest; directives carry none.
    fn receipt(&self) -> String {
        match self {
            Self::Program { current, .. } => current.iter().map(|b| format!("{b:02x}")).collect(),
            _ => "-".to_owned(),
        }
    }
}
pub fn install(rows: &mut Records, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    if !args.len().is_multiple_of(3) {
        return Err("realm entries are PATH SOURCE PREVIOUS_MANIFEST triples".into());
    }
    let before = rows.clone();
    let mut roots = vec![3];
    for (k, h) in &before {
        if k.len() != 10 || !k.starts_with(&[0, b'h']) || h.len() != 56 {
            continue;
        }
        let id = u64::from_be_bytes(k[2..].try_into()?);
        if id > 3
            && u64::from_le_bytes(h[48..].try_into()?) == 2
            && before.iter().any(|(e, v)| {
                e.starts_with(&seed::key(b'e', &3u64.to_be_bytes()))
                    && v.len() == 56
                    && v[8..40] == h[8..40]
            })
        {
            roots.push(id);
        }
    }
    let mut blocked = Vec::<String>::new();
    for pair in args.chunks_exact(3) {
        let path = &pair[0];
        let source = Source::parse(&pair[1])?;
        let previous = if pair[2] == "-" {
            None
        } else {
            if pair[2].len() != 64 || !pair[2].is_ascii() {
                return Err("invalid previous manifest".into());
            }
            Some(
                (0..32)
                    .map(|i| u8::from_str_radix(&pair[2][i * 2..i * 2 + 2], 16))
                    .collect::<std::result::Result<Vec<_>, _>>()?,
            )
        };
        println!("REALM_PROGRAM {path} {}", source.receipt());
        if blocked
            .iter()
            .any(|prefix| path == prefix || path.starts_with(&format!("{prefix}/")))
        {
            continue;
        }
        if let (Source::Program { current, .. }, Some(id)) = (&source, resolve(&before, 3, path)?)
        {
            let digest = &before[&seed::key(b'h', &id.to_be_bytes())][8..40];
            if digest != current && previous.as_ref().is_none_or(|p| p != digest) {
                blocked.push(path.clone());
                continue;
            }
        }

        for &root in &roots {
            let (parent, name) = match path.rsplit_once('/') {
                Some((prefix, name)) => {
                    let Some(parent) = resolve(rows, root, prefix)? else {
                        continue;
                    };
                    // A customized parent blocks all installation beneath it.
                    let template = resolve(rows, 3, prefix)?.ok_or("template parent")?;
                    if rows[&seed::key(b'h', &parent.to_be_bytes())][8..40]
                        != rows[&seed::key(b'h', &template.to_be_bytes())][8..40]
                    {
                        continue;
                    }
                    (parent, name)
                }
                None => (root, path.as_str()),
            };
            let existing = named(rows, parent, name.as_bytes())?;
            match &source {
                Source::Program { data, current } => {
                    let id = if let Some(id) = existing {
                        let digest = &rows[&seed::key(b'h', &id.to_be_bytes())][8..40];
                        if digest != current && previous.as_ref().is_none_or(|p| p != digest) {
                            continue;
                        }
                        id
                    } else {
                        allocate(rows)?
                    };
                    put(rows, id, parent, data)?;
                    hierarchy::describe(rows, id, name.as_bytes(), metadata::EXECUTABLE)?;
                }
                Source::Folder | Source::MatrixConfig if existing.is_some() => continue,
                Source::Folder => {
                    let id = allocate(rows)?;
                    put(rows, id, parent, &[])?;
                    hierarchy::describe(rows, id, name.as_bytes(), b"")?;
                }
                Source::MatrixConfig => {
                    let id = allocate(rows)?;
                    put(rows, id, parent, &hierarchy::configuration())?;
                    hierarchy::describe(rows, id, name.as_bytes(), metadata::MATRIX_CONFIG)?;
                }
                Source::Copy(_) if existing.is_some() => continue,
                Source::Copy(from) => {
                    let origin = resolve(rows, root, from)?
                        .ok_or_else(|| format!("copy source {from} is not installed"))?;
                    copy_subtree(rows, origin, parent, name.as_bytes())?;
                }
            }
        }
    }
    for root in roots {
        let Some(id) = named(rows, root, b"Matrix")? else {
            continue;
        };
        let head = &rows[&seed::key(b'h', &id.to_be_bytes())];
        let manifest = rows
            .get(&seed::key(b'm', &head[8..40]))
            .ok_or("configuration manifest")?;
        if manifest.first() != Some(&0) {
            continue;
        }
        let Some(mut config) = Config::decode(&manifest[17..]) else {
            continue;
        };
        let previous_config = config;
        if matches!(config.launcher_pages, 48 | 64) {
            config.launcher_pages = 96;
        }
        if matches!(
            (config.app_pages, config.app_readers, config.app_surfaces),
            (32, 4, 6) | (80, 64, 9) | (128, 128, 10) | (384, 768, 32)
        ) {
            config.app_pages = 384;
            config.app_readers = 768;
            config.app_surfaces = 64;
        }
        if config != previous_config {
            put(rows, id, root, &config.encode())?;
        }
    }
    let charge = rows
        .iter()
        .filter(|(k, _)| k[0] == 0)
        .map(|(k, v)| k.len() + v.len())
        .sum::<usize>();
    // The owning transaction collects unreachable history under quota pressure
    // after every new head has been staged, preserving live content and pins.
    rows.insert(seed::key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
    Ok(())
}
/// Take the next object number, as the store would.
fn allocate(rows: &mut Records) -> Result<u64> {
    let id = u64::from_le_bytes(rows[&seed::key(b'n', &[])].as_slice().try_into()?);
    rows.insert(
        seed::key(b'n', &[]),
        id.checked_add(1)
            .ok_or("object overflow")?
            .to_le_bytes()
            .to_vec(),
    );
    Ok(id)
}
/// The children of an object in ascending number order, from its edges.
fn children(rows: &Records, parent: u64) -> Result<Vec<u64>> {
    let prefix = seed::key(b't', &parent.to_be_bytes());
    rows.range(prefix.clone()..)
        .take_while(|(k, _)| k.starts_with(&prefix))
        .filter(|(k, _)| k.len() == 18)
        .map(|(k, _)| Ok(u64::from_be_bytes(k[10..].try_into()?)))
        .collect()
}
/// Copy an installed subtree exactly as the guest store does: every copy is a
/// new number at revision one sharing its original's content, enumerated
/// siblings before descendants, with names and types preserved below the
/// root, which takes the given name.
pub(super) fn copy_subtree(rows: &mut Records, source: u64, parent: u64, name: &[u8]) -> Result<u64> {
    let mut order = vec![source];
    let mut cursor = 0;
    while cursor < order.len() {
        order.extend(children(rows, order[cursor])?);
        cursor += 1;
    }
    let mut copies = BTreeMap::new();
    for &node in &order {
        let head = rows[&seed::key(b'h', &node.to_be_bytes())].clone();
        let node_parent = u64::from_le_bytes(head[48..].try_into()?);
        let mut suffix = node_parent.to_be_bytes().to_vec();
        suffix.extend_from_slice(&node.to_be_bytes());
        let entry = Metadata::decode(&rows[&seed::key(b't', &suffix)]).ok_or("source metadata")?;
        let new_parent = if node == source {
            parent
        } else {
            copies[&node_parent]
        };
        let copy = allocate(rows)?;
        let mut copied = 1u64.to_le_bytes().to_vec();
        copied.extend_from_slice(&head[8..48]);
        copied.extend_from_slice(&new_parent.to_le_bytes());
        rows.insert(seed::key(b'h', &copy.to_be_bytes()), copied.clone());
        rows.insert(seed::event(copy, 1), copied);
        let mut suffix = new_parent.to_be_bytes().to_vec();
        suffix.extend_from_slice(&copy.to_be_bytes());
        rows.insert(seed::key(b't', &suffix), Vec::new());
        hierarchy::describe(
            rows,
            copy,
            if node == source {
                name
            } else {
                metadata::text(&entry.name)
            },
            metadata::text(&entry.media_type),
        )?;
        copies.insert(node, copy);
    }
    Ok(copies[&source])
}
