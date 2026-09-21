# Public BufferField Store observations

These original probes execute actual pinned `Interpreter::new/load_table/evaluate`
with synthetic AML: create a named BufferField over a Buffer, then invoke
`Store(source, field)`. Backing bytes and identity are inspected through the public
namespace after evaluation or caught panic. No private algorithm is mirrored,
ObjectToken fabricated or manually acquired, or hardware accessed. All service
callbacks are trapped except construction of the inert mutex handle required by
Interpreter::new. Every row uses a fresh interpreter and process.

```sh
python3 tools/ports/acpi/aml/buffer-field-store/reference.py
```

Use `--write` only to deliberately refresh the retained receipt. `--smoke --write`
creates a separate six-case smoke receipt. The final full record is independent
of root's frozen Omega fixtures; it neither edits nor claims to execute them.

The 116 observations cover Integer and Buffer sources with valid arbitrary-bit
fields, widths through 2048 bits, empty/short/long sources, unaligned destinations,
nonuniform backing and preserved surrounding bits. Both ACPI revisions are used.
Host Integer arguments deliberately retain their raw u64 value: the pin writes
all eight bytes even for revision 1, whereas the bounded primary preparation
normalizes to four bytes. These are explicitly classified raw-width differences.
Python whole-number bit insertion independently computes expected bytes; it is
not a transcription of the pin's private copy_bits loop.

Valid ASCII String sources of lengths 0, 1, 255 and 256 reach the pin's unsupported
source panic before backing writes. The panic is caught around evaluate; immutable
backing observations after unwinding establish unchanged bytes. The probe does
not claim the whole interpreter is failure-atomic: CreateField has already run.
Only Buffer backing is used, so no String backing can acquire invalid UTF8.

Four namespace-alias probes refer to a distinct source Buffer and establish alias
identity plus preserved source bytes. **Source identical to backing is deliberately
not invoked.** The pin takes mutable source Vec access in do_store, then mutable
backing access in write_buffer_field; overlapping source/backing identities risk
conflicting mutable references through these unsafe internals. Omega's existing
checked fixtures exercise that identity safely through a detached snapshot. A
separate copied source would not be evidence of pinned same-backing behavior.

The record binds the exact execution/probe roots, Cargo manifest text and hash,
isolated binary, four probe/lock inputs, three production mapping hashes and all
27 pinned source/manifest/license hashes. Those three Omega hashes identify the
comparison scope; no Omega execution is claimed by this Rust-only record. The
probe verifies source and production hashes before/after running, and default
reproduction checks every observation, AML byte sequence and retained hash.
Pinned rust-osdev/acpi revision: 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5,
MIT OR Apache-2.0, copyright 2018 Isaac Woods. Existing repository notices apply.
