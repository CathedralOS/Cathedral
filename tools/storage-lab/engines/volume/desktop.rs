//! Independent oracle for the default desktop image after its interactive test:
//! construct every expected record without importing the guest store, then
//! compare the whole retained set. The notes root holds the installed objects
//! one to six: the chooser, the folder of users, and inside it the template
//! user with its chrome, Notes and Colors. Realm entries then add programs,
//! folders, matrix configurations and subtree copies in the profile's order,
//! which is how Background, Split and the prebuilt nested desktops arrive. The
//! copied user shares their content, while its saved notes/color are distinct
//! objects. The ungranted root holds nothing.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use cathedral_platform_contracts::storage::metadata::{self, Metadata};
use redb::ReadableTable;
use std::collections::BTreeMap;

const ROOT: u8 = 0;
/// Installed objects, in profile order.
const USERS: u64 = 2;
const TEMPLATE: u64 = 3;
const INSTALLED: u64 = 6;
type Records = BTreeMap<Vec<u8>, Vec<u8>>;

fn key(kind: u8, suffix: &[u8]) -> Vec<u8> {
    let mut key = vec![ROOT, kind];
    key.extend_from_slice(suffix);
    key
}
fn event(object: u64, revision: u64) -> Vec<u8> {
    let mut suffix = object.to_be_bytes().to_vec();
    suffix.extend_from_slice(&revision.to_be_bytes());
    key(b'e', &suffix)
}
fn edge(parent: u64, child: u64) -> Vec<u8> {
    let mut suffix = parent.to_be_bytes().to_vec();
    suffix.extend_from_slice(&child.to_be_bytes());
    key(b't', &suffix)
}
/// The object a parent's name index points at, if any.
fn child(expected: &Records, parent: u64, name: &str) -> Result<Option<u64>> {
    let mut suffix = parent.to_be_bytes().to_vec();
    suffix.extend_from_slice(&metadata::field(name.as_bytes()).ok_or("invalid entry name")?);
    Ok(match expected.get(&key(b'd', &suffix)) {
        Some(id) => Some(u64::from_be_bytes(id.as_slice().try_into()?)),
        None => None,
    })
}
fn resolve(expected: &Records, base: u64, path: &str) -> Result<u64> {
    let mut parent = base;
    for name in path.split('/') {
        parent = child(expected, parent, name)?.ok_or_else(|| format!("missing entry {path}"))?;
    }
    Ok(parent)
}
/// A new object with its own content: a program or an empty folder.
fn place(expected: &mut Records, id: u64, parent: u64, bytes: &[u8], name: &[u8], kind: &[u8]) -> Result<()> {
    let mut store = Store::new(Policy {
        chunk_bytes: 4096,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    let version = store.put(bytes);
    let content = store.records(&version);
    let head = seed::head(1, &content[b"head".as_slice()], bytes.len(), parent);
    for (k, v) in content {
        if k != b"head" {
            if k[0] == b'c' {
                expected.insert(key(b'k', &k[1..]), vec![]);
            }
            expected.insert(key(k[0], &k[1..]), v);
        }
    }
    expected.insert(key(b'h', &id.to_be_bytes()), head.clone());
    expected.insert(event(id, 1), head);
    expected.insert(edge(parent, id), vec![]);
    hierarchy::describe(expected, id, name, kind)
}
/// Copy a subtree the way the store numbers it: siblings before descendants,
/// every copy at revision one sharing its original's content, names and types
/// preserved below the root, the root taking `name`. Returns the copies made.
fn copy_tree(
    expected: &mut Records,
    source: u64,
    parent: u64,
    name: &[u8],
    next: &mut u64,
) -> Result<u64> {
    let mut order = vec![source];
    let mut cursor = 0;
    while cursor < order.len() {
        let prefix = key(b't', &order[cursor].to_be_bytes());
        let children: Vec<u64> = expected
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .filter(|(k, _)| k.len() == 18)
            .map(|(k, _)| u64::from_be_bytes(k[10..].try_into().unwrap()))
            .collect();
        order.extend(children);
        cursor += 1;
    }
    let mut copies: BTreeMap<u64, u64> = BTreeMap::new();
    for &node in &order {
        let source_head = expected[&key(b'h', &node.to_be_bytes())].clone();
        let source_parent = u64::from_le_bytes(source_head[48..].try_into()?);
        let entry = Metadata::decode(&expected[&edge(source_parent, node)]).ok_or("source metadata")?;
        let new_parent = if node == source {
            parent
        } else {
            copies[&source_parent]
        };
        let copy = *next;
        *next += 1;
        let mut head = 1u64.to_le_bytes().to_vec();
        head.extend_from_slice(&source_head[8..48]);
        head.extend_from_slice(&new_parent.to_le_bytes());
        expected.insert(key(b'h', &copy.to_be_bytes()), head.clone());
        expected.insert(event(copy, 1), head);
        expected.insert(edge(new_parent, copy), Vec::new());
        hierarchy::describe(
            expected,
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
    Ok(order.len() as u64)
}
pub fn verify(
    path: &Path,
    offset: u64,
    bytes: u64,
    programs: &[seed::Program],
    additions: &[String],
) -> Result<()> {
    if programs.len() as u64 != INSTALLED {
        return Err("the desktop installs six objects".into());
    }
    if !additions.len().is_multiple_of(2) {
        return Err("realm entries are PATH SOURCE pairs".into());
    }
    let db = open(Region::new(Arc::new(Image::open(path)?), offset, bytes)?)?;
    let tx = db.begin_read()?;
    let table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))?;
    let mut actual = BTreeMap::new();
    for row in table.iter()? {
        let (key, value) = row?;
        actual.insert(key.value().to_vec(), value.value().to_vec());
    }
    let mut expected = seed::expected(programs);
    for (id, name) in [
        (1u64, b"Login".as_slice()),
        (2, b"Users"),
        (3, b"Template"),
        (4, b"Desktop"),
        (5, b"Notes"),
        (6, b"Colors"),
    ] {
        hierarchy::describe(
            &mut expected,
            id,
            name,
            if id == 2 { b"" } else { metadata::EXECUTABLE },
        )?;
    }
    hierarchy::insert_config(&mut expected, 7, TEMPLATE)?;
    let mut next = 8u64;
    // Construct the realm entries independently; no installer call is used.
    for pair in additions.chunks_exact(2) {
        let (path, source) = (&pair[0], &pair[1]);
        let (parent, name) = match path.rsplit_once('/') {
            Some((owner, name)) => (resolve(&expected, TEMPLATE, owner)?, name),
            None => (TEMPLATE, path.as_str()),
        };
        match source.as_str() {
            "@folder" => {
                place(&mut expected, next, parent, &[], name.as_bytes(), b"")?;
                next += 1;
            }
            "@matrix-config" => {
                if name != "Matrix" {
                    return Err("a matrix configuration is named Matrix".into());
                }
                hierarchy::insert_config(&mut expected, next, parent)?;
                next += 1;
            }
            copy if copy.starts_with("@copy=") => {
                let origin = resolve(&expected, TEMPLATE, &copy["@copy=".len()..])?;
                copy_tree(&mut expected, origin, parent, name.as_bytes(), &mut next)?;
            }
            file => {
                let bytes = std::fs::read(file)?;
                place(&mut expected, next, parent, &bytes, name.as_bytes(), metadata::EXECUTABLE)?;
                next += 1;
            }
        }
    }
    let user = next;
    let copied = copy_tree(&mut expected, TEMPLATE, USERS, b"User 1", &mut next)?;
    let (notes, colors) = (user + 2, user + 3);
    let mut store = Store::new(Policy {
        chunk_bytes: 4096,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    // Colors creates its default before either note is saved. Note one is saved
    // once; note two is copied then edited. Colors is toggled last. This workload
    // stays below maintenance pressure, retaining the copy's original event too.
    let original = store.put(b"NTS1hello cathedral");
    let edited = store.put(b"NTS1hello cathedral two");
    let unset = store.put(b"MC01\x00\x00");
    let toggled = store.put(b"MC01\x00\x01");
    let mut manifests = Vec::new();
    for version in [&original, &edited, &unset, &toggled] {
        for (k, v) in store.records(version) {
            if k == b"head" {
                manifests.push(v);
                continue;
            }
            if k[0] == b'c' {
                expected.insert(key(b'k', &k[1..]), Vec::new());
            }
            expected.insert(key(k[0], &k[1..]), v);
        }
    }
    let first = colors::head(1, &manifests[0], 4 + "hello cathedral".len(), notes);
    let second = colors::head(2, &manifests[1], 4 + "hello cathedral two".len(), notes);
    let created = colors::head(1, &manifests[2], 6, colors);
    let color = colors::head(2, &manifests[3], 6, colors);
    let (swatch, note, copy) = (user + copied, user + copied + 1, user + copied + 2);
    expected.insert(key(b'h', &note.to_be_bytes()), first.clone());
    expected.insert(key(b'h', &copy.to_be_bytes()), second.clone());
    expected.insert(key(b'h', &swatch.to_be_bytes()), color.clone());
    expected.insert(edge(notes, note), Vec::new());
    expected.insert(edge(notes, copy), Vec::new());
    expected.insert(edge(colors, swatch), Vec::new());
    expected.insert(event(copy, 1), first.clone());
    expected.insert(event(note, 1), first);
    expected.insert(event(copy, 2), second);
    expected.insert(event(swatch, 1), created);
    expected.insert(event(swatch, 2), color);
    expected.insert(key(b'n', &[]), (copy + 1).to_le_bytes().to_vec());
    let charge = expected
        .iter()
        .map(|(k, v)| k.len() + v.len())
        .sum::<usize>()
        + 10;
    expected.insert(key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
    if actual != expected {
        difference::report(&actual, &expected);
        return Err("desktop records differ from the independent object oracle".into());
    }
    println!(
        "VERIFIED: six base objects, {} realm entries including the prebuilt nested desktops and matrix configurations, a named user copied from the template ({copied} objects), two notes sharing one manifest under its Notes, the color under its Colors, exact heads/history/edges/quota, empty second root ({} records)",
        additions.len() / 2,
        actual.len()
    );
    Ok(())
}
