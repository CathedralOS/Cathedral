# Existing UEFI subset reconciliation (UEFI-000)

Reviewed 2026-09-20 against `uefi-rs` revision
`c0facddf9ba42b74906a37fca2869e6cdbc8da6a` and Omega revision
`eaa7993a23623cd8fabf45350340479c5c9c7879`. This is a source inventory,
not a successful Omega compilation, ABI measurement, or boot run. It describes
`uefi.omg` and `boot_services.omg` as found before the licensed raw-contract
extension. Later additive declarations must retain the distinction between this
legacy consumer view and the complete raw representation.

## Sources and scope

Upstream paths below are relative to `uefi-raw/src/` in the pinned
[uefi-rs tree](https://github.com/rust-osdev/uefi-rs/tree/c0facddf9ba42b74906a37fca2869e6cdbc8da6a/uefi-raw/src).
`boot` means `table/boot.rs`, `system` means `table/system.rs`, `header` means
`table/header.rs`, and `console` means `protocol/console.rs`.

Primary specification references:

- [UEFI 2.10 §2.3, calling conventions and scalar representations](https://uefi.org/specs/UEFI/2.10/02_Overview.html#calling-conventions).
- [UEFI 2.10 §§4.1–4.4, physical entry, header, system and boot tables](https://uefi.org/specs/UEFI/2.10/04_EFI_System_Table.html).
- [UEFI 2.10 §§7.2.1, 7.2.3 and 7.4.6, memory and exit](https://uefi.org/specs/UEFI/2.10/07_Services_Boot_Services.html).
- [UEFI 2.10 §§12.4.1–12.4.3, text output](https://uefi.org/specs/UEFI/2.10/12_Protocols_Console_Support.html?highlight=graphics+output).
- [UEFI status codes, Appendix D](https://uefi.org/specs/UEFI/2.10/Apx_D_Status_Codes.html).
- [UEFI 2.11 §7.2.3, hot-pluggable attribute](https://uefi.org/sites/default/files/resources/UEFI_Spec_Final_2.11.pdf).

Official indexed extracts were available for table and console shapes. Direct
requests for some official HTML pages and the 2.11 PDF returned HTTP 403 during
this review. Consequently numeric cross-checks below use the pinned source where
full primary text was unavailable; the hot-pluggable discrepancy is explicitly
left open below. Spec citations are reference coordinates, not a claim that every
page was retrieved successfully.

## Every existing top-level declaration

### `uefi.omg`

| Cathedral declaration | Pinned upstream counterpart | Spec / disposition |
| --- | --- | --- |
| `EfiStatus` (`code: u64`) | `status.rs::Status` (`usize` newtype) | §2.3, Appendix D; x86-64 specialization, not a target-independent UINTN. Wrapper scalar ABI needs a selected layout/calling plan. |
| `EFI_SUCCESS` | `status.rs::Status::SUCCESS` | Appendix D; 0. |
| `EFI_INVALID_PARAMETER` | `status.rs::Status::INVALID_PARAMETER` | Appendix D; error bit plus 2, hardcoded 64-bit error bit. |
| `EFI_BUFFER_TOO_SMALL` | `status.rs::Status::BUFFER_TOO_SMALL` | Appendix D; error bit plus 5, hardcoded 64-bit error bit. |
| `EfiHandle` (`raw: addr`) | `lib.rs::Handle` | §2.3; inert handle bits. The source comment “gated token” must not be read as authority originating from this public data constructor. |
| `TextOutputProtocol` | `console::SimpleTextOutputProtocol` | §12.4.1; two-field prefix, not complete protocol. Field map below. |
| `SimpleTextOutput` | `console::SimpleTextOutputProtocol::output_string` field | §12.4.3; local boundary requirement, not upstream raw data. Its `output_string` operation fixes a 19-code-unit carrier; no upstream fixed-length restriction. |
| `TextOutputProtocol::output_string_leaf` | same `output_string` function-pointer field | §12.4.3; local foreign binding; `VtableField` retains native `This`. Its shared reference and terminator obligations require separate access/custody review. |
| `EfiTableHeader` | `header::Header` | §4.2.1; complete fields, renames and unwrapped revision below. |
| `EFI_SYSTEM_TABLE_SIGNATURE` | `system::SystemTable::SIGNATURE` | §4.3.1; `0x5453595320494249`. |
| `EFI_SYSTEM_TABLE_REQUIRED_SIZE` | no upstream constant; derived from `SystemTable` x64 shape | §4.3.1; local admission threshold 120, not a universal UEFI table-size constant. |
| `EFI_MIN_SUPPORTED_REVISION` | `table/revision.rs::Revision::EFI_2_00` supplies the value | §4.3.1; choosing minimum 2.0 is Cathedral policy. |
| `EfiSystemTable` | `system::SystemTable` | §4.3.1; complete field sequence, but two raw pointers replaced by semantic references. |
| `UefiX86_64` | none | Local convention/policy subject; not a raw UEFI scalar or structure. |
| `UefiX86_64CallingPolicy` | none | Local named conformance to Omega `CallingPolicy`. |
| `UefiX86_64::plan` | none | Local semantic-entry calling policy. Nested states `accept`, `build`, `reject` are policy implementation, not UEFI functions. |
| `UefiApplication` | none | Local semantic `ProgramStorageEntry` requirement. It is not the physical `EFI_IMAGE_ENTRY_POINT` of §4.1.1. |

`UefiX86_64::plan` accepts two semantic arguments and no result, assigning
indirect copies through RCX/RDX. The physical firmware entry instead receives
image-handle bits and a System Table pointer and returns EFI_STATUS. Sharing
register names does not make these entry schemas interchangeable. The generated
shell and source adapter must connect them under the separate entry contract.

### `boot_services.omg`

| Cathedral declaration | Pinned upstream counterpart | Spec / disposition |
| --- | --- | --- |
| `EFI_LOADER_DATA` | `boot::MemoryType::LOADER_DATA` | §7.2.1; 2. |
| `EFI_BOOT_SERVICES_CODE` | `boot::MemoryType::BOOT_SERVICES_CODE` | §7.2.1; 3. |
| `EFI_BOOT_SERVICES_DATA` | `boot::MemoryType::BOOT_SERVICES_DATA` | §7.2.1; 4. |
| `EFI_CONVENTIONAL_MEMORY` | `boot::MemoryType::CONVENTIONAL` | §7.2.1; 7; not a memory grant. |
| `EFI_ACPI_RECLAIM` | `boot::MemoryType::ACPI_RECLAIM` | §7.2.1; 9. |
| `EFI_MAX_MEMORY_TYPE` | `boot::MemoryType::MAX` | §7.2.1; 16; standard sentinel, not an admissible allocation type. |
| `EFI_OEM_MEMORY_TYPE_MIN` | lower endpoint of `boot::MemoryType::RESERVED_FOR_OEM` | §7.2.1; `0x70000000`; local threshold combines OEM and OS-loader ranges for input validation. |
| `EFI_MEMORY_DESCRIPTOR_VERSION` | `boot::MemoryDescriptor::VERSION` | §7.2.3; 1. |
| `EfiMemoryAttribute` (`bits: u64`) | `boot::MemoryAttribute` | §7.2.3; open bit carrier, not a closed sum. |
| `EFI_MEMORY_RUNTIME` | `boot::MemoryAttribute::RUNTIME` | §7.2.3; `0x8000000000000000`. |
| `EFI_MEMORY_HOT_PLUGGABLE` | `boot::MemoryAttribute::HOT_PLUGGABLE` | UEFI 2.11 §7.2.3; both sources use `0x100000`; primary-text discrepancy below. |
| `EFI_MEMORY_SP` | `boot::MemoryAttribute::SPECIAL_PURPOSE` | §7.2.3; `0x40000`. |
| `EFI_MEMORY_ISA_VALID` | `boot::MemoryAttribute::ISA_VALID` | §7.2.3; `0x4000000000000000`. |
| `EFI_MEMORY_ISA_MASK` | `boot::MemoryAttribute::ISA_MASK` | §7.2.3; `0x0ffff00000000000`. |
| `EFI_MEMORY_RESERVED_MASK` | no named upstream constant; complement of pinned `MemoryAttribute` bits | Local strict-admission policy: `0x30000fffffe00fe0`. Its accepted set must be versioned; not an invariant of all future descriptors. |
| `EfiMemoryDescriptor` | `boot::MemoryDescriptor` | §7.2.3; known prefix, explicit upstream padding omitted semantically. |
| `MapKey` (`raw: u64`) | `usize` output/input of `boot::BootServices::{get_memory_map,exit_boot_services}` | §§7.2.3, 7.4.6; local wrapper, not upstream named raw type. Equal bits do not preserve transaction identity. |
| `BootServicesTable` | `boot::BootServices` | §4.4.1; prefix ending at `exit_boot_services`, detailed below. |
| `EFI_BOOT_SERVICES_SIGNATURE` | no named constant in pinned `boot.rs`; spec `EFI_BOOT_SERVICES_SIGNATURE` | §4.4.1; `0x56524553544f4f42`. |
| `EFI_BOOT_SERVICES_REQUIRED_SIZE` | no upstream constant; derived consumed prefix | §4.4.1; local admission threshold 240; full x64 shape is larger. |
| `BootServices` | two fields in `boot::BootServices` | Local boundary trait; `get_memory_map` and `exit_boot_services` operations are policy-facing projections. Name collides with upstream raw table name. |
| `BootServicesTable::get_memory_map_leaf` | `boot::BootServices::get_memory_map` | §7.2.3; table receiver locates callee and is absent from native argument list. Fixed 65536-byte storage carrier is local. |
| `BootServicesTable::exit_boot_services_leaf` | `boot::BootServices::exit_boot_services` | §7.4.6; table receiver absent from native arguments; actual arguments are image handle and map key. |
| `FinalMemoryMap` | no `uefi-raw` equivalent | Local reserved post-exit carrier (`buffer: &[u8]`, `descriptor_size: u64`); bare slice is not a raw foreign ABI and fields alone prove no finality, key freshness, custody, or successful exit. |

## Field correspondence and expected x64 geometry

Offsets below are derived review expectations for C-compatible x86-64 placement,
not measured Omega output. `addr`, pointers and UINTN occupy eight bytes in this
profile; scalar/wrapper equivalence still requires explicit validated plans.

### Header and System Table

`EfiTableHeader` → `header::Header` has size 24, alignment 8:

| Cathedral field | Upstream field / spec member | Offset |
| --- | --- | ---: |
| `signature` | `signature` / Signature | 0 |
| `revision` | `revision: Revision` / Revision | 8 |
| `header_size` | `size` / HeaderSize | 12 |
| `crc32` | `crc` / CRC32 | 16 |
| `reserved` | `reserved` / Reserved | 20 |

`EfiSystemTable` → `system::SystemTable` has expected size 120, alignment 8:

| Cathedral field | Upstream field / spec member | Offset |
| --- | --- | ---: |
| `header` | `header` / Hdr | 0 |
| `firmware_vendor` | `firmware_vendor` / FirmwareVendor | 24 |
| `firmware_revision` | `firmware_revision` / FirmwareRevision | 32 |
| `console_in_handle` | `stdin_handle` / ConsoleInHandle | 40 |
| `con_in` | `stdin` / ConIn | 48 |
| `console_out_handle` | `stdout_handle` / ConsoleOutHandle | 56 |
| `con_out` | `stdout` / ConOut | 64 |
| `standard_error_handle` | `stderr_handle` / StandardErrorHandle | 72 |
| `std_err` | `stderr` / StdErr | 80 |
| `runtime_services` | `runtime_services` / RuntimeServices | 88 |
| `boot_services` | `boot_services` / BootServices | 96 |
| `number_of_table_entries` | `number_of_configuration_table_entries` / NumberOfTableEntries | 104 |
| `configuration_table` | `configuration_table` / ConfigurationTable | 112 |

All pinned fields are present. Padding at 36–39 is implicit. The source's older
“trailing service pointers unread at milestone 1” comment does not describe a
missing suffix. `con_out` and `boot_services` are authority-bearing reference
views in this consumer declaration; raw upstream contains pointers. A new raw
shape must not infer reference validity from those pointer bytes.

### Text output

`TextOutputProtocol.reset` → `console::SimpleTextOutputProtocol.reset` /
`Reset` at 0; `output_string` → `output_string` / `OutputString` at 8.
The current two-word prefix is 16 bytes, alignment 8. Missing fields, in order:
`test_string` (16), `query_mode` (24), `set_mode` (32), `set_attribute` (40),
`clear_screen` (48), `set_cursor_position` (56), `enable_cursor` (64), `mode` (72).
The complete x64 protocol is 80 bytes. Upstream also supplies `GUID` and
`SimpleTextOutputMode`; neither is declared in these two Cathedral files.

The raw OutputString signature is `(This, CHAR16*) -> EFI_STATUS`; the
19-element borrowed carrier is one native pointer, not an inline array.
Its length alone does not prove a terminator. Upstream gives `This` mutable
pointer shape; a shared Omega `&TextOutputProtocol` cannot authorize arbitrary
foreign mutation, including changes in the omitted mode state. Preserve this
as an adapter/access obligation rather than copying Rust `unsafe`.

### Memory descriptor

`EfiMemoryDescriptor` → `boot::MemoryDescriptor`, expected size 40, alignment 8:

| Cathedral field | Upstream field / spec member | Offset |
| --- | --- | ---: |
| `kind` | `ty: MemoryType` / Type | 0 |
| no semantic field (implicit padding) | `padding: u32` / ABI padding | 4 |
| `physical_start` | `phys_start` / PhysicalStart | 8 |
| `virtual_start` | `virt_start` / VirtualStart | 16 |
| `number_of_pages` | `page_count` / NumberOfPages | 24 |
| `attribute: u64` | `att: MemoryAttribute` / Attribute | 32 |

Upstream explicitly asserts size/alignment/offsets and initializes padding to
zero. Cathedral needs an explicit layout/padding initialization rule for producer
fixtures. Runtime `DescriptorSize` governs traversal even when it exceeds 40;
rejecting unknown versions and requiring stride divisible by eight are current
Cathedral consumer admission policy, not a replacement for raw representation.

### Boot Services prefix and missing suffix

`BootServicesTable.hdr` maps to `boot::BootServices.header` / `Hdr` at offset 0.
Every remaining current field maps to the identically spelled upstream field;
all are declared as `addr`, including the reserved non-function slot. Slot
indices below are zero-based after the 24-byte header.

| Cathedral/upstream field | Spec member | Slot | Offset |
| --- | --- | ---: | ---: |
| `raise_tpl` | RaiseTPL | 0 | 24 |
| `restore_tpl` | RestoreTPL | 1 | 32 |
| `allocate_pages` | AllocatePages | 2 | 40 |
| `free_pages` | FreePages | 3 | 48 |
| `get_memory_map` | GetMemoryMap | 4 | 56 |
| `allocate_pool` | AllocatePool | 5 | 64 |
| `free_pool` | FreePool | 6 | 72 |
| `create_event` | CreateEvent | 7 | 80 |
| `set_timer` | SetTimer | 8 | 88 |
| `wait_for_event` | WaitForEvent | 9 | 96 |
| `signal_event` | SignalEvent | 10 | 104 |
| `close_event` | CloseEvent | 11 | 112 |
| `check_event` | CheckEvent | 12 | 120 |
| `install_protocol_interface` | InstallProtocolInterface | 13 | 128 |
| `reinstall_protocol_interface` | ReinstallProtocolInterface | 14 | 136 |
| `uninstall_protocol_interface` | UninstallProtocolInterface | 15 | 144 |
| `handle_protocol` | HandleProtocol | 16 | 152 |
| `reserved` | Reserved | 17 | 160 |
| `register_protocol_notify` | RegisterProtocolNotify | 18 | 168 |
| `locate_handle` | LocateHandle | 19 | 176 |
| `locate_device_path` | LocateDevicePath | 20 | 184 |
| `install_configuration_table` | InstallConfigurationTable | 21 | 192 |
| `load_image` | LoadImage | 22 | 200 |
| `start_image` | StartImage | 23 | 208 |
| `exit` | Exit | 24 | 216 |
| `unload_image` | UnloadImage | 25 | 224 |
| `exit_boot_services` | ExitBootServices | 26 | 232 |

The existing shape ends at byte 240. The complete pinned table continues:

| Missing upstream field | Slot | Offset |
| --- | ---: | ---: |
| `get_next_monotonic_count` | 27 | 240 |
| `stall` | 28 | 248 |
| `set_watchdog_timer` | 29 | 256 |
| `connect_controller` | 30 | 264 |
| `disconnect_controller` | 31 | 272 |
| `open_protocol` | 32 | 280 |
| `close_protocol` | 33 | 288 |
| `open_protocol_information` | 34 | 296 |
| `protocols_per_handle` | 35 | 304 |
| `locate_handle_buffer` | 36 | 312 |
| `locate_protocol` | 37 | 320 |
| `install_multiple_protocol_interfaces` | 38 | 328 |
| `uninstall_multiple_protocol_interfaces` | 39 | 336 |
| `calculate_crc32` | 40 | 344 |
| `copy_mem` | 41 | 352 |
| `set_mem` | 42 | 360 |
| `create_event_ex` | 43 | 368 |

The complete table has 44 pointer-width slots (one reserved), expected size 376,
alignment 8. Raw address-slot transcription can preserve this geometry without
pretending it implements callable function values. Each eventual invocation must
retain its actual signature, calling policy, access and lifetime contract. The
two multiple-protocol operations are variadic upstream and cannot be replaced
by invented fixed argument lists.

## Collisions, ownership and migration

- Existing `BootServices` names a boundary trait; upstream `BootServices` names
  the raw table. Reuse `BootServicesTable` only through an additive/migrated ABI
  change, or give the complete raw schema an explicit distinct name. Do not
  silently redefine a 240-byte prefix as a complete 376-byte owned value.
- `TextOutputProtocol` is also a prefix. Filling it out changes the size and
  field reach of existing references; complete raw shapes and checked consumer
  views need an explicit migration.
- `EfiStatus`, `EfiHandle`, `EfiMemoryAttribute`, `MapKey`, and `EfiTableHeader`
  already own their local names. Do not add duplicate upstream-spelled wrappers
  without recording whether the new type is a raw representation or an adapter.
- `UefiX86_64`, its conformance and machine, `UefiApplication`, both service
  boundary traits and their leaves, fixed buffer/string capacities, revision
  thresholds, reserved-bit rejection, retry count, and `FinalMemoryMap` are
  local policy or invocation surfaces. They must not become purported generic
  translations of `uefi-raw`.
- The contracts charter permits definitions, not executing implementation.
  Existing calling-policy construction is already embedded here. Record that
  ownership debt before moving anything; this inventory performs no relocation.
- `ExitBootServices` invalidates Boot Services, not all firmware services.
  The comment “firmware is gone” is too broad: Runtime Services require a
  separate surviving contract. Neither a final map nor address geometry grants
  Cathedral RAM authority.

## Language boundary, engineering work and source drift

Read together with Omega [layouts](../../../../Omega/wiki/spec/layouts/plans.md),
[boundary shapes](../../../../Omega/wiki/spec/build/boundary_shapes.md),
[calling plans](../../../../Omega/wiki/spec/build/calling_plans.md), and
[foreign storage](../../../../Omega/wiki/spec/build/foreign_storage.md).

1. **Actual implementation gap: named calling-policy evidence.** The specified
   surface is `Calling<C, Policy>`, but Omega's
   `omega-rust/omega/representations/calling-conventions/README.md` explicitly
   records that `std/calling.omg` still exposes `Calling<C>`. Keep the exact
   blocker `PORT-BLOCKED[omega:named-calling-policy]: select an exact named
   CallingPolicy conformance on each boundary requirement`. Merely adding the
   second argument is not verified implementation support.
2. **Source migration, not a new language design blocker.** The legacy
   `uefi_x64 machine` modifier attaches convention to a leaf, whereas the current
   contract selects policy on its requirement. Omega retains legacy fixtures
   using `uefi_x86_64`; that evidence does not validate Cathedral's spelling or
   all modern semantics. Update source only as an explicit integration migration.
3. **Engineering obligation: explicit representation.** These source files
   refer in comments to a use-site `Uefi` policy but do not attach or author it
   here. Omega documents static `data T in Policy`, fixed `At` geometry and
   recursive records/arrays. Authoring x64 raw plans/vectors is engineering;
   a missing local policy is not a language blocker.
4. **Actual broader design/implementation limits, not blockers to these fixed
   prefixes:** Omega's layout specification leaves programmable unions and
   runtime-stride source forms unspecified. The implementation note separately
   restricts structured materialization of references/dynamic shapes. Record
   `PORT-BLOCKED[omega:programmable-overlays]` or
   `PORT-BLOCKED[omega:runtime-layout-strides]` only at a future raw declaration
   actually requiring those semantics. A bounded byte parser can remain useful
   while that representation seam is open.
5. **Adapter engineering:** establish pointer backing, permitted foreign writes,
   terminators, exact buffer bounds, descriptor stride, boot lifetime, map-key
   provenance and exit receipts. Existing Omega/Cathedral specifications already
   describe these obligations. Their absence from this raw inventory is not an
   owner design question and does not justify blocking inert ABI transcription.
6. **Verification tooling gap:** existing Cathedral canaries assume historical
   `omega-cli` commands/artifact reports. No current Omega invocation was run for
   this inventory. Repairing the harness is engineering; it must not be reported
   as an inherent language limitation.

## Outstanding primary-source discrepancy

Pinned upstream and Cathedral agree on `EFI_MEMORY_HOT_PLUGGABLE = 1 << 20`.
The official 2.11 PDF's search-index extract instead displayed
`0x0000000000010000` (bit 16, colliding with `MORE_RELIABLE`). Full PDF retrieval
was blocked by HTTP 403. This review does not resolve whether the extract is a
transcription/indexing error or published erratum. Retain the current upstream
value, label its primary-spec verification pending, and require full primary
text/erratum review before claiming all exact-value vectors spec-verified.
This is source verification work, not an Omega language/design blocker.

## Review evidence and omissions

All current top-level declarations, boundary operations, policy states, and data
fields in the two requested files are mapped above. The root-level package build
machine is outside the requested reconciliation slice. The remaining upstream
crate is outside UEFI-000; UEFI-001 through UEFI-009 own its complete symbol,
source and test inventory. No upstream operation body or unit test has been
translated by this reconciliation. The existing comments about historical QEMU
success are historical claims, not refreshed evidence for this revision.
