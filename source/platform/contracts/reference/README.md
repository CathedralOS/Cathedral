# Rust-reference protocol bindings

[main.omg](main.omg) names the wire subset used by installed Omega apps.
[matrix.omg](matrix.omg) adds request families for the stock Omega matrix router.
[display.omg](display.omg) names the compositor's six-word frames a root matrix
presents through.
[services.omg](services.omg) names service acquisition and the provider's
selection among preallocated rights, mirroring the Rust services contract.
These are separate imports so matrix operation names do not collide with app host
commands. The reference encodings remain owned by the Rust
[platform contracts](../../../../source-rs/platform/contracts/README.md).
These declarations import no provider implementations and expose no authority.

Compiled Colors tests replay the encoding against those Rust contracts,
including rectangle unpacking, input events and storage records. Extend the
bindings alongside conformance coverage when another Omega app needs more.
The stock matrix's compiled request tests cover the matrix bindings against the
Rust constants, including storage attachment scope bits and owner-only routing.
These experimental values are not a commitment to the final specified input,
rendering or application bootstrap wire formats.

The storage leaves validate [directory rows](storage/entries.omg) and
[canonical metadata](storage/metadata.omg), name the
[operations](storage/operations.omg) a delegated child may issue, and carry
the [scope stamping and hop deadline](storage/scope.omg) a matrix applies on
the way to the provider. [Timing](storage/timing.omg) mirrors
the provider/client deadline separation; the actual chooser process fixture
compares emitted deadlines against the Rust constants. Equal provider/client
expiry would retire nested links before the root can report a storage failure.
