# Transactional namespace-level removal

`namespace_removal::remove_level(&mut Namespace, Path) -> Outcome` removes an
absolute level and its descendants from the bounded canonical namespace.
The object bound at the exact same path belongs to the parent level and survives;
its level flag is cleared. This distinction matters for Device-like paths that
contain both a level and an object. Unrelated entries retain their order.

This is the pure adaptation of rust-osdev/acpi
[`Namespace::remove_level`, namespace.rs:156 at 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/namespace.rs).
Copyright 2018 Isaac Woods, MIT OR Apache-2.0. Cathedral retains the exact notices
and licenses. The pin stores objects and child levels in separate maps; Cathedral
uses one Entry with independent object/level flags. The selected inventory marks
one anchor translated and leaves the rest of the namespace file outside this slice.

All validation precedes mutation. The active namespace must contain 1–32 entries
and at most 64 allocated objects, an absolute root level, unique valid absolute
entry paths, existing parent levels, at least one object/level flag per entry, and
valid IDs for object-bearing entries. Object payloads themselves are not evaluated
or recursively validated. Entries outside the active count are ignored on input.
Ordinary malformed namespaces are rejected as InvalidState, including maximum
unsigned counts/IDs. This structural admission is stricter than the pin's internal
map representation, which cannot express duplicate map keys or orphan child maps.

A relative target returns NotAbsolute, and an invalid canonical target returns
InvalidName, after the count checks. Complete structural validation then precedes
even no-op requests. Removing root is a successful no-op. A nonexistent child level
under an existing parent is also a successful no-op, including object-only paths.
A missing parent returns MissingLevel. No-op and failure results preserve the
entire Namespace, including all unused slots.

A successful removal stages a complete initialized 32-entry array, then replaces
only `entries` and `count`. All vacated slots are default initialized. Surviving
paths, object IDs and alias flags are preserved. `objects` and `object_count` never
change. External aliases and stable object references therefore retain their
identities even when a namespace path disappears. IDs are not recycled and object
slots are not reclaimed; this remains within the existing Program lifetime/capacity
profile. ObjectStore byte storage is outside this function's borrow and cannot be
changed. No method-definition observation or live device state is modified.

The function performs bounded structural/path scans and entry compaction. Maximum
path depth is 16; entry scans are at most 32 with nested bounded parent/duplicate
checks. There is no allocator, handler, raw address or firmware operation. This API
does not implement AML Unload, revoke existing object identities, or detach live
operation-region services. Such behavior requires separate execution/lifecycle work.

The Omega suite contains 44 behavior/control pairs covering subtree removal,
coexisting object/level slots, aliases, root/missing/object-only no-ops, absent
parents, 32-entry capacity, depth 16, malformed structures and high-bit/MAX metadata.
Every case compares all 32 entries including all path slots, all 64 object payloads
and sibling links, and both counts. Controls alter the last object's sibling link
in the expectation, demonstrating that checking only lookup results cannot pass.
Two representative constant pairs cover real subtree removal and a maximum-count
failure. Final receipts bind exact used source/tool hashes; no native execution is
claimed.

Twelve actual public Rust Namespace observations confirm the matching normal-tree
behavior and errors. They compare complete level and binding/alias sets before and
after removal, including predefined namespace entries, without dereferencing
WrappedObject or acquiring an ObjectToken. Handle 0 identifies a retained host-owned
mutex for Namespace construction; the probe performs no handler calls or AML
execution. Malformed flat layouts and fixed capacities have separate Omega checks
because the Rust map representation does not expose those inputs.
