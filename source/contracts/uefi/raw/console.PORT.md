# Console, image-loading and device-path raw slice

## Scope and status

UEFI-003. Reviewed 2026-09-20 at upstream
`c0facddf9ba42b74906a37fca2869e6cdbc8da6a`, Omega
`eaa7993a23623cd8fabf45350340479c5c9c7879`.

**Overall stage: inventoried; fixed declarations, 114 authored layout policies,
and pure helpers typechecked.** The 15 runtime-sized device-path tails are explicitly blocked
at their typed-view seams; their fixed prefixes are retained. The bounded pure-helper fixture executes in the Omega semantic evaluator;
no native Omega layout, firmware or integration success is claimed.
The seven-file source inventory has no pending/unclassified anchors.

The slice includes all of `protocol/console.rs` (including GOP/EDID),
`console/serial.rs`, `loaded_image.rs`, `device_path.rs`, generated
`device_path/device_path_gen.rs`, `media.rs` (including storage security), and
`shell_params.rs`. UEFI-004/005 should reuse the overlapping complete-file
transcriptions rather than create competing definitions.

## Upstream pin and licensing

[uefi-rs](https://github.com/rust-osdev/uefi-rs), `uefi-raw` 0.16.0, exact commit
`c0facddf9ba42b74906a37fca2869e6cdbc8da6a`. Optional checkout is repository-root
`reference_code/rust-osdev/uefi-rs`. Preserved **MIT OR Apache-2.0**; source and
probe headers identify modified translations. Retained scoped texts:
[MIT](../../../../licenses/rust-osdev/uefi-rs/uefi-raw/LICENSE-MIT),
[Apache-2.0](../../../../licenses/rust-osdev/uefi-rs/uefi-raw/LICENSE-APACHE).
See [third-party notices](../../../../THIRD_PARTY_NOTICES.md) and the
[separate pin audit procedure](../../../../licenses/rust-osdev/README.md).

## Source and public-symbol map

[console-inventory.json](console-inventory.json) records seven source hashes and
583 lexical anchors: 567 translated, 15 blocked runtime tails, one upstream
compile-time assertion replaced by vectors. Compact generated macro bodies have
several constants on one line; the supplementary
[schema](../../../../tools/ports/uefi-console/schema.json), generated source and
733 vectors cover their individual declarations/values. Inventory tooling is
lexical; it does not expand Rust derive macros.

| Upstream under `uefi-raw/src/protocol/` | Destination | Deliberate mapping |
| --- | --- | --- |
| `console.rs` | [console.omg](console.omg) | Every raw structure/flag/enum/GUID; callback alias is inert address. Graphics default maps to helper. |
| `console/serial.rs` | [serial.omg](serial.omg) | Both serial revisions, mode, control flags, parity/stop bits and GUIDs. |
| `loaded_image.rs` | [loaded_image.omg](loaded_image.omg) | Complete raw image record; optional unload pointer is nullable numeric slot. |
| `device_path.rs` | [device_path.omg](device_path.omg) | Header, all type/subtype tags, utility/text protocols; length maps to pure helper. |
| `device_path/device_path_gen.rs` | [device_path.omg](device_path.omg) | Every node's fixed fields and all enum/flag/GUID values; category-prefixed type names. |
| `media.rs` | [load_file.omg](load_file.omg) | LoadFile, LoadFile2 and StorageSecurityCommand raw slots/GUIDs. |
| `shell_params.rs` | [shell_params.omg](shell_params.omg) | Shell handle carrier and complete parameter record/GUID. |
| Pure operations | [console_helpers.omg](../../../libraries/uefi/console_helpers.omg) | Length decoding, zero graphics-mode constructor; additional local bounded-prefix check. |

No upstream raw callable operation is silently implemented: each function-pointer
field retains its original signature in a comment and its exact native offset
in vectors, but stores `addr`. Rust derives (`Debug`, comparisons, default for
trivial scalar records) are not a promise of generated Omega operations.

## Primary specifications and representation vectors

Primary contracts: UEFI 2.10 [§9, Loaded Image](https://uefi.org/specs/UEFI/2.10/09_Protocols_EFI_Loaded_Image.html),
[§10, Device Path](https://uefi.org/specs/UEFI/2.10/10_Protocols_Device_Path_Protocol.html),
[§12, Console/Serial/Graphics](https://uefi.org/specs/UEFI/2.10/12_Protocols_Console_Support.html?highlight=graphics+output),
[§13, Load File and storage security](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html),
and [UEFI Shell specification §2.3, parameters](https://uefi.org/sites/default/files/resources/UEFI_Shell_Spec_2_0.pdf).
Later generated node variants follow the exact upstream pin; the full corpus is
not claimed independently reviewed against every specification revision.

[console.vectors.json](console.vectors.json), `cathedral-port-vectors-v1`, contains
733 size/alignment/offset/constant/GUID-byte facts. Numeric facts originate from
pinned Rust, first measured on `aarch64-apple-darwin`, then **all** checked using
Rust compile-time assertions for `x86_64-unknown-uefi`; GUID bytes receive the
same cross-check. GUID fields retain mixed-width little-endian encoding.
This is stronger than host geometry alone and is still not an Omega observation.

Profile: 64-bit pointer/UINTN, little endian, UEFI x64 data geometry. Native
function calling is not tested. Source `addr` slots do not authorize calls.
Packed nodes require alignment 1 and explicit no-padding layouts, including
unaligned fields. Plain Omega `data` does not establish those layouts.

## Translated tests and fixtures

The source files contain a header size/alignment assertion and two explicit pure
methods, but no upstream `#[test]` cases. Host probes preserve the assertion and
all other geometry; two authored Rust tests exhaust every 16-bit length encoding
and check the upstream zero graphics constructor.

[fixtures.omg](../../../../tools/ports/uefi-console/fixtures.omg) retains the
translated Omega checks, with zero, minimum header, byte-carry and maximum length,
truncated available storage, and maximum available extent. They are evaluated by the `main.omg` semantic fixture described below.
Length decoding is total over 16 bits; values below four decode successfully but
fail the separate local bounds helper. There is no claim that bounds validate a
node's payload, path termination, text, device identity or memory custody.

## Omega blockers and boundary seams

`PORT-BLOCKED[omega:runtime-layout-strides]` marks 15 dynamic tails:
HardwareVendor, AcpiExpanded, AcpiAdr, MessagingUsbWwid, MessagingVendor,
MessagingIscsi, MessagingUri, MessagingDns, MessagingRestService,
MessagingNvmeOfNamespace, MediaVendor, MediaFilePath, MediaPiwgFirmwareFile,
MediaPiwgFirmwareVolume and BiosBootSpecBootSpecification.

Omega [layout plans](../../../../../Omega/wiki/spec/layouts/plans.md#placement-vocabulary)
leave runtime-stride source forms unspecified. Each current declaration is only
its fixed prefix; tail offsets remain measured vectors. The missing behavior is
a typed trailing view whose element count/stride derives from a checked runtime
extent. No zero-length field pretends to store the full tail. `MessagingDns`
additionally needs the upstream `IpAddress` union's actual overlay shape when
its tail is represented; an opaque byte carrier is not fabricated into that type.
Pure bounded byte parsing and fixed layouts can continue independently.

No new compiler diagnostic is claimed: this limit is documented in Omega's
normative layout contract. [console_layouts.omg](console_layouts.omg) now authors all 114 fixed policies,
with exact named public `Layout` conformances and field offsets. Policy source
and a selected `InputKeyX64Layout<InputKey>` carrier pass source checking.
Full consumer verification reaches actual compiler limits:

- `omega:plan-laid-public-fields`: selecting the public `InputKey` policy then
  reading/writing `scan_code` across the package boundary rejects with
  `selects private data InputKeyX64Layout<InputKey>::scan_code`; the original
  unwrapped public data/helper access succeeds. Reproducer:
  `tools/ports/uefi-console/layout_projection.omg`.
- `omega:placed-field-callable-arity`: selecting all fixed carriers rejects
  `PlacedField::read`, `take`, and `write` with “expected 0 callable generic
  parameter(s), got 1”. Reproducer: `layout_probe.omg` in that fixture package.

These are observed compiler failures, not missing local policies or a claimed
layout measurement. `layout_type_only.omg` selects a two-scalar policy and
passes without field projection. `layout_array_probe.omg` also passes for the
fixed-array DevicePathProtocol header; the aggregate failure is not evidence
that all array policies fail. Unselected/default Omega representations
remain distinct from explicit foreign geometry.

Upstream `unsafe` marks firmware invocation requirements: live protocol/table,
valid pointees, pointer/count agreement, exact writable outputs, alignment,
aliasing, nullability, callbacks/retention and lifecycle. Pure parsing cannot
establish any of these. No native invocation leaves are added by this slice.

## Deliberate deviations

- Pointer, function-pointer, nullable unload and opaque handle fields become
  inert `addr`. They preserve geometry/signature documentation without callable
  or reference semantics. Scalar Boolean/Char16/MemoryType fields use their raw
  widths so arbitrary incoming values remain representable.
- Rust nested device-path modules become category-prefixed names, avoiding
  `Vendor`, `Parity`, `StopBits` and other collisions in the single source module.
- Open numeric enums/flags become `[copy]` records with `raw` and prefixed named
  constants. Runtime tails are absent fields with precise blocked comments.
- Complete console/storage-security shapes are transcribed now because they
  share upstream source files with this slice. No display/storage driver follows.
- Generated source/probes are deterministic artifacts of the pinned source;
  future pin updates require explicit review, not automatic regeneration.

## Cathedral integration and authority

Owned by [contracts](../../CHARTER.md), isolated in the `cathedral-uefi-raw`
package. Pure helper behavior belongs to `source/libraries/uefi`. No production
boot dependency or existing consumer view is migrated here. The legacy
`TextOutputProtocol` prefix and `EfiSystemTable` remain separate as recorded by
[UEFI-000](../RECONCILIATION.md). Protocol GUIDs and table bytes identify shapes;
they never mint service, framebuffer, image-storage or handle authority.

## Verification commands and results

Run from repository root:

| Command | Observed scope/result |
| --- | --- |
| `python3 tools/ports/inventory.py check source/contracts/uefi/raw/console-inventory.json --checkout reference_code/rust-osdev/uefi-rs` | Seven files, 567 translated / 15 blocked / 1 omitted anchors, zero pending; source audit only. |
| `python3 tools/ports/uefi-console/measure.py` | 733 Rust facts measured and all checked for UEFI x64; no Omega run. |
| `python3 tools/ports/uefi-console/check_transcription.py` | 114 raw carriers and 178 constants agree with target-checked vectors under required C/packed rules. |
| `python3 tools/ports/vectors.py source/contracts/uefi/raw/console.vectors.json` | Vector format check; Omega comparison not run. |
| `cargo test --manifest-path tools/ports/uefi-console/Cargo.toml` | Two upstream behavior checks pass; no translated Omega execution. |
| `../Omega/target/release/omega --check source/contracts/uefi/raw/console.omg` (also serial, loaded_image, device_path, load_file, shell_params, console_layouts) | All seven roots pass. |
| `../Omega/target/release/omega --check source/libraries/uefi/console_helpers.omg` | Helper source passes. |
| `../Omega/target/release/omega --check tools/ports/uefi-console/main.omg` | Pure helper cases execute via compile-time `TEST_RESULT`; requires result zero; pass. |
| `../Omega/target/release/omega --check tools/ports/uefi-console/negative.omg` | Expected rejection: `cannot prove ... require_success ... 0 + 1 == 0`; verifies the real evaluated result is used. |
| `../Omega/target/release/omega --check tools/ports/uefi-console/layout_type_only.omg` | Selected InputKey policy carrier passes; no field access or layout observation. |
| `../Omega/target/release/omega --check tools/ports/uefi-console/layout_array_probe.omg` | Selected fixed-array DevicePathProtocol header passes; no projection/layout observation. |
| `../Omega/target/release/omega --check tools/ports/uefi-console/layout_projection.omg` | Fails synthesized public-field visibility as recorded above. |
| `../Omega/target/release/omega --check tools/ports/uefi-console/layout_probe.omg` | Fails PlacedField callable-generic arity as recorded above. |
| Native compilation | Attempted; package update/review succeeded, then concurrent source changes invalidated local lock. Semantic-evaluator tests avoid this tooling churn; it is not a language blocker. |
| Existing boot canaries/hardware | Not run for this isolated corpus; no production reach changed. |

Rust used: `1.94.0-nightly (f6a07efc8 2026-01-16)`. See the
[probe README](../../../../tools/ports/uefi-console/README.md) for regeneration,
verification scope and required optional checkout. A missing/mismatched checkout
fails instead of silently passing.
