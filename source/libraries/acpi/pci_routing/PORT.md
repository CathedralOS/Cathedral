# Strict detached PCI interrupt routing

Status: **tested bounded strict `_PRT` decode, selection and detached resources**.
The final repository-path check passed 117 actual Omega checked-interpreter
positives and 117 changed-body controls in 121.674 seconds. All 56 input hashes
remained unchanged; retained-record validation also passed. The suite checks
full-capacity rows, distinct second-row contents, exact `_CRS` paths, every
retained resource-buffer byte and declared-scope rejection.

This child package decodes supplied, already-evaluated `_PRT` values, selects a
route, constructs a detached `_CRS` request, and selects an interrupt descriptor
from supplied resource bytes. It does not evaluate AML, acquire resources, install
interrupts, dereference device pointers or grant hardware authority.

## Upstream and specifications

The source is [rust-osdev/acpi, pci_routing.rs at
257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/pci_routing.rs),
crate 6.1.1, MIT OR Apache-2.0, copyright 2018 Isaac Woods. This is a modified,
bounded translation. Exact notices and licenses are retained in the repository's
`THIRD_PARTY_NOTICES.md` and `licenses/rust-osdev/acpi/`.

[ACPI 6.6 §6.2.14, `_PRT`](https://uefi.org/specs/ACPI/6.6/06_Device_Configuration.html#prt-pci-routing-table)
defines the four members, DWORD address and source index, function FFFF,
zero-based pin, NamePath-or-zero source, and physical resource-descriptor index.
NamePath means an AML name reference, not a String literal. Resource decoding
reuses the existing [bounded resource package](../resources/PORT.md), including
its documented strict template profile and unsupported descriptor envelopes.

## Complete source map

`inventory.json` binds the complete pinned file and all seven lexical declaration
anchors. The following map also accounts for private fields, semantic enum cases
and live operations that the lexical scanner does not index individually.

| Pinned declaration/body | Strict detached representation or operation |
| --- | --- |
| `IrqDescriptor` reexport | Reuse `resources::resource_model::Irq`; no duplicate IRQ type. |
| `Pin::{IntA,IntB,IntC,IntD}` | Validated numeric pins 0, 1, 2, 3. No native enum layout claim. |
| `PciRouteType::{Gsi,LinkObject}` | `Target::{Gsi,Link}`; Link retains resolved canonical `Path` and `source_index`. `None` represents an empty/failure result. |
| `PciRoute::{device,function,pin,route_type}` | `Route::{device,pin,target}`. All admitted functions are FFFF, so function state is unnecessary. Device retains its declared 16 bits. |
| `PciRoutingTable::entries` | `Routes` contains a count and 32 initialized `Route` slots. |
| `from_prt_path`: `evaluate` | Caller supplies the evaluated package identity in canonical `ObjectStore`; evaluation remains outside this kernel. |
| `from_prt_path`: packages and fields | `routes::decode`, canonical `object_references::package_element`, exact four-field validation and `unwrap_all` on Source only. |
| `from_prt_path`: level lookup | `routes::resolve_level` resolves a captured NameReference against its declared scope; bare NameSeg searches nearest levels then ancestors. |
| `route`: first-match device/function/pin | `routes::select` validates the complete ordinary table then chooses its first device/pin match. Query function is omitted because every admitted row is wildcard FFFF. |
| `route`: GSI default IRQ construction | Retain the GSI number as `Target::Gsi`. Pin-generated level/low/shared/consumer flags are not new observations or capabilities. |
| `route`: `_CRS` name and evaluation | `routes::crs_request` produces the canonical absolute child path and retained source index. Caller evaluates separately. |
| `route`: resource list selection | `link_resources::select_buffer` or `select_bytes` selects the indexed physical descriptor after complete template validation. |
| Rust allocation, Debug/Clone/Eq derives | Ordinary initialized bounded values; no allocator or runtime formatting port. |

## API and bounded profile

`routes::decode(store, object, budget)` reads canonical `ObjectStore` and accepts
only a Package whose count is at most 32. Each row is exactly four elements. The
canonical package helpers validate the whole advertised linked chain, including
cycles, missing members and unexpected tails. `budget` is a per-chain/reference
bound of 0..64; it is not a total instruction budget. Canonical storage limits
remain 64 objects, 32 namespace entries and 16 path segments. The empty package
succeeds without traversing members. Shared valid member-chain identities are
permitted, so 32 captured rows can fit within the object profile.

Address and SourceIndex must fit unsigned DWORDs. Address low bits must be FFFF;
pin must be 0..3. Source accepts Integer zero or a captured NameReference after
canonical reference unwrapping. Strings are rejected. No implicit unwrapping is
performed for Address, Pin or SourceIndex. The package publishes its candidate
only after every row succeeds. Any failure returns count zero and **all 32 rows
empty**, with the failing row and error retained. Caller storage is read-only.

Name resolution validates name/scope syntax first, then requires the declared
scope to exist as a namespace level. A single relative segment searches that
level and its ancestors. Ordinary object slots do not shadow levels. Absolute,
parent-prefixed and multi-segment paths resolve against the declared scope and
must identify an existing level. Rejecting a missing captured scope is an
explicit strict capture policy: the pin can instead climb from a nonexistent
scope to an existing ancestor. This is a level-specific adapter, not a general
NameReference/object-query resolver.

`routes::select(table, device, pin)` validates all populated entries before
selection, including rows after a prospective match. Every Link must contain a
valid absolute path. This protects the public API from malformed caller-built
ordinary tables. `crs_request` returns an inert path/index request and validates
that `_CRS` fits the canonical path capacity.

`link_resources::select_buffer` uses canonical `byte_storage::read_bytes`,
including Source/Owned buffer identity validation, virtual zero padding and
transparent reference handling. That canonical snapshot is limited to 256 bytes.
`select_bytes` independently accepts initialized `[u8;4096]` with logical length
at most 4096. These bounds are implementation profiles, not ACPI maxima.

Both selectors first validate the **complete** template: framing, implemented
semantic checks, exact final EndTag and checksum rules. SourceIndex counts each
physical descriptor before EndTag, including recognized unsupported envelopes.
The selected descriptor must be an IRQ with exactly one current interrupt
number. This handles mask and ExtendedIRQ table representations. Other unsupported
descriptors remain framed but semantically unvalidated, as the resource package
documents. An index selecting one returns an error, not a guessed IRQ.

`ResourceSelection` retains the canonical IRQ and initialized backing bytes
because IRQ source/table spans refer into those bytes. Unused backing is zero.
Failed results have length zero and zero backing; the canonical AML, byte-storage
and resource outcomes distinguish the observed failure. The type carries data,
not an evaluated object token or installed interrupt.

## Deliberate differences from the pin

The independent public Rust harness preserves the pin's observed behavior. This
kernel provides one strict profile; it has no compatibility branch for defects.

| Pin behavior | Strict behavior |
| --- | --- |
| Directly indexes short rows; ignores extras; a three-field Link may succeed | Require exactly four fields and validate complete package chains. |
| Truncates address high bits and GSI to u32; permits a specific function | Reject DWORD overflow and require FFFF. |
| Accepts legacy String link names, using `_PRT` scope | Accept captured NameReference only, retaining its declared scope. |
| May leave non-search relative names unresolved or accept missing absolute links | Resolve and validate every Link level during decode. |
| Discards Link SourceIndex without validating its type | Require DWORD SourceIndex and retain it through `_CRS` request/selection. |
| Filters unsupported resources and requires one retained resource overall | Count physical descriptors and select SourceIndex; unrelated descriptors can coexist. |
| Accepts missing EndTag, ignores checksum/trailing bytes; some short buffers panic | Reuse explicit strict template errors and bounded reads. |
| Returns IRQ masks/tables containing multiple interrupts | Require exactly one current interrupt in the selected `_CRS` IRQ descriptor. |
| Unbounded Vec and live interpreter calls | Explicit bounded arrays and caller-supplied evaluated values/bytes. |

## Verification and integration

The [public Rust harness](../../../../tools/ports/acpi/pci-routing/README.md)
retains 101 original authored AML/resource fixtures and actual public
`Interpreter`/`PciRoutingTable` observations. Hardware and service callbacks trap;
only the inert interpreter mutex constructor is permitted. Those records remain
independent and unchanged. Their prospective strict expectations are not claims
that Omega executed those exact AML tables.

The [kernel harness](../../../../tools/ports/acpi/pci-routing-kernel/README.md)
constructs canonical captured values and resource buffers directly. Every
positive body has a mutated expected-behavior control, evaluated through the
actual checked Omega interpreter. Decode-value fixtures inspect all 32 output slots on success and failure,
including full capacity and late failure. Resource fixtures compare all 4096
output bytes, including zero tail and full zero output on failure. Separate selector fixtures validate
malformed ordinary tables and missing matches. Strict corrections
are separate expectations, not asserted equality with known pin defects.

No native ABI, live firmware, production interpreter integration or complete
PCI interrupt-routing service is claimed. The final checks use the published canonical storage and namespace dependencies.
Their exact current source hashes are retained in the checked record; root-level
namespace/query integration remains a separate milestone.

### Shared guard correction

Canonical path validation/equality and namespace lookup/allocation/binding now
stage their u64 counts in typed locals before comparisons. On the pinned checked
evaluator, direct record-field comparisons could admit a high-bit count and
enter an excessive validation loop. PCI decode stages namespace entry count too.
Forty-two added cases cover counts at the first invalid limit, 2^63 and u64::MAX
through namespace APIs, decode, link resolution, route selection and `_CRS`
requests. The mathematical bounds and path-equality policy are unchanged.

The shared guard change also passes fresh 96 object-query runtime pairs and 27
parser pairs, recorded under the kernel harness's `regressions/` directory.
Earlier query const and full306 regression records remain historical receipts
for commit `600eb26`; their raw hashes are preserved. The unchanged byte-storage
kernels do not call the modified namespace APIs. No compiler source was changed.
