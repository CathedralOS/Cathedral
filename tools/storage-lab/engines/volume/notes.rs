//! Independent oracle for the notes fixture image: one note per root, saved
//! through the physical typing workload in tools/boot-harness-rs/notes_test.py.
use super::*;
use std::collections::BTreeMap;

const FULL: &str = "hello cathedral notes abcdefghijklmnopqrstuvwxyz 0123456789abc";
fn note(text: &str) -> Vec<u8> {
    let mut bytes = b"NTS1".to_vec();
    bytes.extend_from_slice(text.as_bytes());
    bytes
}
pub fn verify(path: &Path, offset: u64, bytes: u64, last: [u64; 2]) -> Result<()> {
    if last != [4, 2] {
        return Err("the notes workload ends at revisions 4 and 2".into());
    }
    let db = open(Region::new(Arc::new(Image::open(path)?), offset, bytes)?)?;
    let tx = db.begin_read()?;
    let table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))?;
    let actual = colors::records(&table)?;
    let mut expected = BTreeMap::new();
    colors::expect(&mut expected, 0, 4, |revision| match revision {
        1 => note("hello"),
        2 => note("hello cathedral notes"),
        3 => note(&FULL[..60]),
        _ => note(&FULL[..35]),
    });
    colors::expect(&mut expected, 1, 2, |revision| match revision {
        1 => note("private sibling note"),
        _ => note("private sibling note 2"),
    });
    if actual != expected {
        difference::report(&actual, &expected);
        return Err("notes records differ from the independent object oracle".into());
    }
    println!(
        "VERIFIED: two private notes, exact text/history/quota ({} records)",
        actual.len()
    );
    Ok(())
}
