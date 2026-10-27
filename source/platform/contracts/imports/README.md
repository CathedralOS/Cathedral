# Declared services

Start at [main.omg](main.omg). `decode` validates the complete bounded record;
`bind` matches up to four typed `Import` records against an owner's offered mask.
[types.omg](types.omg) owns the boundary values. Matching returns a `Binding`:
success and the matched mask, malformed declaration, missing required interface,
or incompatible required version. Refusals retain the relevant contract ID;
they publish no binding.

Decoding descends into [record.omg](record.omg) for the header, all active entries
and unused zero padding. Its initialized 168-byte input and explicit length cover
the existing 144-byte `CTHIMP01`/`02`/`03` records and 168-byte `CTHIMP05` profile.
Other lengths and versions refuse. The header constants encode those ASCII names
as little-endian words. Broker startup, channel/descendant demand and optional
page/surface demand are decoded metadata, never authority. Reserved bits, excess
entries, non-boolean required flags, malformed intervals and duplicates refuse.
Only use the output when `decode` returns true, and only its first `count` entries.

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

The [stock policy](../../../distribution/matrix/service/children/launch/imports.omg) filters its
offer before calling this matcher: interaction/storage, plus input production
for brokered startup. Capture is supported by other hosts; it is not supplied by
the stock matrix just because its contract ID is recognized. Missing legacy
records use an explicit two-required-interface declaration at the distribution
boundary.

ELF record discovery starts at [image.omg](image.omg). `begin_image` and
`advance_image` request bounded byte spans from one immutable image, then return
absent, found or malformed. The caller supplies each requested span exactly;
terminal scans refuse further replies. The scanner checks the ELF64 header and
the complete table of at most 16 program headers, inspecting read-only load
segments for a supported record. Unknown versions, duplicate records, truncated
ranges and overflow refuse, including bad segments after an earlier match. This
is declaration discovery, not the kernel's executable validation.

Record encoding remains in Rust. The ordinary matrix uses this Omega scanner,
decoder and matcher; its host supplies checked reads from the sealed mapping. Its
[declaration adapter](../../../distribution/matrix/bridge/policy/launch/declaration.omg)
unpacks bytes and publishes metadata only after full validation. Discovery does
not substitute for content validation. Compiled
[conformance tests](../../../../source-rs/distribution/matrix/tests/imports.rs)
compare versions, optionality, duplicates, unknown identities and refusal
precedence against the Rust contract. The [decoder tests](../../../../source-rs/distribution/matrix/tests/declaration.rs)
compare record forms, bit mutations, truncation, metadata and ELF boundary
failures against the original Rust parser. The [image tests](../../../../source-rs/distribution/matrix/tests/image.rs)
compare header/prefix mutations, segment kinds and ranges, duplicate records,
table limits and reply sequencing. This is implementation evidence for
the existing bounded protocol, not a new manifest format.
