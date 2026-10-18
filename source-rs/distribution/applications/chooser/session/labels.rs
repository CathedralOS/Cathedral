//! User creation names; stored in hierarchy metadata.
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
