# Direct String-name lookup

Status: published and tested at final repository paths. All 97 checked
behavior/control pairs and three constant positive/rejecting-control pairs pass.
The verifier confirms all 18 current source, fixture and runner-recipe hashes.

`string_lookup::lookup(input, length, unit, store, source, scope)` accepts a direct
canonical String slot and returns failure or `Found(object, path)`. The object
is a stable ObjectStore ID and path is the resolved canonical absolute path.
This is the name-lookup part of DerefOf, not evaluation of the resulting object.
No value is copied, read as a field, executed as a method, unwrapped as a reference,
allocated or stored. A found String is not recursively interpreted as another
name. The store and source are borrowed read-only.

[ACPI 6.6 §19.6.30](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#derefof-dereference-an-object-reference)
permits String operands for DerefOf, interpreted as ASL names relative to the
current scope. This composition connects canonical byte storage, existing
`name_text::parse` and `namespace::search`. It preserves each helper's documented
profile; it adds no second parser, namespace resolver or canonical Value model.
The upstream component is `src/aml/mod.rs:2383 do_deref_of` in rust-osdev/acpi
257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5, MIT OR Apache-2.0, copyright 2018 Isaac
Woods. Exact notices/licenses remain in THIRD_PARTY_NOTICES.md and
licenses/rust-osdev/acpi. Whole do_deref_of remains pending.

Validation order is explicit. Staged unsigned namespace/object counts and source
ID must fit 32 entries and 64 objects before selecting the direct String case.
Other values, including all top-level references, return UnsupportedValue.
`read_bytes` then admits complete Source/Owned storage and the existing non-NUL
ASCII profile, before parsing any name. The text parser canonicalizes case and
pads short segments; it retains its 16-segment/16-parent limits and complete
error offset. Scoped lookup runs only after that admission succeeds.

Failures distinguish InvalidState, UnsupportedValue, Storage(ByteOutcome),
Name(Outcome, offset), and Search(Outcome). The outer result and inner error both
have failure defaults, yielding InvalidState. Success fields never carry partial
failure metadata. Source/Owned data and unused source metadata follow the same
rules as canonical byte storage; initialized nonlogical tails are ignored.

Single-segment unprefixed names search from the declared current level upward,
preferring the nearest valid object. Absolute, parent-prefixed and multisegment
names use the existing exact-path rules. Aliases preserve the target stable ID
while returning the path used to find it. The Path carries the existing helpers'
initialized unused segments: parent removes only the logical count, and relative
resolution overwrites only appended segments. For example, ROOT found from
DEV0.SUB0 retains SUB0 in unused segment1; an absolute parsed ROOT has zero tails.
This adapter preserves those ordinary initialized values rather than normalizing
them. Namespace counts are checked here;
this helper does not globally revalidate every stored entry. In particular, the
existing search treats a dangling nearest binding as absent and can find a valid
ancestor. Root/level-only paths with no object return a lookup failure. Stored
method/field metadata is not evaluated or required to be executable/readable by
this identity query.

Expected cases use authored namespace bindings and explicit resolved IDs/paths.
Checks compare semantic failure alternatives and detailed reasons/parse offsets,
or the ID and all 16 segments plus absolute/parent/count fields. The 97 cases
include malformed scope counts/parents, nonabsolute scopes, parsing-before-scope
failure precedence, and relative versus absolute unused-tail behavior. Changed-body
controls alter the expected error or ID; the three dirty-scope cases instead
change expected segment15 while retaining the correct ID. Representative constants cover an
absolute lookup, a rejected five-character segment and malformed owned storage.
No new public Rust or private-expression probe is claimed by this composition.
Existing textual-name and namespace components retain their separate observations.
