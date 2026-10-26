# Rust-reference protocol bindings

[main.omg](main.omg) names the wire subset used by installed Omega apps.
[matrix.omg](matrix.omg) adds request families for the stock Omega matrix router.
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
