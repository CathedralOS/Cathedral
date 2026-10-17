# Storage provider

A separate user task owns the exclusive boot-granted disk controller and the
platform catalog engine. [main.rs](main.rs) reports service termination;
[service entrance](service/service.rs) opens the disk and catalog, accepts connections, and
dispatches data requests to the [catalog protocol](../../libraries/object-store/protocol.rs).
Private init control handles health and optional
[recovery hooks](service/recovery.rs). It conveys no object access.

Two boot-approved data links bind to private roots 0 and 1 by their service-local
link position. The kernel isolates principals and grants endpoints; the platform
service enforces root-local object access. The message has no root selector.
The stock profile binds status first and counter second; changing that order on
an existing disk changes ownership and is unsupported. This static lab binding
is not a production identity, realm-admission or authority-transfer protocol.
Both clients may use object numbers 1 through 4 without seeing each other's data.

The separate [matrix storage fixture](../matrix/world/storage/README.md) binds both
links to the root matrix. It delegates each whole catalog through child-local
connections with per-hop binding revisions. Descendant revocation aborts staging;
provider disconnect events also release transaction state before reuse. The
durable format and root-local protocol are unchanged.

Requests are 64 bytes: four little-endian u64s `(operation, object, expected
root revision, payload length)`, then 32 payload bytes. Unused bytes must be zero.
The shared constants and record encoding live in `platform/contracts/storage.rs`.

| Operation | Arguments | Result |
| --- | --- | --- |
| READ (1) | Object, zero revision/length | Object payload and current root revision |
| REPLACE (2), CREATE (4) | Object, expected revision, payload | Committed object |
| LIST (3) | All arguments zero | Present object numbers as packed u64s |
| DELETE (5) | Object, expected revision, empty payload | Committed listing |
| BEGIN (6) | Object zero, expected revision, empty payload | Current listing; private staging starts |
| STAGE_CREATE (7), STAGE_REPLACE (8), STAGE_DELETE (9) | Object, revision zero, payload (empty for delete) | Current committed listing; disk unchanged |
| COMMIT (10), ABORT (11) | All arguments zero | Committed/current listing |

Replies are `(status, root revision, length, reserved zero)` plus 32 payload
bytes. Invalid object numbers return DENIED, absent objects NOT_FOUND, existing
CREATE targets EXISTS, stale revisions WOULD_BLOCK. At most two distinct changes
can be staged; excess returns NO_MEMORY. Staging errors preserve earlier staged
changes until ABORT, COMMIT or connection replacement. COMMIT consumes staging
even on conflict or validation failure; no partial transaction is published.

The runtime serves control and data channels in round-robin order. Idle polling
uses a bounded IPC wait set and periodic checks for new boot-link connections.
A full data reply queue loses that reply without blocking other
clients. Synchronous clients keep one outstanding request; after uncertain
completion they must regain a usable connection and read/reconcile revisions
before retrying a mutation. There is no request-ID deduplication cache.

Only `recovery-lab` accepts private control commands for crash/spin/block and
write-boundary pauses. Identical data-channel packets fail. Device errors retire
the provider; init resets the PIO transport and reopens committed state. App
replacement leaves the provider and other app alive. Provider replacement retires
both data connections and staging, then each app reconnects to its original root.
See the object-store and driver charters for format and durability limits.

The separately built [database provider](database/CHARTER.md) experiments with real
chunked content and private-root IPC on redb. It has its own contract and lab
composition; it does not reinterpret or migrate this catalog's format.
