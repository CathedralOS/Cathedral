# Package Index reference construction

Status: tested. Final repository-path replay passed 65 checked behavior/control
pairs, three constant pairs and 19 public observations; current receipts verified.

`package_index::make_package_index(store, package, index)` composes existing
`object_references::package_element` and `allocate_reference` over the canonical
ObjectStore. The input is a direct Package slot and already-evaluated unsigned
index. Success returns canonical `Read { outcome:Success, value:fresh_id }`, with
next/offset zero; failure returns its exact outcome with value/next/offset zero.
It creates one canonical `Value::Reference { kind:RefOf, object_id:element }`.
No new value, namespace, storage or reference representation is introduced.

[ACPI 6.6 §19.6.63](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#index-indexed-reference-to-member-object)
defines Package Index as a reference to the selected member. The pinned
`src/aml/mod.rs:2283` `Interpreter::do_index` Package branch at line 2320 uses
RefOf, even though the upstream ReferenceKind also has an Index case. Cloning the
WrappedObject preserves member identity; this adaptation preserves its stable ID.
The source is rust-osdev/acpi `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, MIT OR
Apache-2.0, copyright 2018 Isaac Woods. Exact notices are retained under
licenses/rust-osdev/acpi and THIRD_PARTY_NOTICES.md. The aggregate do_index anchor
remains pending: operand evaluation, transparent source unwrapping, target Store,
frame contribution and opcode retirement are outside this component.

## Admission and publication

The existing selection helper first admits object count and direct Package ID
against the 64-slot arena. It then requires a Package payload, rejects advertised
count above 64 as Capacity, and index outside count as MissingObject. A fixed
budget of 64 covers every admitted advertised chain. All nodes, including those
after the selected member, are validated before allocation: dangling IDs give
InvalidState; repeated visited nodes give ReferenceCycle; premature termination
or an extra final link gives BadEncoding. The last link is rejected without
following it. Empty packages have no selectable member. No index truncation or
implicit Integer normalization occurs at this already-evaluated boundary.

Only successful selection reaches canonical reference allocation. Its existing
namespace-entry count guard (at most 32) and free object slot requirement can
return Capacity. Thus malformed chains precede exhausted allocation or malformed
entry-count metadata. These linked-representation and fixed-capacity checks are
Cathedral admission policy, not extra Rust Vec or ACPI limits. No unrelated
namespace path, binding, element payload or storage backing is admitted.

Success replaces the previously unused object slot at the old object_count,
zeros that object's sibling metadata, and increments object_count by one. Every
other object, all namespace entries and the entire byte arena remain unchanged.
In particular, an old byte block at the new reference slot is retained exactly:
a Reference payload does not use that block. No source storage is materialized,
cloned or cleared. Any failure publishes no mutation. A sequence of two calls
retains the first successful allocation if the second encounters capacity.

Repeated indexing allocates distinct wrappers referring to the same member ID.
NameReference elements stay lexical and unresolved; existing Reference elements
stay wrapped rather than followed. Uninitialized, nested Package and Owned byte
payloads are likewise retained without validating their contents. Self-containing
Package payload graphs can be retained when the advertised sibling chain itself
is valid. This is ordinary detached metadata, never a borrow, ObjectToken, live
capability, allocator permission or whole-store validity certificate. The linked
membership representation still cannot encode arbitrary independent Vec
membership using repeated occurrences of the same node identity.

## Evidence

The 65 original scenarios cover each selected position, repeated wrappers,
allocation at slot 63 and exhaustion, malformed tails after selection, all six
reference kinds, every canonical payload case, unresolved names/backing, copied
Package descriptors, a 62-member chain, MAX/high-bit counts and indexes, and exact
validation precedence. The shared checker compares all 64 object payloads and
links, all 32 namespace entries with all 16 Path slots, and every byte and metadata
field of all 64 byte blocks. Controls change expected wrapper kind/target/count
or preserved storage. Three constant pairs retain representative success,
last-slot and late-failure cases with all object/entry metadata and first/last
byte sentinels per block; the exhaustive byte comparison stays in the checked
stage because it exceeds the constant evaluator's 100,000-step budget. No compiler or native ABI result is inferred
from inventory coverage.

The 19 public observations execute original authored AML through the actual
pinned `Interpreter::new/load_table/evaluate` APIs with all hardware/service
callbacks trapped. Ordinary immutable observations compare returned RefOf inner
pointers against original public Package members and confirm repeated results
have distinct wrappers with identical member identity. The probes include mixed
Integer/String/Buffer/nested Package/NamePath members, uninitialized elements,
empty packages and out-of-bounds indexes. No ObjectToken is fabricated, no private
body is mirrored and no hardware is accessed. Fixed-arena failures are Omega-only
policy tests, not claimed public Rust equivalences.

[Tool instructions](../../../../../tools/ports/acpi/aml/package-index/README.md)
include exact commands and source-bound retained receipt validation.
