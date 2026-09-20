# ACPI source inventory and partition (ACPI-000)

## Scope and status

**Stage: inventoried, 2026-09-20.** This milestone covers the complete pinned
Rust corpus and test-scenario metadata. It does not claim an Omega translation,
compiled parser, interpreted AML, or hardware integration. The cumulative source map now records 480 translated table/topology/AML/helper anchors and 1,101
pending anchors. Inventory completion and translation completion are different
milestones. The bounded ACPI-001 slice is separately
[tested by Omega semantic evaluation](headers.PORT.md): 65 original scenarios
and three body-mutating negative controls pass; native and firmware execution
remain untested.

The implemented ACPI-001 slice covers initialized-byte RSDP/SDT headers,
checksums and RSDT/XSDT entries within its explicit 4096-byte input profile.
Fixed tables and topology have separate scope/verification reports in
[fixed.PORT.md](fixed.PORT.md) and [topology.PORT.md](topology.PORT.md).
Those slices pass 291 and 77 original Omega semantic scenarios respectively,
each with three body-mutating controls. The [AML helper slice](interpreter/PORT.md)
passes 150 integer/byte scenarios and three body controls. The [static AML layer](aml/PORT.md)
passes 27 semantic cases and 27 body controls for bounded parsing, declarations
and namespace behavior. [Field declaration metadata](aml/fields/PORT.md) adds
18 cases and 18 body controls for all three declaration families and five
FieldList forms. Bytecode execution, runtime field installation and complete
object/context behavior remain separate queue work.
No production build root imports this directory. The libraries charter governs
pure algorithms; firmware/physical-address facts convey no mapping authority.

## Pin, licensing and exceptions

- [rust-osdev/acpi](https://github.com/rust-osdev/acpi), exact commit
  `257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, package version 6.1.1.
- Checkout: repository-root `reference_code/rust-osdev/acpi`.
- Manifest spells `MIT/Apache-2.0`; README explicitly grants the dual license.
  Preserve `MIT OR Apache-2.0` for derivative translations and modified-file
  notices. Root licenses retain `Copyright (c) 2018 Isaac Woods`.
- Exact pin/working bytes and existing retained license copies are checked by
  `tools/ports/acpi/inventory.py`; license/Cargo/README SHA-256 evidence is in
  `test-scenarios.json`.
- Retained texts: [licenses/rust-osdev/acpi](../../../licenses/rust-osdev/acpi/).
  Root notice: [THIRD_PARTY_NOTICES.md](../../../THIRD_PARTY_NOTICES.md).

The crate grant is not treated as a blanket grant for external fixtures:

| Input | Audit finding / handling |
| --- | --- |
| `tests/pc-bios_acpi-dsdt.asl` | Disassembled firmware with Intel ACPICA disassembler notice; firmware origin/license unresolved. Hash/scenario metadata only; no fixture import or translation. |
| `tests/uacpi_examples.rs` | Explicit adaptations from the uACPI README; exact original revision/license notices need separate review before copying those scenarios. |
| `tests/global_lock.rs::uacpi_global_lock_test` | Explicit adaptation of uACPI `tests/test-cases/global-lock.asl`; separate original provenance review required. Other tests in that file remain separately listed. |
| External uACPI suite | Not present or pinned by this checkout; no suite/source license or execution result inferred. |
| `tools/no-alloc-check` | README credits an external zulinx86 example; implementation would require that provenance review before translation. Only metadata inventoried. |
| Remaining crate-authored tests | Root grant reviewed; no separate external origin identified in the audit. This is not a claim to have proved provenance beyond the available source notices. |

Some tools lack their own Cargo license field; metadata inventory does not
copy those implementations or settle external-example licensing. Actual
translation must retain the origin of every imported dependency or fixture.

## Exhaustive maps

- [inventory.json](inventory.json): all **52 Rust files, 1,581 lexical anchors**
  under `src/`, `tests/`, `tools/`, including the excluded no-allocation tool.
  Every anchor has partition labels, queue tasks and an implementation status.
  The table/topology/static-AML/helper slices overlay only translated anchors; their narrow omissions do
  not remove future work from this complete corpus.
- [partitions.json](partitions.json): per-file SHA-256, categories, queue tasks,
  and observed allocation/concurrency/unsafe/host-I/O indicators.
- [test-scenarios.json](test-scenarios.json): **66 Rust tests, three ignored**,
  **19 ASL/AML assets**, hashes, named ASL methods and provenance exceptions.

The lexical index binds complete file hashes but does not expand macros or
pretend to be a Rust semantic parser. Symbol partition labels describe work;
a file can contain multiple kinds. Indicators are review aids, not inferred
proofs that a function is pure or safe.

## Partition and ownership

| Source surface | Work partition | Destination / constraints |
| --- | --- | --- |
| `rsdp.rs`, `sdt/mod.rs`, `sdt/*.rs` | Byte/table facts and table parsing | ACPI-001/002; initialized bounded bytes, explicit little endian and structural lengths; no reference cast into untrusted input |
| `lib.rs::AcpiTables`, `AcpiTable` | Discovery plus physical handler boundary | Split entry decoding/selection from mapping and table custody; addresses remain data |
| `address.rs` | GAS representation/validation plus `MappedGas` access | Retain region-space/access-width facts; do not translate raw volatile mapping into ambient access |
| `registers.rs` | Fixed-register operations and status/control bits | Facts can translate independently; read/write effects, event enabling and sleep control are later boundaries |
| `platform/{interrupt,numa,pci}.rs` | Platform topology | ACPI-003 inert CPU/controller/NUMA/PCI-region descriptions, without controller/MMIO authority |
| `platform/mod.rs` | Topology plus platform control | Split pure descriptions from event initialization, ACPI mode transition and AP wakeup mailbox writes |
| `aml/mod.rs` cursor/opcode/package/name parsing | AML syntax | ACPI-004 bounded deterministic input and namespace construction |
| `aml/mod.rs` interpreter/method operations | AML execution | ACPI-005 arithmetic/conversions/control flow/packages/fields; ACPI-006 work/recursion/output limits |
| `aml/namespace.rs` | Namespace plus compatibility policy | Separate lookup/normalization from `_OS`/`_OSI` claims; upstream OS impersonation is not a Cathedral default |
| `aml/object.rs` | Value algorithms plus shared mutable object ownership | Preserve semantics through explicit bounded storage/ownership; no unchecked translation of `Arc<UnsafeCell>` or the global `ObjectToken` convention |
| `aml/resource.rs`, `aml/pci_routing.rs` | Resource decoding and routing results | Syntax/byte facts separate from `_PRT` evaluation and later authority grants |
| `aml/op_region.rs`, `Handler` | Handler callbacks | ACPI-007 named physical/MMIO/port/PCI/time/wait/mutex/debug/fatal boundaries; none installed by parsing |
| `tools/` and test infrastructure | Host-only test adapters/compiler runners | Never target code. `iasl`, filesystem/process execution, real dump access and external test suites need separate harness treatment |

## Allocator, concurrency and policy assumptions

Default Cargo features enable `alloc` and `aml`. The no-allocation core can
find/enumerate raw tables, while platform extraction uses allocator-parametric
vectors and defaults to `Global`. AML adds vectors, strings, BTreeMap, Box,
Arc and SmallVec. A translation must select explicit capacities/allocators and
exhaustion outcomes; neither parser input nor firmware supplies allocation rights.

The interpreter coordinates namespace/region maps with spinlocks and a single
object mutation token, uses atomic event/global-lock state, and has explicit
unsafe Send/Sync assertions. `Handler` owns reentrant mutex operations and
infinite-wait timeout 0xffff. The pin documents that balanced method-exit mutex
release is not enforced. These assumptions require a Cathedral ownership/work
model; copying Rust marker traits cannot establish it.

`MAX_NAME_PATH_INDIRECTIONS = 8` bounds one name-resolution chain, not all AML
work. Loops, recursion, namespace growth, object allocation and returned data
still require separate ACPI-006 budgets. The upstream test README names several
external tests that can hang; they must become timeout/exhaustion scenarios,
not be silently deleted or counted as passing.

Compatibility policy is separately inventoried: `_OS` identifies Windows NT,
`_OSI` answers a hardcoded capability/OS list, and `AcpiQuirks::ignore_xsdt`
changes table selection. Those decisions must remain visible to later Cathedral
adapter review rather than entering generic parsing as hidden defaults.

## Test partitions and evidence limits

The 66 Rust test records include unit bit-copy/conversion/resource tests,
namespace lookup, fields/index/bank access, operation-region mocks, global-lock
concurrency, packages/references, malformed opcode handling and host helper
checks. Three are explicitly ignored upstream: reference rebind semantics,
increment/decrement behavior, and buffer-field implicit conversion. Ignored
means unexecuted coverage, not a passing compatibility guarantee.

ASL scenarios cover buffers/strings, integer conversions, methods/scopes,
packages, field creation, control flow, thermal/power objects and multi-table
loading. Their content is not copied into this milestone. Future tests must
retain valid/malformed cases and actual expected observations; script exit zero
alone is insufficient. In particular, the upstream uACPI adapter returns success
for unsupported `resource-tests`; that path must never become positive coverage.
The no-alloc-check executable is build-only by design and must not be run.

No upstream Rust tests, ASL compiler, AML interpreter, Omega parser, simulator
or hardware checks were run for this inventory milestone. This is not an Omega
language blocker. Missing future implementation remains ordinary queue work.

## Verification

From Cathedral root:

```sh
python3 tools/ports/acpi/inventory.py --check
python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/acpi source/libraries/acpi/inventory.json
```

Both pass against the exact pin and current source bytes. The first checks the
partition/scenario metadata and license copies; the second checks Rust anchors
and hashes. `--require-transcribed` intentionally fails while implementation
is pending. No compiler binary/revision is attributed to this metadata audit.
