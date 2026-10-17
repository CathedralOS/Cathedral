# Platform service contracts

[lib.rs](lib.rs) exposes named service boundaries, independent of providers:

| Boundary | Protocol entrance | Provider entrance |
| --- | --- | --- |
| Display/compositor | [display/display.rs](display/display.rs); sibling composition, pointer and interaction protocols | [display/main.rs](../services/display/main.rs) |
| Physical keys | [input.rs](input.rs) | [input/main.rs](../services/input/main.rs) |
| Recursive local worlds | [matrix.rs](matrix.rs) | [matrix/main.rs](../services/matrix/main.rs) |
| Private object storage | [storage.rs](storage.rs) | [storage/main.rs](../services/storage/main.rs) |

These are experimental wire definitions and validators with lab bounds. Placement does not imply a stable ABI or a
production interface. Providers enforce caller authority using kernel-authenticated
connections; a message or numeric token alone grants nothing.

The object protocol carries one 64-byte packet each way. Publications attach one
sealed memory object; bulk reads return one. A matrix forwards both directions,
scoping the operation word by the child's binding and stamping the binding's
scope object into a free scope word (see [matrix.rs](matrix.rs) `STORAGE_BASE`).
Object numbers are relative to the accepted root, arranged as a tree and never
reused; a copy is a new number sharing the source's content, and a subtree copy
repeats that for every descendant.

This no_std package imports no implementations. It uses public kernel status
codes for provider refusals, plus two of its own for a missing object and an
oversized publication. The kernel must not depend on these service protocols. Its
[framebuffer contract](../../kernel/contracts/framebuffer.rs) owns hardware mapping
information, independently of compositor drawing messages.

New interfaces belong to their semantic owner. Private implementation details stay
with that implementation.
Ordinary storage adapter traits live in [block-device](../libraries/block-device/README.md).
