# Whole raw-crate conformance audit (UEFI-009)

The pinned raw crate has **65 Rust source files**, all accounted for. Its raw
Omega modules pass one combined source check. Expected upstream geometry and
values are verified separately from Omega source semantics. This audit does
**not** claim emitted ABI equivalence, native service calls, hardware execution,
or Cathedral integration.

## Source closure

[conformance-index.json](conformance-index.json) records every file's upstream
hash, contributing slice inventories, disposition counts, and exact slice-manifest
hashes. The compact index avoids duplicating their detailed symbol mappings.
`tools/ports/uefi-conformance/inventory.py` reconstructs the union, validates each
slice against the exact checkout, and rejects unreviewed files, missing symbols,
changed targets, changed source or pending anchors. It does not parse full Rust
semantics or expand arbitrary macros. Whole-file hashes and the reviewed field
schemas supplement its conservative lexical index.

At pin `c0facddf9ba42b74906a37fca2869e6cdbc8da6a`:

- 3,486 anchors map to translated declarations, values, helpers or tests.
- 153 anchors explicitly retain compiler/design-blocked typed views or tails.
- 102 anchors are deliberate exclusions with reasons; none is pending.

The combined audit merges overlapping scalar/table coverage, so a table symbol
excluded from the scalar slice is covered by its actual table translation.
Three additional Rust-only scaffolding files are explicitly excluded:
`enums.rs` contains the newtype/formatting macro; `protocol/mod.rs` and
`table/mod.rs` contain module/reexport topology. All macro invocations have
reviewed open carriers/constants in their owning slices. The other exclusions
are documented formatting/trait/namespace conveniences and equivalent alias or
assertion spellings. Macro assertion rows are not declarations in the lexical
index; upstream probes still check the associated representations.

The separately audited `uefi/src/table/cfg.rs` additions are retained in the
table slice but are outside this whole-`uefi-raw/src` file count.

## Verified evidence

The nine vector documents contain **4,194 expected measurements**. Their schema,
source inventories and applicable Rust probes pass. Scalar geometry includes
reviewed expectations and the companion cross-target probe documented in its
port record; not every scalar vector is a Rust-produced observation. All other
slice probes compare actual pinned Rust types/constants against the specified
UEFI x64 target. Probes assert GUID byte order and do not execute firmware.

The consolidated Omega source root imports all 26 raw modules and checks 36
source units. The fixed-plan root imports console, storage, machine, TCG, HII,
and the 80 supported network plan definitions. These checks validate authored
source; they do not select or materialize every foreign representation.

Scalar, table, storage, console, network and HII semantic fixtures pass. Their
original changed-final-assertion controls reject failure. The whole-crate runner
also changes an expected behavior **inside each test body**, keeping its final
success assertion unchanged: each computes failure 1 and rejects `1 == 0`.
This establishes that the executed behavior checks influence the result. The
console and HII pinned Rust behavior tests also pass independently.

On 2026-09-20 the consolidated checks ran against a fresh release build from a
clean sibling Omega checkout:

- Revision: `eaa7993a23623cd8fabf45350340479c5c9c7879`.
- Binary SHA-256: `2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
- Build: `cargo build --locked -p omega --release --target-dir /tmp/cathedral-omega-eaa7993` from Omega.
- Host: macOS ARM64. Rust probes use the installed `x86_64-unknown-uefi` target.

Individual slice records retain their earlier, accurately identified binary
observations. This fresh audit supplements them; it does not rewrite those runs
as builds of a revision that was not then established.

## Confirmed blocked consumers and tests

| Required behavior | Current evidence / next prerequisite |
| --- | --- |
| Full named BootServices, ShellProtocol and PxeBaseCodeMode plan reflection | Current `Schema.fields` capacity is 32. Source checks reject indices 32–44, 32–45 and 32–33 respectively. All fields remain transcribed. |
| Cross-package projection through selected public layouts | Current compiler rejects InputKey.scan_code, NetworkStatistics.rx_total_frames, PxeBaseCodePacket.raw and HiiDate.year as private synthesized fields. Dedicated reproducers remain beside the slice tests. |
| Whole selected console carrier corpus | `layout_probe.omg` rejects PlacedField read/take/write callable generic arity (expected zero, found one). Small InputKey/device-path selection checks pass; this is not a claim that every array layout is unsupported. |
| Typed HII union overlays | Omega layout documents leave programmable union source forms unspecified. Four unions and their dependent carriers preserve visibly named backing storage; typed construction/projection/default semantics remain blocked. All 185 HII fixed carrier selections pass independently. |
| Runtime tails in device paths, storage, networking and HII | Fixed prefix geometry is preserved. General typed runtime extent/stride source forms and validated backing are still required; no tail access test is claimed. |
| Native firmware protocol slots and callbacks | Named calling-policy selection, address materialization, and admitted foreign backing/lifetime/callback contracts remain necessary. Raw addresses authorize nothing. |
| Independent emitted Omega ABI observations | No selected/materialized foreign artifact is produced by these checks. `vectors.py --observed` must receive actual Omega inspection evidence; expected vectors and source plans cannot substitute for it. |

The reflection and projection/selected-console failures above reproduce with
the fresh compiler revision. Expected failures are reported as **confirmed
blockers**, never passing layout tests. The fixed HII selection probe still
passes on that same compiler.

## Reproduce

```sh
python3 tools/ports/uefi-conformance/inventory.py
python3 tools/ports/uefi-conformance/check.py --omega /path/to/current/omega
```

On Windows use `python` and the corresponding `omega.exe` path. The runner uses
portable Python/Cargo subprocesses. `--host-only` explicitly skips Omega;
`--omega-only` runs source/semantic/blocker checks after a separately completed
upstream audit. Both scopes were executed for this audit. A missing checkout,
target, compiler or changed expected diagnostic fails clearly. To update the
index after reviewed slice changes, run `inventory.py --write` and inspect its
diff. Pin changes require the separate license/source review described in the
repository notices.

Existing boot-facing contracts retain their prior authority and lifecycle
policy. None imports the new raw corpus. Legacy production-canary harness drift
is not a language blocker, and these isolated checks do not claim a new boot
image or full production-root run. Producer/consumer image and native-boundary
fixtures belong to the separate UEFI-010 record.
