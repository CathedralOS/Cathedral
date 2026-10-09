# String and number helpers — partial ACPI-005

## Scope and provenance

`string_numbers.omg` adds pure components to `cathedral-acpi-interpreter`.
They use the existing `IntegerSize`/`IntegerResult` and initialized `[u8;256]`
conventions. There is no new generic Object type, opcode execution, namespace
mutation, target store, method service, hardware access or production integration.

The reference is rust-osdev/acpi 6.1.1,
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, copyright 2018 Isaac Woods,
MIT OR Apache-2.0. Exact licenses remain under `licenses/rust-osdev/acpi/`.
The pin's actual path is `src/aml/object.rs`; there is no `object/mod.rs`.
The license/manifest/source hashes are audited before the host comparison runs.
All scenario inputs are original Cathedral data. No external firmware or uACPI
fixture content is copied.

## Source map

| Pin | Component | Remaining operation |
| --- | --- | --- |
| `Object::to_integer`, String branch | `string_to_integer` | Other Object variants, reference dispatch and full conversion policy |
| `Interpreter::do_to_integer` | Numeric parser result only | Argument extraction, target stores, context contribution and retirement |
| `do_to_dec_hex_string`, Integer/Buffer branches | `integer_to_string`, `buffer_to_numeric_string` | String identity, other Object dispatch, target/context behavior |

`string-numbers-inventory.json` indexes both complete source files. All anchors
remain pending in this component inventory; partial target mappings do not
reclassify whole generic operations as translated. Existing independently
completed work is not overwritten by this narrow inventory.

## Primary rules and selected policy

[ACPI 6.6, sections 19.6.139–141](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#tointeger-convert-data-to-integer)
specifies decimal/hexadecimal conversion, comma-separated per-byte formatting,
empty-buffer formatting as an empty string, and illegal empty-string ToInteger.
It does not define every malformed lexical outcome. Its integer data type has
selected 32/64-bit width. The profiles below distinguish those forms from
Cathedral's deterministic validation and the pin's compatibility behavior.

Both parser profiles take an ASCII logical String extent excluding NUL. Any NUL
or non-ASCII byte anywhere in that extent returns `ERROR_ENCODING` before parsing,
including bytes after a numeric-prefix delimiter. This matches this helper
package's bounded String convention. It deliberately excludes the pin's broader
Rust UTF-8 strings, Unicode whitespace trimming and interior-NUL acceptance.
A physical initialized array is always supplied; `length > 256`, including
`u64::MAX`, returns `ERROR_CAPACITY` without indexing.

| Behavior | `StrictDecimalHex` | `PinnedAscii` |
| --- | --- | --- |
| Empty extent | `ERROR_EMPTY` | zero |
| Decimal / `0x` or `0X` hex | Entire digit sequence required | Numeric prefix only |
| Leading zeros | Decimal unless hex prefix | Same; no octal interpretation |
| Whitespace | Rejected | Leading ASCII space or bytes 9–13 skipped; later whitespace stops parsing |
| `+`/`-` | Rejected | First nondigit produces zero; no signed arithmetic |
| Empty hex prefix / initial nondigit | `ERROR_ENCODING` | zero |
| Delimiter, suffix, underscore, dot | `ERROR_ENCODING` | Stops before first nondigit |
| Overflow | Reject above selected 32/64-bit maximum | Reject above u64 maximum; width argument intentionally ignored like pin |

The strict lexical rules are a named Cathedral profile, not a claim that ACPI
mandates these exact whitespace/sign/error decisions. Overflow is checked before
multiply/add; no wrapped, saturated or truncated value is returned on overflow.
Internal wrapping expressions follow an explicit preflight check. A successful
parser returns its integer in `IntegerResult.value`; remainder is zero. Failure
returns error with zero value.

## Formatting and bounded storage

Integer formatting normalizes to the selected width, as the other helpers do;
the pin formats the raw u64. Decimal has minimal digits, including `0` for zero.
Hexadecimal uses uppercase digits and lowercase `0x` prefix, following the pin's
`{value:#X}` expression. Integers have no extra zero padding.

Buffer formatting processes every byte, including zero; zero is not a string
terminator in a Buffer. Decimal bytes have minimal digits. Hex bytes have two
digits after `0x`, following the pin's `{byte:#04X}` expression. Commas separate
values, with no spaces or trailing comma. An empty Buffer formats to an empty
logical String. These exact hexadecimal presentation details follow the pin;
the primary explicit-conversion clauses do not require that particular padding.

Callers provide distinct input/output borrows and an output capacity at most 256.
The complete result length is measured before writing. Too-small or oversized
capacity, or oversized input extent, returns `ERROR_CAPACITY` with every output
byte unchanged. There is no truncating mode or partial successful output. On
success, `value` is logical length excluding NUL; the helper writes no terminator
and leaves all storage beyond that length unchanged. Error tags reuse existing
3 overflow, 4 capacity, 6 empty and 7 encoding meanings.

## Verification stages

The new tools directory is `tools/ports/acpi/interpreter/string-numbers/`.
`reference.py` compiles the pinned crate and invokes the actual public
`Object::to_integer` on constructible original inputs. The private formatting
operation is compared through a separately labelled pure-body mirror in
`reference.rs`; it does not claim to execute the upstream opcode or target path.
Raw pinned results remain visible when the selected-width/ASCII/strict policy
intentionally differs. Invalid UTF-8 and impossible initialized extents are
explicitly outside the Rust comparison domain.

`check.py` checks and executes authored Omega bodies in the pinned checked
interpreter, with a changed-body control for every scenario. Formatting tests
compare all 256 output bytes, including unchanged tails and error paths. The
harness's 10-million step ceiling is separate from these finite API bounds.
Representative cases also use actual constant evaluation and checked contracts.
Retained records bind source, fixtures, toolchain and outcomes; see the tools
README for commands and current results. Native Omega execution, live firmware,
ABI conformance and complete generic conversion opcodes remain unverified.
