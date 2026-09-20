# Bounded ACPI header and root-entry slice (ACPI-001)

## Status and profile

**Stage: tested by Omega semantic evaluation, 2026-09-20.**

This is the first pure parser slice, separate from the complete ACPI-000 inventory.
It implements RSDP, SDT headers and indexed RSDT/XSDT decoding from initialized
bytes. No production build root imports this package. Native execution, native
record layout, firmware discovery and physical mapping are not claimed.

Every input is a shared reference to an initialized `[u8; 4096]` plus an explicit
available length. Lengths above 4096 fail before parsing. This is a local resource
profile for the first slice, **not an ACPI maximum table length**. Larger tables
need a future interface/profile extension. Defaults initialize all result fields;
only `error == ERROR_OK` means successful parsing. No returned value proves a
mapping, valid physical address, address width of a machine, or table lifetime.

Supported RSDP revisions are 0 and 2; other revisions return an explicit error.
Generic SDT revision bytes are preserved; RSDT/XSDT require revision 1. Expected
SDT signatures are caller-supplied exact four-byte values represented as a
little-endian u32; this does not establish registry membership. All 65 pinned
signature constants are present, including FADT=`FACP` and MADT=`APIC` aliases.
The signature registry does not mean all tables share the SDT header (FACS does
not); callers select an appropriate table parser.

## Pin and licensing

Derivative data/validation logic comes from rust-osdev/acpi 6.1.1 at
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, principally `src/rsdp.rs`,
`src/sdt/mod.rs`, and the pure root-entry/selection parts of `src/lib.rs`.
Retained grants are [MIT](../../../licenses/rust-osdev/acpi/LICENCE-MIT) and
[Apache-2.0](../../../licenses/rust-osdev/acpi/LICENCE-APACHE), with
Copyright (c) 2018 Isaac Woods. Modified source files identify the pin and changes.
The complete licensing audit is [PORT.md](PORT.md); neither external uACPI
examples nor captured firmware/ASL/AML test content was copied. Synthetic test
bytes were authored locally from the wire format.

Primary wire requirements are
[ACPI 6.6 chapter 5](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html),
sections 5.2.1, 5.2.5.3, 5.2.6, 5.2.7, and 5.2.8. Numbers use little-endian byte order.
RSDP has a 20-byte legacy portion and a 36-byte extended prefix. SDT headers are
36 bytes; root entries are 4 or 8 bytes. Checksums cover the appropriate declared
extent. Reader code preserves and ignores reserved RSDP bytes, as required by
the general reserved-field reading rule; it does not reject nonzero reserved bits.

## Source map

[headers-inventory.json](headers-inventory.json) binds all anchors and complete
hashes of the three reviewed files. Omissions are scoped to this first slice;
the full queue inventory still lists subsequent work. A mapped-table method is
not marked translated merely because its pure suboperation has been extracted.

| Source | Omega representation |
| --- | --- |
| `Rsdp`, fields/accessors, lengths/signature | `headers.omg::Rsdp`, constants and owned fields |
| `Rsdp::validate` | `parser.omg::parse_rsdp`, bounded strict validation |
| `SdtHeader`, fields/accessors, `Signature` constants | `headers.omg`, decoded records and byte-exact constants |
| `SdtHeader::validate` | `parser.omg::parse_sdt` |
| `AcpiTables::table_entries` | `parse_root` and revalidating `root_entry`; index lookup replaces unsafe iterator |
| `AcpiQuirks::ignore_xsdt` | explicit `select_root` argument; result is inert numeric data |
| BIOS search, mappings, `Handler`, allocation and callbacks | omitted from this pure slice; no authority replacement |
| `ExtendedField`, fixed table modules | deferred to bounded fixed-table parsing in ACPI-002 |
| Rust formatting/borrowed strings | omitted; owned bytes/integers preserve the information |

`bytes.omg` provides explicit little-endian decoding and bounded modulo 256
checksums. Eight-byte checksum blocks reduce evaluator work; a checked
`Nat::BoundedDistance` ranking bounds both block and remainder scans. Reader
helper preconditions and explicit offset guards establish array-index bounds.

## Deliberate differences and limits

- The pin's extended validation checks one fixed 36-byte sum and can miss a bad
  legacy checksum repaired in the extended checksum. This parser independently
  validates the first 20 bytes and the entire declared extended extent.
- Extended length must be at least 36 and within the available bytes. A revision 2
  extension beyond 36 bytes participates in its checksum. Revision 0 uses exactly
  20 bytes; its unavailable extension fields are zero and decoded length is 20.
- The pin's mapped-table constructors may continue after checksum errors.
  This parser reports checksum errors and never continues discovery from a failed result.
- OEM and OEM-table fields remain exact bytes, including non-ASCII data. The
  primary header definitions do not require a reader-side UTF-8 validity gate.
  This differs from the pin's validation/borrowed string convenience: formatting
  or text decoding must be a separate future API. Creator ID remains four exact
  bytes encoded in a u32, with no string conversion.
- Root payload length must be an exact multiple of its entry width; the pin's
  integer-division iterator can silently truncate a trailing partial entry.
- `root_entry` validates its source each call before checking the requested
  index. Even the largest u64 index fails before offset multiplication. Caller
  construction of a `RootResult` cannot grant permission to read bytes.
- `select_root` prefers a nonzero XSDT for revision 2 unless the explicit quirk is
  set, then falls back to RSDT. Zero root addresses are reported unavailable.
  No address is dereferenced, canonicalized, masked or widened into authority.

## Tests and verification

The two upstream header files have no direct pure unit tests. The generic
`src/lib.rs` Send/Sync mapping test is outside this parser slice, and is explicitly
omitted in the narrow source map. These fixtures derive from the primary wire
requirements and the reviewed implementation differences.

`tools/ports/acpi/header_fixtures.py` generates original initialized inputs and
explicit expected outcomes, not a Python replacement for the Omega parser.
`main.omg` source-checks all 65 authored cases. `check.py` executes the actual
Omega bodies in bounded groups with one constant assertion per temporary fixture.
Large capacity cases run alone. This respects the evaluator work budget and avoids
the current duplicate-selection diagnostic from multiple independent constants
that traverse the same imported states in one fixture. Tests cover truncation,
capacity, malformed signatures/lengths/revisions, both checksums, compensated
legacy corruption, extension checksums, reserved-byte preservation, raw OEM bytes,
root payload remainders, high address bits and out-of-range indexes. All seven
checksum remainder lengths and the last full-capacity root-entry slots are tested.

`check.py` also mutates actual expected address/checksum results in three
separate fixtures. Each must evaluate 1 and fail the `value == 0` contract;
changing only the final assertion would not exercise the parser body.

Expected wire vectors are in [headers.vectors.json](headers.vectors.json).
`header_evidence.py` compares all 65 constants against pinned Rust literals and
checks exact source bindings. Those 88 vectors describe serialized bytes; they
are not an observation of native Omega record layout.

```sh
python3 tools/ports/acpi/header_evidence.py --check
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi --require-transcribed source/libraries/acpi/headers-inventory.json
python3 tools/ports/vectors.py source/libraries/acpi/headers.vectors.json
python3 tools/ports/acpi/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

Compiler source revision: `eaa7993a23623cd8fabf45350340479c5c9c7879`.
Compiler binary SHA-256:
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
Observed verification: source checking passed for all 65 authored cases; all
65 scenarios passed actual Omega semantic evaluation in 21 bounded groups; all
three body-mutating negative controls failed with computed `1 == 0` as required.
Narrow inventory and all 88 expected wire vectors passed their checks. No native
or physical-firmware operation has been run. This new isolated package changes
no existing production canary or build root.
