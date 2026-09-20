# UEFI table foundations (UEFI-002)

Status: **transcribed** overall; the raw declarations and constant fixture are
**typechecked**, the constant fixture is **tested by semantic evaluation**,
while the separate layout policies remain blocked. All five raw
table source files and the added
configuration-GUID source are mapped. Full named schemas, x86-64 layout policies,
service signature metadata, slot ordinals, related scalar representations, and
vectors are present. Source inventory and upstream Rust ABI checks pass. Omega
semantic checking reaches a confirmed 32-field reflection implementation limit
in the 45-member BootServices plan; no Omega ABI comparison, firmware execution,
or Cathedral integration is claimed.

## Provenance and scope

- Upstream: https://github.com/rust-osdev/uefi-rs
- Pin: `c0facddf9ba42b74906a37fca2869e6cdbc8da6a`.
- Preserved license: `MIT OR Apache-2.0`; Copyright (c) The uefi-rs contributors.
  See [third-party notices](../../../../THIRD_PARTY_NOTICES.md) and the exact
  [uefi-raw texts](../../../../licenses/rust-osdev/uefi-rs/uefi-raw/).
- Translation changes: Rust function/pointee pointers become inert `addr`
  members; function signatures are retained as adjacent metadata. Explicit
  x86-64 layout plans in `tables_layouts.omg` and flat constant names replace Rust representation
  attributes and associated constants. Source-file notices identify these
  modifications.
- Primary specification: [UEFI 2.11, sections 4.2–4.6](https://uefi.org/specs/UEFI/2.11/04_EFI_System_Table.html),
  table header, system table, boot/runtime services, and configuration entries;
  sections 7 and 8 own service contracts. Signatures and geometry are not
  permission to invoke services. The 15 configuration GUIDs in `uefi/src/table/cfg.rs`
  include PI/vendor/legacy identities as well as standard UEFI configuration
  identities; inclusion does not claim that each is a UEFI 2.11 table type.
- Omega documents consulted before authoring: `wiki/spec/layouts/plans.md`,
  `wiki/spec/build/{boundary_shapes,foreign_storage,uefi_entry}.md`,
  `omega-rust/psi/semantics/build-time-evaluation/layouts.md`, and the authored
  `source/library/std/targets/uefi_x86_64/tables.omg` in the sibling Omega checkout.
- Cathedral documents consulted: `wiki/spec/boot/uefi_boot_services.md` and
  existing `source/contracts/uefi/{uefi,boot_services}.omg`.

## Source map

All paths below are relative to the pinned upstream repository. The exact
file hashes, lexical symbols, dispositions, and local anchors are in
[tables-inventory.json](tables-inventory.json).

| Upstream source | Destination and disposition |
|---|---|
| `uefi-raw/src/table/header.rs` | `tables.omg`: Header, every field, CRC polynomial, and HeaderX64Layout (in tables_layouts.omg). |
| `uefi-raw/src/table/system.rs` | `tables.omg`: SystemTable, all fields, SIGNATURE renamed SYSTEM_TABLE_SIGNATURE, SystemTableX64Layout (in tables_layouts.omg). Rust Default deliberately omitted below. |
| `uefi-raw/src/table/configuration.rs` | `tables.omg`: ConfigurationTable and its layout. |
| `uefi-raw/src/table/boot.rs` | `tables.omg`: complete BootServices, slot constants, every original service signature, EventNotifyFn inert carrier, OpenProtocolInformationEntry. `scalars.omg` owns AllocateType, EventType, InterfaceType, MemoryAttribute, MemoryDescriptor, MemoryType, Tpl, TimerDelay, PAGE_SIZE and their constants. |
| `uefi-raw/src/table/runtime.rs` | `tables.omg`: complete RuntimeServices, every service signature, slot constants, ResetType, TimeCapabilities, VariableAttributes, VariableVendor and both vendor GUIDs. |
| `uefi/src/table/cfg.rs` | `tables.omg`: all 15 configuration GUIDs, PropertiesTable, MemoryProtectionAttribute. ConfigTableEntry is deliberately consolidated with ConfigurationTable; `guid` maps to `vendor_guid`, `address` to `vendor_table`. |

The boot table has 44 pointer-sized positions after its header: 43 function
slots plus the reserved pointer position. The runtime table has 14 function
slots. `reserved` remains present and has an ordinal/offset vector but no
callable interpretation. Every field remains in upstream/specification order.
No array replacement hides the named boot members to satisfy a compiler limit.

`Header` uses upstream field names `size` and `crc`, corresponding to Cathedral's
older `header_size` and `crc32`. SystemTable `stdin/stdout/stderr` correspond to
existing `con_in/con_out/std_err`; all are inert address data here. Existing
boot-facing `EfiSystemTable` and `BootServicesTable` remain separate production
contracts, with their current admission and lifecycle policy. This corpus is
not imported by those contracts or any production build root.

## Representations and deliberate deviations

- The target is specifically UEFI x86-64, little-endian, 64-bit pointers and
  UINTN. Layout policies have public named `Layout` conformances and static
  `plan(schema) satisfies Layout::plan` implementations in `tables_layouts.omg`.
  They are explicit requested plans; plain Omega data
  declarations alone do not establish these byte positions. Header revision,
  handles, and GUIDs use the scalar slice's nominal carriers.
- TimeCapabilities `sets_to_zero` is an uninterpreted `u8` Boolean byte. Nonzero
  truth interpretation belongs to scalar Boolean helpers. Padding after it is
  a plan gap, not an invented firmware field.
- Runtime scalar wrappers retain unknown values and flag bits. ResetType is a
  u32 carrier, not a closed Omega sum; VariableVendor wraps Guid. EventNotifyFn
  wraps an address, not an executable Omega machine identity.
- The two multiple-protocol variadic signatures preserve upstream's `extern
  "C"` metadata and its distinction from `efiapi`. No variadic or callback
  boundary is invented. The nonreturning `reset_system` result remains `!` in
  signature metadata. Signature comments retain const/mut and pointer-depth
  information lost from the inert numerical slot carrier itself.
- Rust Clone/Debug/Eq/Ord/Hash/Default derives, bitflags operator generation,
  and newtype-enum macro ergonomics are not ABI. This slice exports raw
  carriers/constants rather than recreating that API. In particular it omits
  SystemTable::default: a mostly zero table is not a firmware-admitted table.
  MemoryDescriptor defaults, anonymous layout assertions, and MemoryType::custom
  belong to scalar verification, not duplicate table helpers.
- PropertiesTable is the pinned safe crate's legacy UEFI properties shape;
  it is not substituted for the newer EFI_RT_PROPERTIES_TABLE.
- This slice performs no pointer dereference, service invocation, callback
  registration, GUID-based authority establishment, or CRC admission. Later
  producer/consumer fixtures and reviewed boundary contracts own those tasks.

## Verification and tests

```sh
python3 tools/ports/inventory.py check source/contracts/uefi/raw/tables-inventory.json --checkout reference_code/rust-osdev/uefi-rs --require-transcribed
python3 tools/ports/vectors.py source/contracts/uefi/raw/tables.vectors.json
python3 tools/ports/uefi-tables/check.py
python3 tools/ports/uefi-tables/semantic-check.py
../Omega/target/debug/omega --check tools/ports/uefi-tables/main.omg
../Omega/target/debug/omega --check source/contracts/uefi/raw/tables_layouts.omg
```

The inventory passes with six mapped files, 207 translated lexical anchors,
four deliberate omissions, and no pending symbols. The scanner does not expand
Rust macros or prove Omega behavior. The host checker additionally verifies all
87 fields of the seven raw records, exact field order, boot/runtime slot ordinals,
17 GUID numeric/byte encodings against upstream, and authored geometry against
vectors. It builds a small `no_std` probe against the actual pinned `uefi-raw`
crate for **x86_64-unknown-uefi**, then extracts 101 independently compiler-evaluated
size/alignment/offset values from its LLVM artifact. All 101 agree. This is a
cross-target compile, not host-layout substitution or firmware execution.
The probe's companion scalar report measures Guid (16,4), CapsuleHeader (28,4),
Time (16,4), and MemoryDescriptor (40,8), as size/alignment pairs.

The pinned five raw table sources have no separate `#[test]` cases beyond
boot.rs's anonymous MemoryDescriptor layout assertions, which are preserved in
the scalar slice. The new raw-table probe covers sizes, alignments, every
field offset, and first/last/reserved slot positions. `tables.vectors.json`
also preserves signature values, runtime enums/flags, the configuration GUIDs,
and the legacy PropertiesTable geometry. Its PropertiesTable expectations are
transcribed, not measured by the uefi-raw probe, because that type belongs to
the separate `uefi` crate. Standard signature identities and runtime constants
are source-review expectations; no runtime table validation is implied.

The executable Omega constant fixture is
`tools/ports/uefi-tables/main.omg`. Its actual package import and the raw
module pass Omega `--check`. `semantic-check.py` evaluates `test_result()`
as an Omega constant and requires its result to equal zero; a temporary negative
control adds one and rejects with the exact `0 + 1 == 0` obligation. Both checks
pass. This is semantic-evaluator execution, not native execution. The explicit layout policies are
kept in a separate module so that their blocker does not hide this semantic
check. Constant-fixture checking is not object-layout or firmware evidence. No
upstream operation was converted into a fake successful test. Existing production canaries are
unaffected by this isolated package; running the legacy production harness is
root-task verification, not evidence supplied by this table-only check.

## Confirmed Omega blockers and remaining evidence

`PORT-BLOCKED[omega:layout-reflection-capacity]`: the authored BootServices plan
requires schema member keys 0 through 44. Current `Schema.fields` has length
32 (`Omega/source/library/core/layout.omg:65`). Running the command above reports `cannot prove index 32 is within length
32` through index 44. The sibling Omega authored UEFI table explicitly works
around the same limit with a `[u64;44]` service array. A general larger/complete
schema reflection capability is needed for this full named source plan. This is
an implementation limit, not missing Cathedral engineering.

`PORT-BLOCKED[omega:layout-address-carrier]`: the current authored Omega UEFI
layout source explicitly notes that materialization has no `addr` carrier and
therefore uses u64. The specification permits inert `addr`, but actual
materialization of these named addr-containing records still needs supported
schema/consumer evidence. No forged callable pointer or reference substitutes
for that missing evidence.

Once those mechanisms are available, evaluate all policies, collect independent
Omega size/alignment/offset/value output with compiler revision and artifact
hash, and run vectors.py `--observed`. Until then the Rust probe establishes
upstream ABI agreement only. No integration, hardware run, or service authority
is established by the source status or the passing inventory.
