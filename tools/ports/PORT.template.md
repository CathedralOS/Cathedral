# <Package / precisely named translation slice>

> Copy this file beside the translated package as `PORT.md`. Replace every
> placeholder; use explicit `none` or `not run` with a reason where applicable.
> Paths below are relative to this port directory unless labelled otherwise.

## Scope and status

- Queue task(s): `<task IDs>`.
- Claimed upstream slice: `<crate/modules/files; explicit exclusions>`.
- Current stage: `inventoried`.
- Stage date / Cathedral revision reviewed: `<date / revision>`.
- Source review: `<reviewed files/specification editions and sections>`.
- Production build inclusion: `<excluded, or exact build root and dependency>`.
- Remaining implementation work: `<work still required; not a language blocker>`.

Choose the highest stage established for the **whole claimed slice**; list
mixed per-file stages in the source map. Completion of a task must meet that
task's own deliverables, even when this port's current stage is earlier.

| Stage | Minimum evidence; additional limits remain explicit below |
| --- | --- |
| `inventoried` | Pinned source and license reviewed; files/public symbols mapped, including omissions and blockers. No translation or compiler success implied. |
| `transcribed` | Claimed translation authored and reviewed against its inventory; deviations and blocked seams explicit; upstream tests translated or retained as unexecuted fixtures/vectors. No compiler or ABI success implied. |
| `typechecked` | Exact Omega revision/target/commands successfully check the claimed translation. Excluded or blocked declarations cannot inherit that claim. |
| `tested` | Claimed translated tests actually execute and pass; list test coverage and unexecuted cases separately. Host-only vector validation does not establish this stage. |
| `integrated` | Named Cathedral build/adapters and authority/lifecycle contracts are integrated and their applicable checks pass. Record hardware execution separately; integration does not imply hardware was exercised. |

## Upstream pin and licensing

- Project / repository URL: `<URL>`.
- Exact upstream commit: `<full commit ID; never a branch name alone>`.
- Upstream crate/version (informational): `<crate/version>`.
- Optional local checkout: `<repository-root-relative reference_code/...>`.
- License at this pin: `<SPDX expression verified from pinned files>`.
- Preserved/chosen license: `<MIT OR Apache-2.0 for substantial queued translations unless owner chooses otherwise>`.
- Pinned license/notice paths: `<paths and any per-file exceptions>`.
- Retained license texts / notices: `<links to committed files>`.
- Root notice entry: `<link to THIRD_PARTY_NOTICES.md entry>`.
- Modified-file identification: `<source headers or explicit complete mapping>`.
- Pin audit: `<date, source/license checks; link separate pin-change review if any>`.

Distinguish original Cathedral code, primary-source facts, and derivative
translations per file. Copying organization, algorithms, tests, or prose retains
upstream provenance even when all source syntax changes.

## Source and public-symbol map

- Machine-readable inventory: `<path; exact upstream slice and symbol coverage>`.
- Inventory command/result: `<command and evidence link; absent checkout is failure>`.

| Upstream path / symbol or explicit symbol set | Cathedral file / symbol | Origin category | Disposition / stage | Reason, blocker, or deviation |
| --- | --- | --- | --- | --- |
| `<path::symbol>` | `<file::symbol, or none>` | `<fact / derivative / original adapter>` | `<mapped / omitted / blocked; stage>` | `<reason or none>` |

Map every file and public constant, representation, and operation in the
claimed slice. Link exhaustive companion manifests if this table would be too
large. An omitted symbol needs a reason; an unlisted symbol is not an omission.
Existing local declarations need collision/compatibility review before adding
parallel representations.

## Primary specifications and representation vectors

| Declaration / fact | Governing specification, edition, section | Upstream cross-check | Vector/fixture path and target profile |
| --- | --- | --- | --- |
| `<declaration>` | `<specification citation>` | `<pinned path/symbol>` | `<path; pointer width, endianness, ABI>` |

Record size, alignment, field offsets/order, discriminants, flags, GUIDs, and
status values as applicable. Distinguish specification-authored expectations,
upstream-derived expectations, measured upstream output, and measured Omega
output. Record any disagreement and resolution. State unsupported profiles.

- Vector format/version: `<format/version>`.
- Upstream generation command and environment: `<command, compiler, target; or not run>`.
- Omega comparison command/result: `<command and output; or unavailable with blocker>`.
- Layout/ABI claim: `<exact observed coverage; unmeasured when no consumer exists>`.

## Translated tests and fixtures

| Upstream test/path or spec case | Cathedral test/vector | Cases and expected outcome | Execution state / evidence | Blocker |
| --- | --- | --- | --- | --- |
| `<test>` | `<path>` | `<normal/boundary/malformed/overflow>` | `<passed / failed / not run; command/result>` | `<ID or none>` |

Record tests with no upstream equivalent as locally authored. Upstream Rust
tests and host fixture checks are useful evidence, but do not count as passing
Omega tests. Unexecutable translated tests remain labelled fixtures or vectors.

## Omega blockers and boundary seams

| Blocker ID | Affected declarations/tests | Required language behavior / unresolved design | Contract and implementation evidence | Minimal reproducer / exact diagnostic | Work that can continue |
| --- | --- | --- | --- | --- | --- |
| `omega:<short-name>` | `<symbols>` | `<required behavior>` | `<spec path/section, Omega revision/source>` | `<command/fixture/diagnostic; or not attempted and why>` | `<independent work>` |

Put `PORT-BLOCKED[omega:<short-name>]: <required behavior>` at each unresolved
source seam and link it here. Do not guess language semantics or edit Omega to
make this port compile. A task is design-blocked only by an actual language
implementation gap or unsettled design. Ordinary implementation effort and
missing local tool setup remain work, not language blockers.

For each upstream `unsafe` operation, record the invariant it relied on and
whether ordinary Omega proves it or a named boundary must establish it. Include
pointer lifetimes, aliasing, bounds, external mutability, and calling/custody
requirements where applicable. Parsed addresses and ABI slots confer no access
or service authority.

## Deliberate deviations

| Upstream behavior / ABI / API | Cathedral translation | Reason and impact | Compatibility/test evidence |
| --- | --- | --- | --- |
| `<behavior>` | `<change or omission>` | `<reason; policy belongs in adapter>` | `<evidence or unverified>` |

State `none` only after source-map review. Identify local policy separately from
generic behavior, including existing local restrictions and planned migration.

## Cathedral integration and authority

- Owning layer and charter: `<path/link>`.
- Pure representation/algorithm surface: `<symbols>`.
- Authority-bearing operations and named seams: `<I/O, MMIO, firmware calls, DMA, instructions, etc.>`.
- Cathedral adapter and grants/lifecycle review: `<path/evidence, or not integrated>`.
- Build roots containing the port: `<exact list, or none>`.
- Explicitly excluded/untypable source: `<exact list and why>`.
- Affected existing canaries: `<commands/results; failures and limitations>`.
- Hardware/simulator execution: `<target, image revision, command/result; or not run>`.

Keep pure parsing and state transitions separate from access. Raw-contract
conformance, mock service execution, and successful layout comparison do not
establish firmware-service authority or Cathedral lifecycle integration.

## Verification commands and results

Run from `<working directory>`. Record the exact Cathedral and Omega revisions,
target/profile, command, exit/result, and evidence for each claim. If reviewing
uncommitted work, identify that state and do not attribute it to an older commit.

| Check | Exact command | Environment / revision | Result / evidence and scope |
| --- | --- | --- | --- |
| Source/license/inventory review | `<command>` | `<pin>` | `<result>` |
| Fixture/vector validation | `<command>` | `<host/profile>` | `<result; no Omega-layout claim>` |
| Omega typecheck | `<command or unavailable>` | `<Omega revision/target>` | `<result or not run/blocker>` |
| Translated test execution | `<command or unavailable>` | `<Omega revision/target>` | `<result or not run/blocker>` |
| Omega layout/value comparison | `<command or unavailable>` | `<Omega revision/target>` | `<result or not run/blocker>` |
| Affected Cathedral canaries | `<command>` | `<revisions>` | `<result or not run/reason>` |
| Hardware/integration | `<command or not applicable>` | `<image/firmware/target>` | `<result or not run/reason>` |

List unavailable checks explicitly. A successful command supports only the
files and behavior it actually covers; a green manifest check cannot promote
an uncompiled translation to `typechecked` or `tested`.
