// SPDX-License-Identifier: MIT OR Apache-2.0
// Authored exhaustive checks of the pinned upstream pure flag selectors.
use uefi_raw::protocol::hii::ifr::{IfrDateFlags,IfrTimeFlags,IfrNumericFlags};
#[test]
fn all_flag_selector_inputs() {
    for value in 0..=u8::MAX {
        assert_eq!(IfrDateFlags::from_bits_retain(value).storage().0,value & 0x30);
        assert_eq!(IfrTimeFlags::from_bits_retain(value).storage().0,value & 0x30);
        assert_eq!(IfrNumericFlags(value).size().0,value & 0x03);
        assert_eq!(IfrNumericFlags(value).display().0,value & 0x30);
    }
}
