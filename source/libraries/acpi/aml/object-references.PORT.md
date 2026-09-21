# Canonical AML object reference kernels

This is a licensed component translation of `rust-osdev/acpi`
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5` (6.1.1), primarily
`src/aml/object.rs`. Copyright 2018 Isaac Woods; MIT OR Apache-2.0 texts remain
in `licenses/rust-osdev/acpi/LICENCE-MIT` and `LICENCE-APACHE`.
`object-references-inventory.json` binds the exact pin, license and source
anchors. This is a pure static namespace component, without an executor call
site, native object representation, hardware handler or boot integration.

## Source map

`model.omg` adds `ReferenceKind { Named, RefOf, Local, Arg, Index, Unresolved }`
and `Value::Reference { kind, object_id }`. Existing `Namespace`, `Object`,
`Value`, result records and stable-ID allocation remain canonical. These are
ordinary copyable metadata, never an Omega borrow, `ObjectToken`, access grant,
reference-counted allocation or proof of executable method provenance.

| Pinned component | Omega entry | Scope |
| --- | --- | --- |
| `WrappedObject::unwrap_reference` | `unwrap_all` | Follows all six object-reference kinds |
| `WrappedObject::unwrap_transparent_reference` | `unwrap_transparent` | Follows only Named, Local and Arg; Index, RefOf and Unresolved remain opaque |
| `ReferenceKind` | `model::ReferenceKind` | Exact six semantic cases |
| Reference construction / `WrappedObject::new` | `allocate_reference` | Validated existing target plus fresh bounded stable ID; no Arc/UnsafeCell/token construction |
| `Object` shallow clone components | `copy_value` | Copies only destination `.value`; retains both objects' sibling-link metadata |
| Package portion of `do_index` | `package_element` | Returns existing element identity after complete linked-chain validation; no target store or expression retirement |

`NameReference` is a distinct lexical descriptor. Neither unwrap operation
performs lookup, and package selection does not resolve element names or
unwrap element references. Existing `references::resolve_reference` retains
its separate namespace lookup policy. Generic `do_store`, `do_copy_object`,
DerefOf/RefOf/Index opcode execution, frame operands and targets remain pending;
a payload-copy primitive does not implement their distinct target rules.

## Bounds and failure policy

Namespace object counts and every dereferenced ID are checked against the
existing 64-slot arena. IDs are local to the supplied namespace, not global
handles; copied namespaces remain ordinary independent data. Validation is deliberately object-arena-only: `Namespace.count` and unrelated
path entries are not inspected. No whole-namespace certificate is produced. Inputs can
be ordinary malformed metadata; unrelated namespace entries and payloads are
not recursively validated. Reference-hop and package budgets must be in
0..64; larger values, including `u64::MAX`, return `InvalidState`.

Unwrap requires one budget step per inspected object. Zero budget returns
`WorkLimit`; insufficient budget retains the visited bitmap and current ID.
A followed edge checks its target bounds immediately, then detects a target
already visited (including a self-edge) as `ReferenceCycle`. The pin loops
without a cycle bound; finite cycles and dangling numeric IDs are explicitly
added Cathedral rejection policies. Transparent unwrap does not inspect the
pointee of an opaque reference, matching the pin's stopping rule.

Package selection validates the **complete advertised chain**, including the
tail after the requested element. Count above 64 returns `Capacity`, an index
outside count returns `MissingObject`, and a budget below count returns
`WorkLimit`. A dangling ID returns `InvalidState`; a repeated node encountered
within the advertised chain returns `ReferenceCycle`. Premature link termination
or an extra link after the advertised final node returns `BadEncoding`. An
empty package has no selectable element. Last-node extra-link rejection takes
precedence over following that extra link. These are linked-representation
validation policies, not extra constraints on a Rust Vec.

A copied package payload shares its first/count descriptor and element IDs;
future changes to those element slots are visible through both package objects.
The current linked-node representation cannot encode arbitrary independent
Vec membership using repeated occurrences of one node ID. General mutable
package membership/storage remains ordinary future work. Copying a String or
Buffer descriptor copies its inert source span; mutable byte backing and the
pin's owned byte-container clone behavior are not implemented here.

`allocate_reference` validates the target before allocation. Capacity failure
and all validation failures leave the original namespace unchanged. `copy_value`
validates source/destination IDs and then changes only destination `.value`;
source and destination `has_next`/`next` metadata, namespace bindings and object
count remain unchanged. Self-copy is allowed. It does not recursively validate
or materialize the copied payload.

## Method provenance and integration

`copy_value` may copy an inert Method payload, including flags and source span.
`MethodDefinitions` is a separate observation table and is not an argument to
this namespace primitive. A copied method is therefore not established as
executable by this operation. Future Program-level generic copy must atomically
invalidate or recapture affected method-definition provenance before publishing
the copy. The current integer executor and owned pipeline are unchanged; no
call site to these kernels is added. This is pending integration, not a compiler
or authority blocker.

## Verification

Current evidence passes **160 checked-interpreter scenarios and 160 changed-body
controls**, **five current constant-evaluation pairs**, and **116 actual public
Rust observations**. Existing parser **27**, integer executor **79**, owned
pipeline **22**, and field parser **18** scenarios and their original controls
also pass after the additive model change. All retained current record hashes
verify; previous parser/field/pipeline constant proofs keep their original
hashes and are explicitly historical.


The tools under `tools/ports/acpi/aml/object-references/` call the actual public
pinned immutable `WrappedObject` unwrap/clone APIs and compare identity through
public Deref observations. No `ObjectToken`, unsafe host mutation or hardware
callback is used. Cycle/dangling/resource-limit cases are original Omega policy
tests. Source checking, checked-interpreter execution and representative constant
proofs are separate stages, retained with exact source hashes. Native Omega
and hardware execution are not claimed. See the harness README and verification
records for current results; inventory regeneration alone is not execution proof.
