# TCG raw protocols (UEFI-008)

Status: **typechecked** for the raw declarations and layout-policy machines
with the identified local Omega binary below. This is not a compiled ABI,
firmware, TPM, measured-boot, or attestation result. The raw protocol corpus is
not imported by Cathedral's production boot root.

## Upstream and source mapping

Derived from [rust-osdev/uefi-rs](https://github.com/rust-osdev/uefi-rs) at
`c0facddf9ba42b74906a37fca2869e6cdbc8da6a`, under **MIT OR Apache-2.0**.
Copyright (c) The uefi-rs contributors. See the
[retained notices](../../../../THIRD_PARTY_NOTICES.md) and
[license texts](../../../../licenses/rust-osdev/uefi-rs/uefi-raw/).
Modified for Cathedral: explicit Omega records, inert address slots, flattened
constant names, and separate authored layout policies replace Rust repr,
function-pointer, and macro surfaces. No upstream code is linked into Cathedral.

| Complete pinned file | Destination |
| --- | --- |
| `uefi-raw/src/protocol/tcg.rs` | `tcg` module; Rust-only module/reexport topology deliberately flattened. |
| `uefi-raw/src/protocol/tcg/enums.rs` | `tcg.omg`: AlgorithmId, EventType, all 15 algorithm and 36 event constants. |
| `uefi-raw/src/protocol/tcg/v1.rs` | `tcg.omg`: TcgVersion, TcgBootServiceCapability, all five TcgProtocol slots and GUID. |
| `uefi-raw/src/protocol/tcg/v2.rs` | `tcg.omg`: version/capability/header/protocol data, all seven slots, nine bitmap/flag constants and GUID. |

[tcg-inventory.json](tcg-inventory.json) checks all four files and 118 lexical
anchors: 113 mapped, five deliberately omitted module/reexport/alias spellings,
none silently missing. `Tcg2EventLogFormat` aliases `Tcg2EventLogBitmap` upstream;
use that same carrier directly, without inventing a second nominal Omega type.
Rust Debug/order/bitflags convenience traits are not foreign ABI operations;
the port retains their open numeric carriers and complete named constants.
The pinned files contain no unit-test bodies or pure algorithms to translate.

## Specification and representation

Primary references consulted:

- [TCG EFI Protocol Specification TPM 1.1/1.2, version 1.22 revision 5](https://trustedcomputinggroup.org/wp-content/uploads/TCG_EFI_Protocol_1_22_Final-v05.pdf), protocol interface and capability definitions.
- [TCG EFI Protocol Specification Family 2.0, revision 0.13](https://trustedcomputinggroup.org/wp-content/uploads/EFI-Protocol-Specification-rev13-160330final.pdf), §§6.2, 6.4.3, 6.6.3–6.6.4: interface, capability and event definitions.
- [TCG PC Client Platform Firmware Profile](https://trustedcomputinggroup.org/resource/pc-client-specific-platform-firmware-profile-specification/), event-log vocabulary; the pinned upstream list is preserved, not claimed to exhaust later registries.

Omega contracts read before implementation: `wiki/spec/layouts/plans.md`,
`wiki/spec/build/{foreign_storage,boundary_shapes,calling_plans}.md` and
`source/library/core/layout.omg` in the sibling repository. Exact source
signatures remain in comments beside every address slot. Those comments preserve
native input/output directions and pointee types without declaring an unchecked
Omega call surface.

[tcg_layouts.omg](tcg_layouts.omg) contains 12 named `Layout` conformances and
static `plan(schema)` machines implementing the corresponding fixed x64 plans.
These are available for explicit selection; the raw data's default home layout
is not asserted to equal its UEFI representation. In particular, the packed
Tcg2EventHeader is 14 bytes/alignment 1 with fields at 0, 4, 6, and 10; ordinary
C padding or an Omega home layout would not be a valid substitute.

[tcg.vectors.json](tcg.vectors.json) retains 125 measurements: all sizes,
alignments, field offsets, constants, and both GUIDs in little-endian UEFI byte
order. The Rust helper cross-compiles **actual pinned uefi-raw types and
constants** for `x86_64-unknown-uefi`; it reads retained LLVM constants without
executing firmware. It checks the authored Omega field order, literal values,
GUID fields and layout-plan geometry against that independent source evidence.
It does not measure Omega's selected or emitted layout.

Upstream's opaque event pointers do not define a flexible-array log parser.
The port preserves that scope. TPM log lengths, returned addresses and success
statuses remain inert facts until a checked caller supplies backing, lifetime,
bounds and independent trust policy. No digest is computed and no PCR is read,
extended or reset. A firmware capability record is not attestation authority.

## Verification and limitations

Run from Cathedral's repository root:

```sh
python3 tools/ports/inventory.py check source/contracts/uefi/raw/tcg-inventory.json --checkout reference_code/rust-osdev/uefi-rs --require-transcribed
python3 tools/ports/vectors.py source/contracts/uefi/raw/tcg.vectors.json
python3 tools/ports/uefi-tcg/check.py
../Omega/target/release/omega --check source/contracts/uefi/raw/tcg.omg
../Omega/target/release/omega --check source/contracts/uefi/raw/tcg_layouts.omg
```

On Windows use `python` and the equivalent `omega.exe` path. The host checker
requires Python 3.9+, Cargo and Rust's `x86_64-unknown-uefi` standard library,
the pinned checkout, and the checked-in Cargo lock. All commands fail when their
required inputs are missing. Rust dependencies are host verification inputs,
not target OS dependencies or a committed vendor tree.

Observed on macOS ARM64, 2026-09-20: inventory and vector validation pass;
123 cross-target numeric measurements, two GUIDs and 39 source fields agree.
Raw source and named layout machines both pass Omega `--check`. The existing
release binary SHA-256 is
`a87e533bc7c068419ab2ff05c213abbb4a874e3fe8dbba35e5636004e6f5bfe6`;
its build revision is not independently established, so this does not claim a
fresh build of the current Omega checkout. Checks can be rerun with a supplied
new compiler. No source in the existing boot-facing contracts was changed.

Actual remaining compiler seam: named calling-policy selection and inert-address
materialization for native protocol calls, as documented in Omega's
`omega-rust/omega/representations/calling-conventions/README.md` and authored
`source/library/std/targets/uefi_x86_64/tables.omg`. Source markers name that
seam. Native call admission, backing and lifetime adapters, selected Omega ABI
observation, hardware execution, and Cathedral integration are **not run**.
They are distinct from this completed raw transcription and source typecheck.
