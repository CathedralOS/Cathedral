# Bootstrap bitmap font

Owns an original hand-authored 5-by-7 pixel font for printable ASCII. `font.hex`
stores seven hexadecimal row bytes per glyph in ASCII order; only five bits per
row are used. Lowercase currently shares uppercase shapes. Out-of-range input
maps to the question-mark glyph. This allocation-free, host-testable library has
no device access, kernel imports or distribution appearance policy.

The display provider uses these glyphs for bounded text requests. Font shaping,
Unicode, accessibility and production typography remain unimplemented.
