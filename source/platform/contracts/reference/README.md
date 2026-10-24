# Rust-reference protocol bindings

[main.omg](main.omg) names the wire subset used by the first installed Omega
app. The reference encodings remain owned by the Rust
[platform contracts](../../../../source-rs/platform/contracts/README.md).
These declarations import no provider implementations and expose no authority.

Compiled Colors tests replay the encoding against those Rust contracts,
including rectangle unpacking, input events and storage records. Extend the
bindings alongside conformance coverage when another Omega app needs more.
These experimental values are not a commitment to the final specified input,
rendering or application bootstrap wire formats.
