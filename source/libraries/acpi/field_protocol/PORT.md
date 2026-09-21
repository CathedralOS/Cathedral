# Detached BankField and IndexField sequencing

Status: **tested at the preserved `24aed04` metadata-owner checkpoint**.
Its three selected checked behavior/control pairs pass with exact recorded
inputs. The subsequent [Bank/Index namespace extension](../aml/field-indirect.PORT.md)
changes the shared object model; this conservative full-source receipt remains
historical. Protocol algorithms and canonical parser/metadata bodies are unchanged.
Upstream `b92c4ed` retains 204 checked-interpreter behavior/control pairs, three
constant-evaluator pairs and 60 public Rust observations. Their original exact
source/tool hashes remain historical after the shared Field types move to
`aml::field_model`; those receipts are unchanged. The migration changes imports,
the direct AML dependency and receipt source enumeration, with no protocol
algorithm or arithmetic-vector changes. This is a partial ACPI-005 component,
outside production roots.
The new migration receipt is
[retained separately](../../../../tools/ports/acpi/field-protocol/owner-migration-verification.json).
It plans logical operations, without executing AML, moving payload bytes,
installing namespace objects, acquiring locks or invoking a hardware provider.

## Origin and scope

Modified translation of rust-osdev/acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/mod.rs`:
`do_field_read` at line 2519 and `do_field_write` at line 2612. Upstream URL:
<https://github.com/rust-osdev/acpi>. Copyright 2018 Isaac Woods;
MIT OR Apache-2.0, with retained notices in
[`THIRD_PARTY_NOTICES.md`](../../../../THIRD_PARTY_NOTICES.md) and
[`licenses/rust-osdev/acpi/`](../../../../licenses/rust-osdev/acpi/).
`model.omg` and `protocol.omg` are modified detached decompositions; `build.omg`
defines the isolated package. Original host fixtures live in
[`tools/ports/acpi/field-protocol`](../../../../tools/ports/acpi/field-protocol/).

The primary rules are ACPI 6.6
[BankField, §19.6.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#bankfield-declare-bank-data-field),
[IndexField, §19.6.64](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#indexfield-declare-index-data-fields),
and [Field, §19.6.48](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#field-declare-field-objects).
The planner reuses the existing canonical `Field`, `DeclarationKind`,
`IntegerSize`, normal-field geometry and its complete footprint validation.
It introduces no competing field, byte-storage or namespace implementation.

The two-file inventory retains all 158 whole-source anchors as pending.
Only the two interpreter methods and FieldUnitKind have partial component
links. All other algorithms in those files remain outside this claim;
`inventory.py check` audits the exact pin and every scanner anchor, not
semantic equivalence or full method completion. The upstream read/write
methods have no directly ported standalone unit tests. Original public AML
observations exercise their actual dispatch through the pinned interpreter.

## Inputs and admission

`protocol::plan(&Request)` accepts Bank or Index plus a read/write direction.
The outer field supplies bit offset, bit length, effective access/update and
lock metadata. `selector` supplies a normal register field, its declaration
kind and parent-region byte extent. Index additionally supplies an independent
normal `data` register and region extent. Register kinds are explicit;
recursive Bank/Index registers reject as `UnsupportedKind` at their component.
Resolving names, verifying region identity and retaining source observations
are future caller obligations. Relative offsets are not physical addresses.

Admission order is outer kind, complete outer geometry, selector geometry,
Index data geometry, then selector fit. Failures identify Field, Selector or
Data with the existing geometry error; selector overflow is a separate result.
There is no partial recipe on failure. The ordinary input is borrowed and
unchanged, and the result confers no capability or reusable validation token.

All normal-geometry constraints remain in force: 1–2048 field bits, at most
257 chunks, byte/word/dword/qword access, AnyAcc choosing byte access, strict
flags/metadata/connection checks, and complete aligned footprints. A bit
interval exceeding `u64::MAX` rejects. Bank uses its supplied region length.
Index outer offsets describe a virtual indexed range, so no physical outer
region is checked; `Request.region_bytes` is ignored for Index. Both actual
register-region footprints are independently checked.

BankValue is normalized to the selected four/eight-byte Integer size before
its selector-fit check. Index selectors retain exact unsigned byte offsets;
they are not narrowed by IntegerSize. Every emitted selector must fit the
selector field's logical bit length. Checking the final Index chunk suffices
because admitted chunk offsets increase monotonically. Fields of at least
64 bits accept any `u64` selector. No truncating selector wrap is accepted.
This is an explicit bounded-profile choice, stricter than ordinary Field Store;
it is not presented as an ACPI mandate.

Unused data metadata in Bank requests is ignored and yields a completely
default `data` plan. Unused BankValue in Index requests is ignored and the
returned `bank_value` is zero. Tests include malformed and locked unused data.

## Recipe meaning and remaining obligations

A successful `Recipe` retains independently validated outer, selector and
Index data plans, direction, normalized BankValue and a bounded action list.
Each data read or write is immediately preceded by `Select(value)`. A read
produces Select/ReadDatum for every outer chunk. A write produces
Select/ReadDatum/Select/WriteDatum for partial Preserve chunks, and
Select/WriteDatum for complete chunks or WriteAsOnes/WriteAsZeros updates.
Read/WriteDatum identifies the corresponding outer chunk by index. The
1028-action capacity follows the conservative four-actions-per-chunk bound;
all unused slots are initialized to `None`.

Bank Select uses the normalized BankValue. Index Select uses each outer chunk's
aligned byte offset, independent of the DataName width or native access type.
Selector writes use their own normal field geometry and update rule. Bank data
actions identify native outer chunks. Index data actions identify complete
logical transfers through the independently planned DataName field, including
its own alignment, update rule, native widths and footprint. DataName and
selector lengths may range across the entire inherited 2048-bit profile;
neither must equal the outer access width.

The consumer contract for an Index datum uses the low `outer.width` bytes of a
DataName read, zero-extending a shorter result. A datum write is truncated or
zero-extended to the DataName logical length, with its own surrounding-bit
update rule. This is the stated bounded composition interpretation of indirect
Field transfers. **The Omega planner does not perform these payload operations
or expand logical actions into native callbacks.** Public Python comparisons
expand the intended recipe into inert-memory operations as reference evidence;
they are not Omega payload-execution evidence. Existing normal geometry is
retained so a later consumer can implement the full transfer without guessing
register width or discarding update metadata.

The returned global-lock requirement is the union of all used components'
lock flags, always unmet when requested. Independently of those flags, a future
consumer must establish selector-region identity and serialize the complete
selector/data transaction against interfering accesses. It must retain that
serialization across Preserve reads and writes, including re-selection. The
recipe establishes neither region synchronization nor Global Lock ownership.
Bank/data regions may differ, and an apparent numeric match proves no shared
identity. Provider grants, supported address spaces/widths, physical placement,
completion/failure accounting and recursive-register cycle limits remain the
obligations in [`ADAPTER.md`](../ADAPTER.md).

Repeated selection is a deliberate per-datum policy compatible with the ACPI
ordering rule; the specification does not dictate its exact callback count.
The pin writes BankValue once per field operation and an Index selector once
per chunk, including across a Preserve read/write pair. These frequency
differences are recorded without declaring them specification violations.

## Differences and verification

The public probe has 60 actual pinned Interpreter load/evaluate observations
using initialized Vec memory and the unchanged public Field harness. Every
other service traps; all observations record zero forbidden callbacks.
There are 28 agreements and 32 explicit differences: 12 selector-frequency,
eight independent DataName geometry, six unaligned Index selector, four strict
selector-overflow rejection, one Buffer-size and one 32-bit BankValue
normalization difference. The pin directly accesses DataName's region using
the outer width instead of transferring through DataName's own field geometry.
The pin also starts Index selection at an unaligned byte offset, contrary to
§19.6.64. No compatibility branch reproduces those behaviors.

The 204 Omega scenario/control pairs compare every recipe field, all three
257-slot geometry arrays and the 1028-slot action array, including inactive
tails. Independent arithmetic covers both directions, all native widths and
update rules, lock composition, differing register widths, capacity extremes,
high-bit/MAX values, malformed input and error precedence. Controls change
live selector values, datum kinds/indices, direction/count, component geometry,
inactive tails and failure component/discriminant. Three representative
constant-evaluator pairs are separate evidence. Checked execution took 312.784
seconds with three workers; constant evaluation took 128.916 seconds. Maximum
checked fuel was 182,114 per body. The pre-relocation receipts bind 23 original inputs, exact
fixture/build text, original execution root and immutable compiler/runner
hashes. Their pre-relocation read-only verifier passed, including binary checks.
Final receipts and exact commands belong to the tooling README.

No Omega modification or compiler blocker is required for this slice.
Namespace integration, dynamic BankValue evaluation, recursive field protocols,
payload assembly, providers, synchronization and opcode retirement remain
ordinary pending work. ACPI-004/005/006 remain open.
