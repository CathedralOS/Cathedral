# Equal-extent named Buffer Store

The later [zero-length Buffer conversion extension](zero-buffer-store.PORT.md)
admits Integer/String stores into existing empty Buffers. The profile and
source-bound receipts below record this earlier milestone, before that extension.

Status: tested in the isolated worktree at
`/private/tmp/cathedral-acpi-buffer-store`: 97 new and 504 regression
behavior/control pairs pass; both exact-input verifiers and the inventory audit
pass. Twelve actual public Rust observations also pass.
This extends `named_value_store::store_value` only. It retains the canonical
ObjectStore, byte arena, failure results and private atomic publisher.

The partial source relationship is rust-osdev/acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/mod.rs:2406 do_store` and
`src/aml/object.rs:317 replace_with_implicit_casting`. The added source remains
MIT OR Apache-2.0, copyright 2018 Isaac Woods, with notices retained in
`licenses/rust-osdev/acpi` and `THIRD_PARTY_NOTICES.md`. The
[inventory](named-buffer-store-inventory.json) retains all 158 aggregate anchors
as pending; it does not claim whole Store completion.

## Supported behavior

An admitted direct Buffer destination now accepts a direct Buffer source with
the same admitted logical length, from zero through 256 bytes. Both Source and
Owned storage are supported, including self-store, shared immutable input spans,
Buffer initializers shorter than their declared size, aliases to the destination
ID and a full object arena. IntegerSize does not alter Buffer bytes.

Destination ID, kind and complete old backing are admitted first. Source ID and
complete source backing follow. Only then are lengths compared. The existing
`implicit_conversions::convert(..., Buffer)` snapshots the canonical bytes and
zeros the unused tail. The existing publisher installs that snapshot at the same
destination ID. There is no fresh allocation or writable alias to the source.
Self-store therefore normalizes the destination's unused storage without losing
its logical bytes. Object links, all bindings and every other object/byte block
remain unchanged; all failures preserve the complete store.

Unequal lengths retain the previous profile errors: Bounds for an empty
destination with nonempty source, UnsupportedValue for other unequal lengths.
These are local exclusions, not normative AML errors. Malformed source backing
is now reported before this length comparison. Existing Integer/nonempty String
conversion rules, their positive target extents, and empty String exclusions are
unchanged. The standalone `buffer_target_values::prepare_buffer_extent` remains
an Integer/String conversion helper.

## Extent decision still open

[ACPI 6.6 named-object rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#named-objects)
require copying with type matching. The
[conversion table](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
specifies existing Buffer geometry for Integer/String conversions, but has no
same-type Buffer row. The
[Buffer declaration rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#buffer-declare-buffer-object)
define initial geometry and admit empty Buffers. These passages do not explicitly
settle unequal-extent Buffer Store. The historical
[ACPI 3.0a](https://www.intel.com/content/dam/www/public/us/en/documents/articles/acpi-config-power-interface-spec.pdf)
§17.2.5.9.3/Table 17-15, §17.5.9 and §17.5.113 retain the same generic copying and
declaration wording.

The pin replaces a Buffer with the source's exact length. In contrast,
[ACPICA's public implementation](https://github.com/acpica/acpica/blob/master/source/components/executer/exstorob.c)
retains a nonzero dynamic target's length, truncating or padding the source, and
grows a zero-length or static target. Equal extents avoid this disagreement.
Selecting unequal-extent compatibility is the narrow high-level decision still
needed; this does not block other ACPI-005 work. No resize or fixed-extent rule is
claimed to follow merely from the word “copy.”

## Evidence

The new suite has 97 behavior/control pairs. It checks both integer widths,
Source/Owned combinations, empty and maximum lengths, all logical byte values,
source padding, self-stores, complete failure atomicity and admission ordering.
Every check compares all 64 object payloads/links, 32 namespace entries, their
16 Path slots, byte block metadata and every one of the 16,384 arena bytes.
Controls alter the expected result, target kind, identity, links or byte tail.

The regression suite reuses all 504 scalar/admission fixture bodies, changing
only the expected results of the two former Buffer self-store exclusions to
successful copies. The old receipts retain their historical inputs. Fresh
receipts bind this worktree's root/build text, the used production closure,
baseline/current fixture files, rendered bodies and immutable checked runner.
There is no new constant, native ABI, hardware or interpreter-dispatch claim.

The dedicated run completed in 877.391 seconds and the regression run in
1226.699 seconds, with two independent workers each. Maximum observed evaluator
fuel was 502,098 and 516,854 respectively, below the fixed 10,000,000 ceiling.
Both receipts confirm unchanged source inputs and runner bytes. This is
worktree execution evidence; it is not relabelled as canonical-path replay.

Twelve fresh calls to the actual pinned public
`Object::replace_with_implicit_casting` pass: six equal-extent observations and
six unequal-extent observations documenting the pin's resize behavior. They use
the existing ordinary-owned-object Rust probe, without an Interpreter, unsafe
ObjectToken, source/target aliasing or service calls.

Commands and receipts are in the
[tool directory](../../../../tools/ports/acpi/aml/named-buffer-store/README.md).
