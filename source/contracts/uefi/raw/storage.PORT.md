# UEFI storage representations (UEFI-004)

Status: **typechecked** source declarations and layout-policy definitions.
The raw slice is completely transcribed: 39 records, 254 fields, 26 scalar
carriers, 297 constants, and nine source files. The host audit compares 657
actual upstream UEFI x64 measurements with the integer vectors, and checks all
23 GUIDs separately. No Omega ABI measurement, device execution, or Cathedral
integration is claimed.

## Provenance and documents reviewed

- Upstream: https://github.com/rust-osdev/uefi-rs
- Exact pin: `c0facddf9ba42b74906a37fca2869e6cdbc8da6a`.
- Preserved license: `MIT OR Apache-2.0`; Copyright (c) The uefi-rs contributors.
  [Third-party notices](../../../../THIRD_PARTY_NOTICES.md) retain the exact
  [license texts](../../../../licenses/rust-osdev/uefi-rs/uefi-raw/).
- [UEFI 2.11 section 13, Media Access](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html):
  simple file system/file, disk, block, SCSI, ATA, and NVMe protocol shapes;
  sections 13.5, 13.7–13.10 specifically cross-check file/disk revisions and
  the version-dependent BlockIoMedia suffix.
- [UEFI 2.11 section 23, Firmware Update and Reporting](https://uefi.org/specs/UEFI/2.11/23_Firmware_Update_and_Reporting.html):
  FMP records, image-check flags, dependency opcodes, and packed capsule bodies.
  Section 23.3.2 explicitly requires packed capsule structures; no natural
  alignment is inferred for incoming capsule buffers.
- [PI 1.9, volume 3 code definitions](https://uefi.org/specs/PI/1.9/V3_Code_Definitions.html):
  firmware volume headers/block maps and FirmwareVolume2/FirmwareVolumeBlock2
  protocol shapes. These PI interfaces are distinguished from UEFI services.
- Omega `wiki/spec/{layouts/plans,build/foreign_storage,build/boundary_shapes}.md`
  and `omega-rust/psi/semantics/build-time-evaluation/layouts.md`: semantic
  fields, validated geometry, inert addresses, and unsupported variable placement.
- Cathedral `wiki/spec/boot/uefi_boot_services.md`: raw descriptors and successful
  firmware operations do not establish Cathedral resource authority.

## Complete source mapping

Paths in this table are relative to `uefi-raw/src/` at the pin. All named
records, fields, enum/flag values, associated constants, and global aliases map
to `storage.omg`. All 39 record placement policies are in `storage_layouts.omg`.
[storage-inventory.json](storage-inventory.json) retains exact source hashes and
613 lexical anchors, all translated, with no omitted or pending source anchors.
The explicit field audit includes Rust raw identifier `r#type`.

| Source | Raw content retained |
|---|---|
| `protocol/file_system.rs` | SimpleFileSystemProtocol; FileProtocolV1/V2; FileIoToken; FileInfo, FileSystemInfo, FileSystemVolumeLabel; revisions, modes, attributes and information GUIDs. |
| `protocol/block.rs` | Lba; complete revision-3 BlockIoMedia; BlockIo/BlockIo2 protocols and token; GUIDs and revision values. |
| `protocol/disk.rs` | DiskIo/DiskIo2 protocols and token; DiskInfoProtocol and all interface GUIDs. |
| `protocol/ata.rs` | Pass-through attributes/mode/protocol/packet, command/status blocks, command protocol and length encodings, GUID. |
| `protocol/scsi.rs` | SCSI IO types/status/direction, complete request packet, SCSI IO and extended pass-through mode/protocol, target byte count and GUIDs. |
| `protocol/nvme.rs` | Command-word validity, controller attributes/mode, command/completion records, queue type, pass-through packet/protocol and GUID. |
| `protocol/firmware_volume.rs` | FvAttributes, file attributes, write policy, file/section kinds and range endpoints; FvWriteFileData; FirmwareVolume2 and FirmwareVolumeBlock2 protocols, GUIDs and terminator. |
| `protocol/firmware_management.rs` | Capsule flags and packed headers; FMP dependency opcodes/prefix; image/package attributes, compatibility flags and updatability results; descriptor, management protocol and GUID. |
| `firmware_storage.rs` | FirmwareVolumeHeader, block map record, full firmware-volume attribute/alignment constants, signature bytes. |

`source-shapes.json` under the host fixture directory records every field's exact
upstream type and corresponding Omega type. Its contents are checked against
the pinned Rust declarations before comparing the Omega declarations. Function
signatures and pointer depths are also retained beside every inert address field
in source. This metadata is source-review support, not executable authority.

## Deliberate translation choices

- Pointer and function-pointer members are inert `addr` data. Handles, status,
  Boolean, time, and UTF-16 elements use the shared scalar modules. Pointer
  constness, output parameters, callback signatures, and variadic tails survive
  in signature comments; none becomes a dereference or call operation.
- UINTN is u64 for this explicitly UEFI x86-64 profile. Lba is a named u64
  carrier because Omega does not provide the Rust type-alias spelling used
  upstream. Scalar flag and enum wrappers preserve every underlying bit pattern;
  they are not closed sums and do not reject vendor/unknown values.
- Constants have type-prefixed flat names. FvFiletype ranges become inclusive
  MIN/MAX endpoint constants. The anonymous `ImageCompatibilities` all-bits flag
  is named `IMAGE_COMPATIBILITIES_ALL_BITS`, retaining vendor bits rather than
  narrowing them to the one known standard flag.
- `FvWriteFileData.r#type` becomes `file_type`; field order and offset remain
  unchanged. FileProtocolV2 retains its nested V1 prefix, so V1 operations
  remain covered without duplicate slots.
- BlockIoMedia includes the upstream revision-3 suffix. Reading it requires a
  matching revision/boundary contract; its declaration does not make those
  suffix bytes present in older firmware. The three revisions remain distinct.
- Rust derives and generated bitflags/newtype-enum ergonomics are intentionally
  not cloned. The input files contain no independently authored pure algorithms
  or standalone `#[test]` cases; this slice preserves their records, values and
  ABI facts rather than inventing safe Rust-style device operations.
- The six zero-length fields retain the exact upstream fixed-prefix markers:
  file names/volume labels, capsule item-offset list, FMP dependencies, and
  firmware block map. They are not actual zero-length limits on valid firmware
  payloads. Prefix size, tail offset, dynamic length, and terminator/bounds
  validation are distinct. No variable-tail parser is claimed.
- Two capsule headers preserve packed alignment-one expectations. All other
  records use their upstream C alignment. The policies have public named
  `Layout` conformances with static `plan(schema) satisfies Layout::plan`
  definitions; source checking does not establish selection, normalization,
  materialization, or native ABI agreement of those policies.
- Progress callbacks, asynchronous tokens, device reset/write, firmware updates,
  and the variadic `erase_blocks` slot have no callable boundary here. Invocation
  must later bind explicit provider, lifetime, buffer, and firmware/device
  authority. A firmware-management GUID cannot authorize flash writes.

## Verification

From the Cathedral root:

```sh
python3 tools/ports/inventory.py check source/contracts/uefi/raw/storage-inventory.json --checkout reference_code/rust-osdev/uefi-rs --require-transcribed
python3 tools/ports/vectors.py source/contracts/uefi/raw/storage.vectors.json
python3 tools/ports/uefi-storage/check.py
python3 tools/ports/uefi-storage/semantic-check.py
../Omega/target/release/omega --check source/contracts/uefi/raw/storage.omg
../Omega/target/release/omega --check source/contracts/uefi/raw/storage_layouts.omg
```

The inventory and 681-vector schema pass. The host checker validates exact
source/field/signature coverage, scalar carriers, constants, all requested
record geometry, and 23 mixed-endian GUID encodings. It compiles the actual
pinned Rust types for **x86_64-unknown-uefi** and reads 657 compiler-evaluated
size/alignment/offset/constant values from a static LLVM artifact. Every integer
vector matches; no host ABI substitution is involved. The 24 byte vectors
represent 23 GUIDs and the `_FVH` signature. Cargo dependencies are locked and
build output is temporary. Missing optional source or target support fails.

Both raw source and layout definitions pass Omega `--check`. The separate
constant fixture exercises high-bit values, unknown carrier values, vendor
range endpoints, revision distinctions, signature byte order, and protocol
identity. Its executable evidence is recorded by `semantic-check.py`, which
requires an evaluated zero result and checks that changing the expected result
fails with the exact obligation. This is semantic evaluation, not native
execution or a layout observation. No source module is in a production boot
build root, and no affected device canary is claimed to have run firmware.

## Unresolved language/consumer seams

`PORT-BLOCKED[omega:variable-tail-layout]`: runtime tail/stride placement and
projection still lack the general supported source forms described as
unspecified in Omega's layout contract. Zero-array prefix records and exact
tail offsets can be retained now; a live variable-length typed view cannot be
invented from them. Ordinary explicit bounded byte parsers can still be written
in later parser tasks; their engineering is not classified as a language block.

Omega's existing authored UEFI layouts also document that `addr` is not a
supported schema materialization carrier and substitute u64. This slice retains
inert `addr` semantics and awaits an actual supported consumer/inspection route;
source typechecking does not supply the missing observation. No record here
requires more than the current 32 reflected members. Packed and zero-sized
placement normalization must be exercised before claiming foreign ABI support.

After obtaining actual Omega observations, compare them with `vectors.py
--observed`, recording compiler revision and artifact hash. Device operation,
firmware update policy, tail parsing, custody transfer, and Cathedral integration
remain separate later work; no engineering item is hidden behind the two source
representation limitations.
