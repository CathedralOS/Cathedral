# Bounded byte comparisons — partial ACPI-005

Status: tested with 81 checked-interpreter cases and 81 changed-body controls,
four constant-evaluation pairs, and 62 actual public Rust comparison calls. This component implements same-type
Buffer and ASCII String ordering over two initialized `[u8;256]` inputs and
explicit logical lengths. Results use semantic `Comparison` cases; no object,
reference, namespace or target representation is introduced.

Modified source: rust-osdev/acpi 6.1.1, revision
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/object.rs::aml_cmp` and
`src/aml/mod.rs::do_logical_op`. Copyright 2018 Isaac Woods, MIT OR Apache-2.0;
retained licenses are under `licenses/rust-osdev/acpi/`. All test data is original.

## Source map

The inventory binds both complete upstream files. This helper covers only their
same-type byte ordering expressions. Generic Object dispatch, implicit right-hand
conversion to the left type, transparent references, op retirement, logical truth
conversion and target/context effects remain pending. Neither entire upstream
method is marked translated by this component.

[ACPI 6.6 sections 19.6.70–73](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#lgreater-logical-greater)
require unsigned lexicographic byte order, using length only if the compared
prefix is equal. The pin instead compares Buffer lengths before any bytes.
`BufferOrdering::Lexicographic` implements the primary rule;
`PinnedLengthFirst` preserves that intentional compatibility difference. For
example `[90]` is greater than `[65,65]` lexicographically but less under the
pinned rule. Equal-sized buffers use the same ordering in both profiles.
Zero and bytes 128–255 are ordinary unsigned Buffer data.

`compare_strings` always uses lexicographic order. Its logical String extent
excludes NUL and requires every byte to be nonzero ASCII. Both entire extents
are validated before returning an ordering, including bytes after a differing
prefix. This is the existing Cathedral bounded String profile; the pin's Rust
String also permits Unicode and interior NUL. Empty String is valid.

Both APIs reject either length above 256, including u64::MAX, as `Capacity`
before indexing. Invalid String bytes produce `Encoding`. Input tails beyond
the logical lengths are ignored. All loops have finite bounds, and inputs are
immutable. No hardware, allocation, execution or storage authority is conveyed.

The Rust reference calls the actual public `Object::aml_cmp` on original owned
values and preserves its raw results, including cases where primary ordering or
ASCII validation deliberately differs. Impossible logical extents and invalid
UTF8 Strings are explicitly outside its comparison domain. The checked runner
executes actual Omega bodies and changed expected-result cases, with separate
representative constant-evaluation controls. These are semantic stages, not
native execution or complete logical-opcode integration.
