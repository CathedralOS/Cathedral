# Declared service matching

Start at [main.omg](main.omg). `bind` takes up to four typed `Import` records and
an owner's offered interface mask. It returns a `Binding`: success and the matched
mask, malformed declaration, missing required interface, or incompatible required
version. Refusals retain the relevant contract ID; they publish no binding.

This mirrors the bounded Rust reference's common contract in
[imports.rs](../../../../source-rs/platform/contracts/imports/imports.rs).
Known interfaces currently have version 1. Unknown optional interfaces remain
absent; unknown required interfaces refuse. A composite mask is not an interface
identity. Duplicate identities and invalid version intervals are malformed.
Matching keeps the reference's ordered refusal behavior; image decoding validates
all declaration syntax separately before admission.

This package has no distribution dependency, process identity, allocation or
endpoint lookup. A matched bit says only that the supplied offer satisfies that
declaration. Actual authority and provider behavior remain separate. The API uses
typed records; the stock matrix's temporary native packet is not this contract.

The [stock adapter](../../../distribution/matrix/launch/imports.omg) filters its
offer before calling this matcher: interaction/storage, plus input production
for brokered startup. Capture is supported by other hosts; it is not supplied by
the stock matrix just because its contract ID is recognized. Missing legacy
records use an explicit two-required-interface declaration at the distribution
boundary.

Record encoding/decoding and ELF extraction still run in Rust. The ordinary
matrix now uses this Omega matcher after reading a sealed image. Compiled
[conformance tests](../../../../source-rs/distribution/matrix/tests/imports.rs)
compare versions, optionality, duplicates, unknown identities and refusal
precedence against the Rust contract. This is implementation evidence for the
existing bounded protocol, not a new manifest format.
