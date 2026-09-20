# UEFI raw-contract port

## Scope and status

- Queue task: **UEFI-000**, reconciliation of existing declarations in
  `uefi.omg` and `boot_services.omg` before raw-contract extension.
- Current stage: **inventoried**. This does not claim the complete `uefi-raw`
  crate is inventoried or transcribed.
- Reviewed: 2026-09-20, Cathedral base
  `c70c54418cff04731c02fbec48d095e7122dec69`.
- Omega reviewed: `eaa7993a23623cd8fabf45350340479c5c9c7879`.
- Source review: all declarations and fields of the existing two-file subset,
  their pinned upstream counterparts and the specification references in
  [RECONCILIATION.md](RECONCILIATION.md).
- Production inclusion: existing contracts are reached by
  [`source/boot/uefi/build.omg`](../../boot/uefi/build.omg). This inventory adds
  no source declarations or build reach.
- Remaining work: UEFI-001–009 cover the raw crate, layouts, values, operations,
  omitted files, test vectors and compiler verification; UEFI-010 owns producer
  and consumer fixtures. Existing policy/source migration remains separate.

## Upstream pin and licensing

- Project: [rust-osdev/uefi-rs](https://github.com/rust-osdev/uefi-rs).
- Exact commit: `c0facddf9ba42b74906a37fca2869e6cdbc8da6a`.
- Crate: `uefi-raw` 0.16.0.
- Optional checkout: repository-root `reference_code/rust-osdev/uefi-rs/`.
- Verified license: `MIT OR Apache-2.0` in root `Cargo.toml`, inherited by
  `uefi-raw/Cargo.toml`; scoped texts in `uefi-raw/LICENSE-MIT` and
  `uefi-raw/LICENSE-APACHE`, with matching source SPDX headers.
- Preserved expression for derivative translations: `MIT OR Apache-2.0`.
  Existing Cathedral definitions were authored as primary-spec facts and local
  adapters; reconciliation does not retroactively relabel all local policy as
  upstream-derived code.
- Retained texts: [MIT](../../../licenses/rust-osdev/uefi-rs/uefi-raw/LICENSE-MIT),
  [Apache-2.0](../../../licenses/rust-osdev/uefi-rs/uefi-raw/LICENSE-APACHE).
- Root record: [THIRD_PARTY_NOTICES.md](../../../THIRD_PARTY_NOTICES.md).
- Modified files for this slice: new `PORT.md` and `RECONCILIATION.md` only.
  No upstream Rust vendor files or translation bodies are included by UEFI-000.
- Pin audit: exact checkout HEAD and manifest/license files reviewed on
  2026-09-20. Future pin changes follow the separate
  [license audit procedure](../../../licenses/rust-osdev/README.md).

## Source and public-symbol map

The exhaustive declaration and field map is
[RECONCILIATION.md](RECONCILIATION.md#every-existing-top-level-declaration).
The claimed slice is the **existing Cathedral subset**, not the entire contents
of each Rust file mentioned below. Unrepresented upstream declarations remain
outside this slice and must be inventoried by the later whole-crate slices.

| Upstream path, relative to `uefi-raw/src` | Cathedral file / scope | Origin category | Disposition |
| --- | --- | --- | --- |
| `status.rs::Status`, three associated constants | `uefi.omg::EfiStatus`, three `EFI_*` statuses | primary-spec facts, upstream cross-check | Existing x64 wrapper and values inventoried. |
| `lib.rs::Handle` | `uefi.omg::EfiHandle` | primary-spec fact / local wrapper | Inert bits, not authority. |
| `protocol/console.rs::SimpleTextOutputProtocol` first two fields, `output_string` signature | `uefi.omg::TextOutputProtocol`, trait and leaf | raw prefix plus original adapter | Prefix and fixed-string policy inventoried; suffix omitted from existing code. |
| `table/header.rs::Header` | `uefi.omg::EfiTableHeader` | primary-spec facts | Complete field map; revision wrapper removed. |
| `table/system.rs::SystemTable`, `SIGNATURE` | `uefi.omg::EfiSystemTable`, signature | primary-spec facts / checked-view deviation | Complete field sequence; semantic references differ from raw pointers. |
| `table/revision.rs::Revision::EFI_2_00` | `EFI_MIN_SUPPORTED_REVISION` | fact selected by local policy | Minimum version is local admission policy. |
| `table/boot.rs::MemoryType`, `MemoryAttribute`, `MemoryDescriptor`, two function signatures, BootServices prefix | `boot_services.omg` raw declarations and boundaries | primary-spec facts / original adapters | Every existing tag, field and operation mapped; suffix and missing constants recorded. |
| No raw upstream equivalent | calling policy, `UefiApplication`, size thresholds, reserved mask, `MapKey`, `FinalMemoryMap` | original policy / adapter | Explicitly not generic upstream translation. |

The [header inventory](../../../tools/ports/fixtures/uefi-header.inventory.json)
checks the complete pinned Header file and six mapped declaration/field anchors.
Run `python3 tools/ports/inventory.py check tools/ports/fixtures/uefi-header.inventory.json
--checkout reference_code/rust-osdev/uefi-rs --require-transcribed` from the root.
It passes; this reconciliation's broader coverage claim remains its explicit
two-file declaration map.
Do not interpret it as a whole-crate inventory success.

## Primary specifications and representation vectors

The companion map cites UEFI §§2.3, 4.1–4.4, 7.2.1, 7.2.3, 7.4.6, 12.4 and
Appendix D. Its geometry tables are **derived expectations**, not measured output:
x86-64, little endian, 64-bit pointers/UINTN, UEFI x64 calling convention.

Expected shapes are header 24/8, full System Table 120/8, current text-output
prefix 16/8 versus complete 80/8, descriptor 40/8, and Boot Services prefix
240/8 versus complete 376/8 (size/alignment in bytes). Complete field-offset
and missing-suffix tables appear in the companion map.

No machine-readable vectors are claimed by UEFI-000. Upstream generation and
Omega comparison: **not run**. Other architectures are unsupported by the
current local `u64` specialization. Primary-text verification of the
hot-pluggable bit remains pending because the official PDF search extract
conflicted with the pinned value and full retrieval returned HTTP 403;
[details](RECONCILIATION.md#outstanding-primary-source-discrepancy).

## Translated tests and fixtures

None in UEFI-000. This is a reconciliation task and contains no new behavior.
The pinned `table/boot.rs` compile-time descriptor size/alignment/offset asserts
are identified for UEFI-001 vectors; `table/revision.rs::tests::test_revision`
and status tests belong to their later scalar translation slices. Historical
boot-success comments are not refreshed test evidence.

## Omega blockers and boundary seams

| Blocker / obligation | Evidence and scope | Work that continues |
| --- | --- | --- |
| `omega:named-calling-policy` | Current [calling-plan spec](../../../../Omega/wiki/spec/build/calling_plans.md) requires `Calling<C, Policy>`; [implementation README](../../../../Omega/omega-rust/omega/representations/calling-conventions/README.md) records only `Calling<C>` source vocabulary. No diagnostic run in this documentation slice. | Inert tables, constants, mappings and host vectors. |
| Pointer backing, access, retention and boot lifetime | [Foreign storage](../../../../Omega/wiki/spec/build/foreign_storage.md) requires authority separately from `addr`/`Ptr<T>`. Existing shared console view cannot by itself authorize foreign state mutation. | Document raw signatures; implement checked adapters against established contracts later. This is not itself an unresolved language design. |
| Legacy `uefi_x64 machine` and commented use-site `Uefi` layout | Current requirement-owned calling-policy and explicit layout rules differ from legacy source. No current compilation claim. | Source migration and authored fixed layouts are engineering work. |

`PORT-BLOCKED[omega:named-calling-policy]: select an exact named CallingPolicy
conformance on each boundary requirement.` No source seam is edited by this
inventory; a later boundary migration must place the marker at its unresolved
source declaration. Programmable overlays/runtime-stride source forms are
future raw-corpus blockers only where actually needed; they do not block fixed
record transcription here.

For upstream `unsafe` function pointers, the required invariants are valid
service/protocol backing, selected ABI, live firmware phase, nonaliasing writable
outputs, exact buffer bounds/alignment and nonretention/retention contracts.
Table slots are inert data until an admitted boundary establishes those facts.

## Deliberate deviations

See [full reconciliation](RECONCILIATION.md#collisions-ownership-and-migration):
64-bit scalar wrappers, raw pointers represented as references, fixed 19-unit
text and 64-KiB map carriers, incomplete table/protocol prefixes, implicit
memory-descriptor padding, local revision/size/reserved-bit admission policy,
and local semantic-entry calling policy. `BootServices` already names a
boundary trait and conflicts with the upstream raw table name. None of these
is silently redefined by this slice.

## Cathedral integration and authority

The [contracts charter](../CHARTER.md) owns raw ABI. The
[entry handoff](../../../wiki/spec/boot/uefi_entry_handoff.md) and
[boot-services transition](../../../wiki/spec/boot/uefi_boot_services.md) own
Cathedral policy. A raw address, memory descriptor or successful parsing result
cannot establish firmware authority or RAM custody. Runtime Services have a
separate lifetime after Boot Services end.

Existing build reach is `source/boot/uefi` → `source/contracts/uefi`; the
calling-policy canary also consumes the contract. No new source enters these
roots in this slice. No authority adapter, grants, hardware execution or
producer/consumer integration is newly implemented or verified.

## Verification commands and results

Commands run from the Cathedral repository root on 2026-09-20:

| Check | Command / action | Result and scope |
| --- | --- | --- |
| Upstream pin | `git -C reference_code/rust-osdev/uefi-rs rev-parse HEAD` | Exact pinned `c0facddf9ba42b74906a37fca2869e6cdbc8da6a`. |
| Omega pin | `git -C ../Omega rev-parse HEAD` | `eaa7993a23623cd8fabf45350340479c5c9c7879`. |
| Source / license review | Read both local files and the upstream paths listed above; inspect root/crate manifests and scoped license files | Declaration and provenance review only. |
| Layout/value comparison | Not run | Derived expectations only. |
| Omega typecheck / translated tests | Not run | Documentation-only inventory; no new translation claimed. |
| Affected canary | `tools/uefi-calling-policy-canary/run.sh`, inspected but not executed | Still invokes historical `omega-cli`; harness migration is engineering work. |
| Hardware / integration | Not run | No boot or runtime claim. |
