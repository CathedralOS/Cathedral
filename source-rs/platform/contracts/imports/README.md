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

## Current bounded representation

The runtime's `service_imports!` macro emits `CTHIMP01` and a count, followed by
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

For new store programs, [matrix admission](../../services/matrix/service/children/admission.rs)
maps the immutable image, resolves its imports, unmaps it, then reserves and
installs only the selected connections. Owner `LAUNCH_IMPORTS` selects the offer
mask for future launches in a slot. A child's `LAUNCH` can additionally withhold
offers, never widen them. Reads still use the launcher's realm; granting storage
binds the admitted program to its own subtree. No-storage launches receive no
storage endpoints or realm binding. Attempts through the interaction endpoint
are denied in the desktop's separate-stream protocol.

The version-1 startup transport still uses the existing five-endpoint kernel
bundle: interaction pair, wake receiver, and optional storage pair. Three
endpoints mean storage is absent; five mean present. [Runtime views](../../libraries/user-runtime/matrix.rs)
provide required connections or `OptionalConnections` with an actual `Option`
storage pair. The compiler checks these view types, not whole-program reach or
the honesty of arbitrary executable declarations.

Old program copies without a record use the **explicit legacy declaration**:
interaction and storage both required. This preserves saved realms without
silently teaching old code to handle absent storage. Withholding storage refuses
those binaries. Boot and catalogue-based lab admission retain their existing
explicit wiring; they are not claimed to enforce this new store-launch contract.
Legacy headless matrices still multiplex storage on interaction under a scoped
binding; withholding the binding denies that access too.

## Visible proof

The F5 user launcher has STORAGE ON/OFF for subsequent launches. Notes declares
optional storage and displays TEMP NO SAVE when absent; editing and dirty-close
protection still work, while save/reload cannot claim persistence or discard a
draft. Colors requires storage, so the same offer produces NEEDS SERVICE without
occupying the slot. Restoring the offer and relaunching Notes reads saved notes.

Host tests cover declaration parsing, version compatibility, unknown contracts,
missing required and optional services, malformed records and truncated ELF.
`desktop_test.py` drives both modes of the same Notes image and cold-reopens the
disk; `--matrix-storage-test` checks owner policy, child withholding/refused
widening, successful retry, realm confinement and revocation. Service discovery,
first-request activation, dynamic grant addition and final manifest persistence
remain separate work.
