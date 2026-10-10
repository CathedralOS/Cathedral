# Storage provider

A separate user task owns the exclusive boot-granted disk controller and the
platform object-store engine. Private init control accepts health requests;
one boot-approved application data link grants read/replace of object 1.
Other object numbers are denied. Device failures retire the task so recovery
reopens disk state; replacing the application leaves the service and data live.

Requests are 64 bytes: four little-endian u64s `(operation, object, expected
generation, payload length)`, then 32 payload bytes. READ (1) requires zero
generation/length/payload; REPLACE (2) requires length <=32 and zero unused
payload. Replies are `(status, generation, length, reserved zero)` plus payload.
Control messages never grant data access. There is no unrestricted pathname or
object-number lookup. This single-client grant is not a multi-tenant realm API.

Only `recovery-lab` accepts private control commands for crash/spin/block and
write-boundary pauses. Identical packets sent over the app data channel fail.
Test probes also check device register bounds, rejected DMA/slave commands,
invalid buffers and read-only destinations before device side effects.

Init decides restart policy. The driver resets unfinished PIO on replacement;
the engine recovers its committed record. A service restart counter is volatile;
the object's generation and contents are persistent. See the driver and
object-store charters for the precise hardware and durability limits.
