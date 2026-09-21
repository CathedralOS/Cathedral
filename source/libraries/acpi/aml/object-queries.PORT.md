# Bounded canonical AML object queries

This component adapts `resolve_name_path`, `object_type` and `do_size_of` from
rust-osdev/acpi `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, copyright 2018 Isaac
Woods, MIT OR Apache-2.0. Retained license texts are in
`licenses/rust-osdev/acpi/`. It uses the existing Namespace, Value, ObjectStore,
ReferenceState and byte-storage APIs. Generic opcode dispatch is separate.

`resolve_object` follows both lexical NameReference values and all six canonical
Reference kinds. Name lookup retains each lexical value's declared scope and
uses the existing namespace search rules. A single visited-ID bitmap and caller
budget cover alternating edge kinds, so mixed name/reference cycles cannot
restart a fresh budget indefinitely. Each inspected object consumes one unit;
zero budget inspects none. Budgets above64, arena counts above64, entry counts
above32 and invalid starting IDs reject. A followed invalid ID or repeated target
rejects immediately; exhausted work retains the pending ID and visited bitmap.
The pin's unbounded ID unwrap and separate eight-name loop become this explicit
bounded policy. No new string-path parsing or method evaluation occurs.

These checks validate relevant resolution paths, not every namespace entry or
payload. Existing duplicate-entry and level rules remain namespace policies.
Unused package sibling links are irrelevant to reference resolution. The helper
does not read byte contents, execute a method or establish access authority.

`object_type` resolves the supplied object ID and reports its canonical numeric
AML kind. It deliberately does not validate an owned-byte locator, source span,
method provenance or region service merely to report the metadata kind. Pure
Scope levels are entries without an object ID and need a separate opcode lookup
path. Runtime FieldUnit, Debug and RawDataBuffer are not current Value variants;
no coverage of those absent payloads is claimed.

`size_of` resolves references, then validates complete current String/Buffer
storage with `byte_storage::byte_length`, or the complete advertised package
sibling chain with `object_references::package_element`. Package validation has
its own64-node bound in addition to the resolution budget. Empty packages have
no live first-element ID. Child values need not be initialized or recursively
valid merely to count membership. A malformed tail cannot publish a successful
size. Non-size-bearing objects return the semantic UnsupportedValue case;
storage and resolution errors remain separate QueryResult alternatives.

The primary [ACPI6.6 ObjectType definition](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#objecttype-get-object-type)
and [SizeOf definition](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#sizeof-get-data-object-size)
define numeric kinds, byte lengths and package membership counts. Bounded
metadata validation and recoverable failures are Cathedral adaptations.

The namespace get/bind guards stage object IDs and counts into explicit `u64`
locals. On the pinned checked evaluator, directly comparing the two record-field
expressions could incorrectly admit `u64::MAX < 1`; the staged comparisons reject
that malformed ID. Direct binding and mixed lexical-resolution cases retain the
regression. No Omega source is modified.

The final source snapshot passes 96 checked-interpreter scenarios and 96 changed
expected-body controls, plus three positive/negative constant-evaluation pairs.
The separate public Rust probe loads 44 original AML fragments: 41 numeric
results agree, two Integer SizeOf operations return explicit errors, and a Buffer
initializer larger than its declared length panics during load before SizeOf.
This query helper uses the primary Buffer sizing rule already implemented by
owned storage. No forbidden host callback occurs in the public observations.

All 306 existing parser, reference, integer-execution, field and pipeline
regression pairs also pass. Their fresh receipts live under this milestone's
`regressions/` directory; earlier model-dependent receipts keep their original
hashes as historical evidence. The namespace change does not alter byte-storage
call paths, so unchanged storage kernels retain their prior behavior receipts.

Records and reproduction tools live under `tools/ports/acpi/aml/object-queries/`.
The source inventory translates the bounded resolver and retains numeric type
and size as partial operation components. Native ABI, device, firmware and
production integration remain untested.
