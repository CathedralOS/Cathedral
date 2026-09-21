# Selecting one normal Field write chunk

Status: **tested**. All 629 checked-interpreter positive/control pairs pass:
175 selected-chunk and 454 unchanged bulk pairs. Exact verification passes all
64 source hashes, generated batches and the pinned runner binary. This extends the existing
detached write algorithm with `write::assemble_chunk`. Both public assembly forms
share the same extraction and surrounding-bit merge. It prepares ordinary numeric
data and does not complete interpreter field writes or change the ACPI-005 task
status.

The modified translation derives from rust-osdev/acpi
[`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, `src/aml/mod.rs:2612`](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2612),
copyright 2018 Isaac Woods, MIT OR Apache-2.0. The existing
[inventory](inventory.json), [notices](../../../../THIRD_PARTY_NOTICES.md) and
[licenses](../../../../licenses/rust-osdev/acpi/) retain the source mapping.
`write.omg` and `model.omg` contain the modified algorithm and result shape;
the new [fixture directory](../../../../tools/ports/acpi/field-write-chunks/README.md)
contains original host vectors and authored Omega assertions.

## Contract

`assemble_chunk(kind, field, region_bytes, size, payload, payload_length, index,
previous)` recomputes the complete canonical geometry. A bad later footprint
therefore prevents even an earlier selected word from being returned. Next it
requires the exact logical field-sized payload length, then checks the unsigned
chunk index against the internally planned count and 257-slot capacity. No
caller-supplied Plan or Chunk is accepted.

Geometry errors take precedence over payload errors, which precede index and
merge errors. An invalid index returns `WriteError::Chunk` with that exact index
and `InvalidChunk`, including high-bit and maximum unsigned values. A partial
Preserve chunk without a supplied word returns `NeedsPrevious` at its own index.
Other chunks' previous values are irrelevant and are never supplied to this API.
The shared `CountMismatch` error remains used only by the bulk API.

Success is `ChunkWriteResult::Write {record, lock}`. `record` contains the selected
native byte offset, width and normalized word; `lock` retains the geometry's unmet
Global Lock requirement. Default initialization yields the same failure-first
`Geometry {InvalidFlags}` shape as the existing bulk result. All input borrows
are read-only. No partial array or mutable continuation is exposed.

The input bytes are an already converted logical bit vector. This API retains
the [bulk algorithm's limits](PORT.md): normal fields, 1–2048 bits, native
byte/word/dword/qword access, strict metadata and full-region bounds, and at most
257 native chunks. It adds no AML source conversion, repeated wider-Buffer Store,
Bank/Index transfer, name lookup or source ownership admission.

## Ordering and authority

The pinned implementation reads an old native word when Preserve needs it,
merges the selected source bits, writes that word, then advances. Selecting one
chunk lets a future controller retain that order without collecting every old
word before issuing writes. A supplied `Previous` number remains data, with no
claim that a read happened or that an observation is fresh.

[ACPI 6.6 §19.6.48](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#field-declare-field-objects)
defines native access alignment, surrounding-bit updates and locking. Actual
provider access, per-chunk acknowledgments, partial effects, synchronization,
device semantics and access grants remain separate obligations in
[ADAPTER.md](../ADAPTER.md). A successful recipe cannot authorize a write or make
a register suitable for a read-modify-write operation.

## Evidence boundary

Checked execution took 1052.578 seconds wall time and 3110.472 seconds summed
batch time across three workers and 64 packages. Maximum evaluator fuel was
789,104 per body. Every positive returned zero and every changed-expectation
control returned one after complete authored/dependency checking.

The authored corpus has 175 selected-chunk positive/control pairs. Independent
interval arithmetic covers all native widths and update rules, both Integer
sizes, lock states, first/interior/final chunks, the 257-chunk capacity, absent
Previous values, poisoned high bits, geometry/payload/index precedence and
maximum unsigned indices. Success controls alter the expected native word;
failure controls demand a different error variant. The same verification run
includes all 454 unchanged original bulk pairs, which compare all output records
and inactive tails. Ten-pair packages bound the compiler's fixture admission cost;
each package checks the full dependency bodies before executing its original
positive/control bodies.

The final receipt records exact production, fixture, generated module, build and
runner hashes, actual observations and failures, source stability and elapsed
time. The 126 original public Rust observations replay exactly with the same
source hashes, inputs, callback logs and resulting memory. A separate receipt
retains the rebuilt binary identity; it need not match the historical executable
hash. These are the existing bulk arithmetic observations, not direct witnesses
of a new upstream single-chunk API. Earlier constant proofs retain their original
source identity. No new constant, native Omega, firmware or hardware result is
claimed.
