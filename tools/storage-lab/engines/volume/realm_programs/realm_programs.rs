// Add named programs to recognized stock realms without reserving object IDs.
// Existing customized entries block that path and its descendants. Only copies
// matching the installer receipt's previous/current bytes are refreshed. One transaction
// and the realm-program backup cover code, metadata and the old stock budget upgrade.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use cathedral_desktop_matrix_contracts::matrix::config::Config;
use cathedral_platform_contracts::storage::metadata;
use std::collections::BTreeMap;
type Records = BTreeMap<Vec<u8>, Vec<u8>>;
use entries::{named, put, resolve};
pub fn install(rows: &mut Records, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    if !args.len().is_multiple_of(3) {
        return Err("realm programs are PATH FILE PREVIOUS_MANIFEST triples".into());
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
        let data = std::fs::read(&pair[1])?;
        if !data.starts_with(b"\x7fELF") {
            return Err("realm addition is not a program".into());
        }
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
        let mut content = Store::new(Policy {
            chunk_bytes: 4096,
            inline_limit: 128,
            layout: Layout::Flat,
        });
        let version = content.put(&data);
        let current = content.records(&version)[b"head".as_slice()].clone();
        println!(
            "REALM_PROGRAM {path} {}",
            current
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        if blocked
            .iter()
            .any(|prefix| path == prefix || path.starts_with(&format!("{prefix}/")))
        {
            continue;
        }
        if let Some(id) = resolve(&before, 3, path)? {
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
            let id = if let Some(id) = named(rows, parent, name.as_bytes())? {
                let digest = &rows[&seed::key(b'h', &id.to_be_bytes())][8..40];
                if digest != current && previous.as_ref().is_none_or(|p| p != digest) {
                    continue;
                }
                id
            } else {
                let id = u64::from_le_bytes(rows[&seed::key(b'n', &[])].as_slice().try_into()?);
                rows.insert(
                    seed::key(b'n', &[]),
                    id.checked_add(1)
                        .ok_or("object overflow")?
                        .to_le_bytes()
                        .to_vec(),
                );
                id
            };
            put(rows, id, parent, &data)?;
            hierarchy::describe(rows, id, name.as_bytes(), metadata::EXECUTABLE)?;
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
        if config.launcher_pages == 48 {
            config.launcher_pages = 64;
        }
        if (config.app_pages, config.app_readers, config.app_surfaces) == (32, 4, 6) {
            config.app_pages = 80;
            config.app_readers = 64;
            config.app_surfaces = 9;
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
    if charge > 1024 * 1024 {
        return Err("realm programs exceed logical quota".into());
    }
    rows.insert(seed::key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
    Ok(())
}
