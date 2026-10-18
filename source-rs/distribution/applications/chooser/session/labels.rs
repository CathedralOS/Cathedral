//! Distribution display titles, read only through the chooser's own realm.
//! The installer appends a 32-byte footer to program objects; subtree copies
//! retain it. This is display text, not an object kind, identity or permission.
use cathedral_platform_contracts::storage as store;
use cathedral_user_runtime::storage::Scope;

pub fn program(scope: &mut Scope, object: u64) -> Option<[u8; 13]> {
    let head = scope
        .call(store::Request::new(store::HEAD, object, 0, 0, 0))
        .ok()?;
    let offset = head.length.checked_sub(32)?;
    let (reply, _) = scope
        .request(
            store::Request::new(store::READ, object, head.revision, offset, 32),
            None,
        )
        .ok()?;
    store::Reply::decode(&reply).ok()?;
    let footer = &reply[32..];
    if &footer[..8] != b"CTHTIT01" {
        return None;
    }
    let text = &footer[8..];
    let len = text.iter().position(|&ch| ch == 0)?;
    if !(1..=13).contains(&len)
        || !text[..len].iter().all(|ch| (32..=126).contains(ch))
        || text[len..].iter().any(|&ch| ch != 0)
    {
        return None;
    }
    let mut title = [0; 13];
    title[..len].copy_from_slice(&text[..len]);
    Some(title)
}

/// Ordinals describe visible entries, never expose or truncate database IDs.
pub fn numbered(prefix: &[u8], mut number: usize) -> [u8; 13] {
    let mut digits = [0; 20];
    let mut at = digits.len();
    loop {
        at -= 1;
        digits[at] = b'0' + (number % 10) as u8;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    let mut label = [0; 13];
    let len = digits.len() - at;
    label[..prefix.len()].copy_from_slice(prefix);
    label[prefix.len()..prefix.len() + len].copy_from_slice(&digits[at..]);
    label
}
