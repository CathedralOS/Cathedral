# CHARTER — `source/libraries/`

**Scope.** Reusable Omega algorithms and adapters that are not independently
running services. The UEFI library contains pure helpers translated from the
pinned upstream under the licensed-port policy, alongside source/test provenance.

**Depends on.** Public contracts and lower-level foundation/library packages as
explicitly declared. No privileged-core internals or ambient hardware access.

**Authority.** Parsing or normalizing bytes produces data, never firmware,
physical-memory, port-I/O, or attestation authority. Any future authority-bearing
adapter must state its grants, boundaries, and lifetime separately.

**Status.** Staged UEFI helpers; each package's `PORT.md` records its actual
compilation, test, and integration coverage. Presence here does not imply
production build inclusion.
