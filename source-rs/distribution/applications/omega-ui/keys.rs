use cathedral_platform_contracts::input;

// shortcut: this lab transport exposes physical positions only. Add composed
// text and modifiers when Cathedral delivers them; never invent Ctrl/IME events.
pub(crate) fn translate(key: u64) -> Option<(i32, i32)> {
    let key = u8::try_from(key).ok()?;
    Some(match key {
        input::A..=input::Z => (0, i32::from(b'a' + key - input::A)),
        input::DIGIT_0..=input::DIGIT_9 => (0, i32::from(b'0' + key - input::DIGIT_0)),
        input::SPACE => (32, 32),
        input::BACKSPACE => (8, 0),
        input::LEFT => (37, 0),
        input::RIGHT => (39, 0),
        input::ENTER => (13, 0),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn physical_positions_are_bounded_and_navigation_is_not_text() {
        assert_eq!(translate(input::A as u64), Some((0, 97)));
        assert_eq!(translate(input::Z as u64), Some((0, 122)));
        assert_eq!(translate(input::DIGIT_9 as u64), Some((0, 57)));
        assert_eq!(translate(input::LEFT as u64), Some((37, 0)));
        assert_eq!(translate(input::RESET as u64), None);
        assert_eq!(translate(271), None);
    }
}
