# Declared service imports

Start at [imports.rs](imports.rs): resolve a program's requirements against the
services its owner offers. [image.rs](image.rs) locates the declaration in sealed
ELF bytes. This platform contract imports no implementation and adds no kernel
service names, lookup syscall or PID-based authorization.

The declaration is a request ceiling, not a grant. Each import names a contract,
an inclusive compatible wire-version interval, and required/optional status.
The owner selects actual capabilities. Missing or incompatible required imports
refuse launch before reservation/spawn; optional ones remain absent. Services
not declared are not supplied. A bound service can fail or be revoked later;
startup compatibility is not a permanent availability promise or proof that a
provider implements its advertised behavior correctly.

The accepted [component/matrix ownership contract](../../../../wiki/spec/execution/components_and_matrices.md)
owns the parent-ceiling and supplied-service semantics. The Rust representation
below is bounded implementation evidence, not the final manifest or a persisted
matrix configuration schema.

## Current bounded representation

The runtime's `service_imports!` emits `CTHIMP01`; `service_bootstrap!` emits
`CTHIMP03` for [scoped acquisition](../services/README.md) and requested resources.
The earlier `CTHIMP02` scoped-acquisition format remains accepted. All contain a header followed by
four reserved records of four little-endian u64 words: contract, minimum version,
maximum version, required (0/1). Unused records are zero. The linker retains this
144-byte record first in the read-only PT_LOAD segment, so trimming debug/section
headers preserves it. It remains inside the sealed code object used for admission.
The kernel independently validates the ELF; it never interprets this record.

The two implemented contracts are INTERACTION (1: local input, surfaces and wake)
and STORAGE (2: scoped object operations), both at wire version 1. Their current
offer/withholding masks use bits 1 and 2. Unknown optional contracts are ignored;
unknown required ones refuse. This is not yet an arbitrary extensible endpoint
table or the final installed component manifest.

For new store programs, [matrix admission](../../../distribution/matrix/service/children/admission.rs)
maps the immutable image, resolves its imports, unmaps it, then reserves and
installs only the selected connections. Owner `LAUNCH_IMPORTS` selects the offer
mask for future launches in a slot. A child's `LAUNCH` can additionally withhold
offers, never widen them. Reads still use the launcher's realm; granting storage
binds the admitted program to its own subtree. No-storage launches receive no
storage endpoints or realm binding. Attempts through the interaction endpoint
are denied in the desktop's separate-stream protocol.

Legacy startup transport uses the existing five-endpoint kernel
bundle: interaction pair, wake receiver, and optional storage pair. Three
endpoints mean storage is absent; five mean present. [Runtime views](../../libraries/user-runtime/matrix/matrix.rs)
provide required connections or `OptionalConnections` with an actual `Option`
storage pair. Broker startup instead installs one request/reply pair and transfers approved
service endpoints on request. The compiler checks view types, not whole-program reach or
the honesty of arbitrary executable declarations.

Old program copies without a record use the **explicit legacy declaration**:
interaction and storage both required. This preserves saved realms without
silently teaching old code to handle absent storage. Withholding storage refuses
those binaries. Boot and catalogue-based lab admission retain their existing
explicit wiring; they are not claimed to enforce this new store-launch contract.
Legacy headless matrices still multiplex storage on interaction under a scoped
binding; withholding the binding denies that access too.

## Visible proof

Notes and Colors now enter through scoped service acquisition in F5. Notes accepts
absent storage and keeps editing/dirty-close protection; Colors requires storage.
The old launcher STORAGE toggle is gone. The disposable desktop test variants
`--service-imports undeclared` and `--service-imports incompatible` remove Notes'
storage binding while Colors keeps its own persistent connection. No runtime
request can widen the result resolved from the sealed import record.

Host tests cover declaration parsing, version compatibility, unknown contracts,
missing required and optional services, malformed records and truncated ELF.
`desktop_test.py` drives the actual desktop and cold-reopens the disk;
`--matrix-storage-test` checks owner policy, child withholding/refused widening, successful retry, realm confinement and revocation. Service discovery,
first-request activation, dynamic grant addition and final manifest persistence
remain separate work.

`CTHIMP03` packs the import count in header bits 0..7, requested channel creation
in 16..31 and requested descendant capacity in 32..47. Other header bits must be
zero. `service_bootstrap!(imports)` requests zero channels/descendants;
`service_bootstrap!(imports, channels, descendants)` requests explicit ceilings.
Stock store admission grants the minimum of each request and the owner's offer.
These are ceilings, not required minima, and grant no extra service or executable.
This prevents ordinary leaf programs from reserving the budget of an entire matrix.
The older formats retain owner-offered resource policy for saved-code compatibility.


`CTHIMP04` adds three little-endian u64 words after the 144-byte import record:
private pages, reader pages and surface demand. Its channel/descendant header
matches `CTHIMP03`, but it uses the ordered environment bootstrap currently used
by the stock matrix. All five resource values are required funding for this form:
stock admission refuses an insufficient owner offer before spawning. Resource
requests never widen imports, storage scope, CPU authority or executable grants.
The owner still enforces conserving budgets and may refuse an otherwise valid
request. Earlier formats keep their existing semantics.

The installed Nested program uses this form. Its source is the same stock matrix;
the `nested-desktop` build selects scoped picker startup. The selected user matrix
receives the explicit stock `WORKSPACE_ARGUMENT` to run its ordinary CONFIGURE
path. Neither choice changes how the supplied environment is authenticated.
