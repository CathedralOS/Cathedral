# Platform service contracts

[lib.rs](lib.rs) exposes named service boundaries, independent of providers:

| Boundary | Protocol entrance | Provider entrance |
| --- | --- | --- |
| Display/compositor | [display/display.rs](display/display.rs); sibling composition, pointer and interaction protocols | [display/main.rs](../services/display/main.rs) |
| Physical keys | [input.rs](input.rs) | [input/main.rs](../services/input/main.rs) |
| Recursive local worlds | [matrix.rs](matrix.rs) | [matrix/main.rs](../services/matrix/main.rs) |
| Experimental content objects | [database.rs](database.rs) | [storage/database/main.rs](../services/storage/database/main.rs) |
| Private catalogs | [storage.rs](storage.rs) | [storage/main.rs](../services/storage/main.rs), or experimental [database catalog mode](../services/storage/database/catalog.rs) |

These are experimental wire definitions and validators, with lab bounds and
legacy protocol modes still present. Placement does not imply a stable ABI or a
production interface. Providers enforce caller authority using kernel-authenticated
connections; a message or numeric token alone grants nothing.

This no_std package imports no implementations. It uses public kernel status
codes for the existing catalog response convention. The kernel must not depend
on these service protocols. Its [framebuffer contract](../../kernel/contracts/framebuffer.rs)
owns hardware mapping information, independently of compositor drawing messages.

New interfaces belong to their semantic owner. Private implementation details stay
with that implementation; distribution status/launch ordering stays in
[session-protocol](../../distribution/libraries/session-protocol/lib.rs).
Ordinary storage adapter traits live in [block-device](../libraries/block-device/README.md).
