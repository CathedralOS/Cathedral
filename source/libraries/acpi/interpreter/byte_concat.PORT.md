# Bounded same-type concatenation

Status: **63 checked-interpreter positives and 63 changed-body controls pass**,
plus three representative constant-evaluation pairs. The reference probe passes
49 explicitly labelled private-expression observations and 162 actual public
conversion/access calls, with 14 explicit inapplicable Rust inputs. This is a
partial pure component of ACPI-005, not generic `Concatenate` opcode execution.

The source is a modified translation of `do_concat` in
[rust-osdev/acpi at 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2171),
MIT OR Apache-2.0, copyright 2018 Isaac Woods; Cathedral modifications 2026.
The existing `THIRD_PARTY_NOTICES.md` and `licenses/rust-osdev/acpi/` apply.
Primary rules are [ACPI 6.6 §19.6.12, Table 19.30](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#concatenate-concatenate-data).

## API and boundary

All three operations return the existing `IntegerResult`: error 0 and `value`
equal to output length on success; error 4 for bounded-capacity failure, error 7
for invalid string encoding; `remainder` is always zero. Failures return zero
length and leave all 256 output bytes unchanged. Successful writes touch only
`output[0..value]`; every tail byte is preserved, including bytes outside the
caller-declared capacity. Inputs and mutable output are distinct borrows.

- `concat_integers(size, left, right, output, capacity)` produces 8 or 16 bytes,
  left integer first, little-endian. Four-byte width truncates both inputs to
  their low 32 bits. It preflights capacity, reuses `integer_to_buffer` in local
  initialized arrays and appends with the same bounded copy primitive.
- `concat_buffers(left, a, right, b, output, capacity)` requires each logical
  extent and capacity at most 256, then checks complete combined fit before
  writing. A logical extent need not consume its entire initialized input array.
- `concat_strings` uses the same extent representation without a terminator.
  Order is fixed: reject any extent/capacity outside 0..256; validate both entire
  logical inputs as nonzero 7-bit ASCII; check combined output fit; then copy.
  Thus a late bad byte wins over insufficient output space, while an invalid
  capacity argument wins over encoding. No output NUL is appended. Input bytes
  outside logical extents are ignored.

The 256-byte capacity and failure codes are Cathedral's explicit resource
profile, not an ACPI limit or a claim about the pin's allocation failures. Empty
inputs, including two empties with capacity zero, succeed. Subtraction-based fit
checks follow bounded extents, so arbitrary `u64` lengths/capacities cannot wrap.
The pin's Rust strings can contain NUL/UTF-8 that this ASCII extent profile rejects.

No new `Value`/`Object` representation is introduced. Inputs are already converted
and of the same type. Generic implicit conversion, type-name formatting,
reference resolution, namespace/frame storage, target coercion, result object
allocation, operation retirement and AML opcode dispatch remain pending. Existing
Mid/ToString/ToBuffer helpers are unchanged. This module grants no device access.

## Source map

| Pin component | Mapping |
| --- | --- |
| `do_concat`, lines 2203–2213, integer bytes | `concat_integers`, reusing `integers::integer_bits` and `conversions::integer_to_buffer` |
| `do_concat`, lines 2216–2218, buffer append | `concat_buffers` with bounded preflight and tail preservation |
| String clone/append within `do_concat` | `concat_strings` for already-String inputs, with the explicit ASCII profile |
| Whole `do_concat` and local `resolve_as_string` | Pending: this component does not implement generic object/conversion/Store/context semantics |

`byte-concat-inventory.json` audits complete `src/aml/mod.rs` and `object.rs`
bytes/anchors. The aggregate `do_concat` anchor remains pending with a partial
helper target. Existing conversion helpers are dependencies, not retranslations
or newly completed generic object anchors.

## Verification and updates

Fixtures are original synthetic inputs. Each positive checks all 256 output
bytes, status, length and remainder; each control mutates expected byte 255,
including full-capacity outputs, preserved tails and error atomicity. The corpus
covers both integer widths, truncation, arbitrary `u64` extent/capacity values,
empty/exact/overflow cases, late invalid ASCII/NUL and error precedence.

The Rust probe performs actual public `Object::as_integer`, `to_integer`,
`as_buffer` and `to_buffer` calls. Its append operations are explicitly labelled
private expression mirrors: **it does not call the private `do_concat`, execute
an AML opcode or use a Handler/Store implementation**. Same-type conversion
results independently agree with those mirrors. Invalid Rust strings and
unrepresentable lengths are explicitly omitted; bounded-only errors are Omega
checks. Full source bodies, licenses and Cargo lock are hashed.

The checked interpreter executes authored Omega bodies and changed-body controls.
Representative constant-evaluation pairs provide additional evidence; neither
route makes a native ABI or live firmware claim. The five-file Cathedral source
closure is `byte_concat`, `integers`, `conversions`, `buffer_fields` and this
package's unchanged build declaration. Omega core and shared runner are pinned
separately. Reproduction and final hashes are in
`tools/ports/acpi/interpreter/byte-concat/README.md` and its verification records.

On pin updates, audit both complete source inventories and licenses, review
conversion/string policy and integer width, regenerate cases, and rerun public
conversion observations, private-expression mirrors and actual Omega controls.
Keep the whole generic operation pending until its remaining behavior is supplied.
