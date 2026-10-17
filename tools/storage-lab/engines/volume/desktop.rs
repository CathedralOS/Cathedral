//! Independent oracle for the default desktop image after its interactive test:
//! construct every expected record without importing the guest store, then
//! compare the whole retained set. The notes root holds the installed programs
//! as objects one to four; two notes whose first revisions share one manifest
//! nest under the Notes program, and the color under the color program. The
//! ungranted root holds nothing.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use redb::ReadableTable;
use std::collections::BTreeMap;

const ROOT: u8 = 0;
/// Object numbers, in profile order: workspace, Notes, Colors, launcher.
const NOTES_PROGRAM: u64 = 2;
const COLORS_PROGRAM: u64 = 3;
const PROGRAMS: u64 = 4;

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
pub fn verify(path: &Path, offset: u64, bytes: u64, programs: &[seed::Program]) -> Result<()> {
    if programs.len() as u64 != PROGRAMS {
        return Err("the desktop installs four programs".into());
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
    let mut store = Store::new(Policy {
        chunk_bytes: 4096,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    // Note one is saved once; note two is copied from it, then edited and
    // saved; the color is created, then toggled once. Collection runs before
    // each write, so the copy's first event went when the color was created,
    // while the color's first event survives because nothing was written after
    // its toggle.
    let original = store.put(b"NTS1hello cathedral");
    let edited = store.put(b"NTS1hello cathedral two");
    let unset = store.put(b"MC01\x02\x00");
    let toggled = store.put(b"MC01\x02\x01");
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
    let first = colors::head(1, &manifests[0], 4 + "hello cathedral".len(), NOTES_PROGRAM);
    let second = colors::head(2, &manifests[1], 4 + "hello cathedral two".len(), NOTES_PROGRAM);
    let created = colors::head(1, &manifests[2], 6, COLORS_PROGRAM);
    let color = colors::head(2, &manifests[3], 6, COLORS_PROGRAM);
    let (note, copy, swatch) = (PROGRAMS + 1, PROGRAMS + 2, PROGRAMS + 3);
    expected.insert(key(b'h', &note.to_be_bytes()), first.clone());
    expected.insert(key(b'h', &copy.to_be_bytes()), second.clone());
    expected.insert(key(b'h', &swatch.to_be_bytes()), color.clone());
    expected.insert(edge(NOTES_PROGRAM, note), Vec::new());
    expected.insert(edge(NOTES_PROGRAM, copy), Vec::new());
    expected.insert(edge(COLORS_PROGRAM, swatch), Vec::new());
    expected.insert(event(note, 1), first);
    expected.insert(event(copy, 2), second);
    expected.insert(event(swatch, 1), created);
    expected.insert(event(swatch, 2), color);
    expected.insert(key(b'n', &[]), (swatch + 1).to_le_bytes().to_vec());
    let charge = expected.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>() + 10;
    expected.insert(key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
    if actual != expected {
        difference::report(&actual, &expected);
        return Err("desktop records differ from the independent object oracle".into());
    }
    println!(
        "VERIFIED: four installed programs, two notes sharing one manifest under Notes, the color under Colors, exact heads/history/edges/quota, empty second root ({} records)",
        actual.len()
    );
    Ok(())
}
