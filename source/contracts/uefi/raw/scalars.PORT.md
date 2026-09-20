# UEFI scalar foundations (UEFI-001)

## Scope and status

**Current stage: tested by Omega semantic evaluation (2026-09-20); native
execution and foreign-layout comparison are not established.** This slice covers the scalar declarations,
constants, Boolean semantics, status predicates, revision packing, time range
validation/equality, capsules, memory descriptors/types/attributes, and common
boot enums from the pinned `uefi-raw` sources. It does not claim all of `lib.rs`
or `table/boot.rs`: their module scaffolding and service tables are explicitly
excluded in the exhaustive inventory. Existing boot contracts are untouched.

The five raw modules are `scalars.omg`, `status.omg`, `revision.omg`, `time.omg`,
and `capsule.omg`. Pure convenience behavior lives in
`../../../libraries/uefi/scalar_helpers.omg`; executable fixtures live under
`../../../../tools/ports/uefi-scalars/`. No production boot root imports this
package. Source presence is not a foreign-layout or firmware-call guarantee.

## Upstream pin and licensing

- Repository: <https://github.com/rust-osdev/uefi-rs>.
- Pin: `c0facddf9ba42b74906a37fca2869e6cdbc8da6a`.
- Checkout: repository-root `reference_code/rust-osdev/uefi-rs`.
- Files: `uefi-raw/src/{lib,status,time,capsule}.rs`,
  `uefi-raw/src/table/{revision,boot}.rs`.
- Preserved license: `MIT OR Apache-2.0`; each translated Omega file identifies
  its modified origin. License texts/notices are indexed in
  [THIRD_PARTY_NOTICES.md](../../../../THIRD_PARTY_NOTICES.md).
- GUID representation is independently authored from the specification; no
  `uguid` source or its macro implementation was copied. The pinned dependency
  is `uguid` 2.2.1, used by the independent upstream Rust layout probe.

## Source and public-symbol map

[scalar-inventory.json](scalar-inventory.json) maps all 283 lexical review
anchors in the six source files: 211 translated and 72 deliberately omitted.
Every omission has a reason. Service table fields, callback signatures, and
open-protocol records belong to the separate UEFI-002 inventory. The mapping
covers public raw values, fields, helper operations and upstream tests, not
merely type names. It is a source audit, not a semantic Rust parser.

Rust `Display`/`Debug` formatting, `guid!` string parsing, trait glue, crate
reexports, and unrelated protocol/net/firmware-storage modules are outside this
raw ABI and scalar-algorithm slice. Boolean hashing preserves the exact byte
fed to a hasher, with tests for equal values yielding equal bytes; a general
hashing framework is not recreated. All integral unknown values remain admitted
by the raw carriers, rather than incorrectly becoming exhaustive Omega sums.

## Primary specifications and representation vectors

[scalars.vectors.json](scalars.vectors.json) contains 233 expected measurements
in `cathedral-port-vectors-v1`, including every authored scalar constant,
size/alignment/field offset, and endian-sensitive GUID/status/revision/time
examples. Target: **UEFI x86-64, 64-bit pointers, little-endian, pinned upstream
raw C representation**. Other target profiles are not claimed.

Governing sources consulted:

- [UEFI 2.11 §2.3.1](https://uefi.org/specs/UEFI/2.11/02_Overview.html#data-types):
  scalar widths, natural C alignment and pointer width.
- [§4.2](https://uefi.org/specs/UEFI/2.11/04_EFI_System_Table.html#efi-table-header):
  revision encoding; `2.3.1 = 0x0002001f`, `2.10 = 0x00020064`.
- [§7.1–7.3](https://uefi.org/specs/UEFI/2.11/07_Services_Boot_Services.html):
  events, TPL, allocation/memory types, descriptor attributes and GUID structure.
- [§8.3.1 and §8.5.3](https://uefi.org/specs/UEFI/2.11/08_Services_Runtime_Services.html):
  time/daylight and capsule records/flags.
- [Appendix D](https://uefi.org/specs/UEFI/2.11/Apx_D_Status_Codes.html):
  standard statuses and high-bit interpretation.
- [UEFI 2.10 Appendix A](https://uefi.org/specs/UEFI/2.10/Apx_A_GUID_and_Time_Formats.html):
  explicit GUID byte ordering, used as a cross-edition format check.

Some direct UEFI 2.11 page fetches returned HTTP 403; indexed official sections
were available for the scalar, revision, time, capsule, and status review.
Primary review of every boot-memory numeric value remains incomplete; pinned
upstream values are retained with that limitation, not called spec-certified.

An independent compiled Rust probe in `tools/ports/uefi-tables/check.py` targets
`x86_64-unknown-uefi` and confirms Guid `(size=16, align=4)`, CapsuleHeader
`(28,4)`, Time `(16,4)`, and MemoryDescriptor `(40,8)`. Those are upstream
measurements, not Omega observations. Remaining scalar geometry is expected
from the reviewed field types/C representation. Full Omega comparison is not
run; default Omega layout is not asserted to equal these foreign layouts.

## Deliberate deviations and unresolved specification questions

- Named wrappers with `raw` fields replace Rust primitive aliases, transparent
  newtypes and pointers. Their nominal source identity differs. Handle, Event,
  PhysicalAddress and VirtualAddress are inert `u64` values in this profile;
  constructing them establishes no authority or typed pointer. Char8/Char16
  preserve numeric carriers without proving a string encoding.
- Constants use explicit prefixes (`STATUS_`, `MEMORY_ATTRIBUTE_`, etc.) in
  place of Rust associated constants. All are mapped individually.
- Upstream accepts any nonzero Boolean byte as true. UEFI §2.3.1 defines only
  0/1 and leaves other values undefined. Helpers preserve upstream truthiness
  and raw byte round-trip, but do not claim noncanonical values are valid input
  to firmware. Do not replace the raw carrier with Omega `bool`.
- UEFI §2.3.1 says GUID values are normally aligned on an eight-byte boundary;
  the upstream `uguid` C representation has four-byte alignment. Vectors label
  the latter explicitly. Pointer/storage alignment at the boundary requires
  separate review; blindly changing Guid alignment would change enclosing
  CapsuleHeader size from the upstream 28 bytes. No boundary layout is selected
  here, and this unresolved specification interpretation must be settled before
  integration.
- `MemoryAttribute::HOT_PLUGGABLE` is `0x100000` in the pin and existing
  Cathedral facts. An indexed 2.11 specification extract encountered during
  reconciliation reports `0x10000`, conflicting with MORE_RELIABLE. Preserve
  the pin and track primary-source resolution; this is not an Omega blocker.
- Revision minor values are ordinary integers split into decimal digits for
  display, despite the upstream introductory “BCD” wording. No nibble-packed
  BCD conversion is introduced.
- `time_is_valid` preserves upstream's numeric-range test. It intentionally
  accepts February 31 and reserved daylight bits; it is not full calendar or
  SetTime validation. `time_equal` ignores pad1/pad2, but compares daylight.
- `memory_type_custom` replaces upstream's assertion with an explicit
  `value >= 0x80000000` precondition. Unknown raw memory types still fit the
  representation. The historic OVMF custom-type compatibility caveat remains
  in `scalars.omg`; no allocation policy is chosen.
- Capsule union alternatives share one physical-address representation selected
  by `length`, as upstream already does. This creates no union access or
  pointer-dereference permission. `header_size` may exceed the fixed header.

## Translated tests and fixtures

`tools/ports/uefi-scalars/main.omg` translates upstream Boolean conversions,
comparison/order/hash equivalence and `test_revision`, and adds status high-bit
boundaries, revision 0/maximum inputs, all time numeric range edges, padding
independence, the deliberately incomplete calendar/daylight predicate, and
custom memory-type endpoints. Hash testing compares canonical input bytes; the
upstream private rolling TestHasher is unnecessary for that invariant.

Upstream time formatting expectations are retained here as **unexecuted,
out-of-slice Display fixtures**, not passing Omega tests:

| Inputs (2023-05-18, 11:29:57, 123456789 ns) | Expected upstream Display |
| --- | --- |
| timezone 2047 | `2023-05-18 11:29:57.123456789 (local)` |
| timezone 120 | `2023-05-18 11:29:57.123456789UTC+2` |
| timezone 150 | `2023-05-18 11:29:57.123456789UTC+2.5` |
| timezone -330 | `2023-05-18 11:29:57.123456789UTC-5.5` |

These are upstream formatting behavior, not a correction of UEFI's
`LocalTime = UTC - TimeZone` convention. A later formatting implementation must
review that distinction. Revision display examples are `1.02`, `1.10`, `2.0`,
`2.3`, `2.3.1`, and `2.10`; only packing/extraction/order are implemented here.

## Omega blockers, integration and remaining work

There is **no demonstrated Omega language blocker for these semantic scalar
records or pure helpers**: source checking succeeds. Explicit type/machine
imports avoid unresolved-selection diagnostics from broad imports in the
available compiler binary. Missing foreign-layout policies/observations and
native test integration are outstanding engineering, not fabricated language blockers.
Actual firmware calls, pointer establishment, owned-memory admission, callback
lifetimes, and capsule execution remain separately reviewed boundary concerns.

Production integration: none. Existing `uefi.omg` and `boot_services.omg`
remain separate, with their collision/migration audit in UEFI-000. No existing
boot build root is changed. Existing runtime canaries have not been rerun for
this unreferenced new package; the focused source check covers only the new
scalar package and tests. No firmware, hardware, or QEMU execution is claimed.

## Verification commands and results

Run from repository root. Omega source checkout reviewed:
`eaa7993a23623cd8fabf45350340479c5c9c7879`. The existing release binary was **not
rebuilt**, so that source revision is not claimed as its build identity.
Binary SHA-256: `a87e533bc7c068419ab2ff05c213abbb4a874e3fe8dbba35e5636004e6f5bfe6`.

```sh
python3 tools/ports/inventory.py check source/contracts/uefi/raw/scalar-inventory.json --checkout reference_code/rust-osdev/uefi-rs --require-transcribed
python3 tools/ports/vectors.py source/contracts/uefi/raw/scalars.vectors.json
python3 tools/ports/uefi-scalars/check.py
```

Inventory: passed (six files, 211 mapped, 72 deliberate exclusions, no pending
or blocked anchors). Vector syntax: passed (233 expectations); Omega layout
comparison **not run**. Semantic check: passed using the declared raw/library
package dependencies; this establishes no foreign ABI measurement.

Omega fixture execution: **passed in semantic evaluation**, with
`TEST_RESULT = test_result()` and a checked `requires value == 0` call. The
negative control changed that call to `TEST_RESULT + 1` and rejected exactly at
`0 + 1 == 0`, confirming the tests execute and the success gate is active.
The runner creates its negative fixture in a temporary directory without
changing the port. All positive/negative checks used the binary hash above.

Native `omega run --both` was attempted but did not execute these tests. Early
attempts encountered missing package acceptance and missing/incorrect program
entry setup. A hosted test entry was then prepared, but concurrent changes to
the raw package invalidated its source-hash-bound review. That temporary hosted
setup was removed; the committed fixture requires no std/Console dependency or
native entry. No native compiler defect or native passing result is claimed.
Semantic-evaluator success does not prove native machine realization.
