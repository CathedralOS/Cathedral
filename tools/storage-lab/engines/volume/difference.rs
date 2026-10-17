//! When an oracle disagrees with an image, say which records differ. Keys are
//! printed as hex with their kind byte, values by length, so a failing proof
//! names the record rather than only the fact of a mismatch.
use std::collections::BTreeMap;

pub(super) fn report(actual: &BTreeMap<Vec<u8>, Vec<u8>>, expected: &BTreeMap<Vec<u8>, Vec<u8>>) {
    let describe = |key: &[u8]| {
        let kind = key.get(1).map_or('?', |k| *k as char);
        format!("root {} kind {kind} key {}", key.first().unwrap_or(&255), hex(key))
    };
    for (key, value) in actual {
        match expected.get(key) {
            None => println!("UNEXPECTED {} ({} bytes)", describe(key), value.len()),
            Some(other) if other != value => println!(
                "DIFFERENT  {} ({} bytes, expected {} bytes){}",
                describe(key),
                value.len(),
                other.len(),
                if value.len() <= 64 && other.len() <= 64 {
                    format!(" actual {} expected {}", hex(value), hex(other))
                } else {
                    String::new()
                }
            ),
            Some(_) => {}
        }
    }
    for (key, value) in expected {
        if !actual.contains_key(key) {
            println!("MISSING    {} ({} bytes)", describe(key), value.len());
        }
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
