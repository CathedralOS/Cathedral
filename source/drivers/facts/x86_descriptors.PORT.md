# x86 descriptor, GDT and TSS slice

## Scope and status

The bounded pure X86-001/002 descriptor slice is source checked and semantically
tested with Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`. It supplies six
raw fact carriers and one semantic sum, 22 descriptor flag/default constants, two result carriers,
27 pure descriptor/validation machines and eight complete byte-codec machines.
Four explicit policies request fixed layout geometry. Actual imported-layout
consumers retain the specific failures below; native layout equality and live
CPU integration are not claimed.

This slice represents GDT entry words and detached append decisions, not the
complete generic mutable `GlobalDescriptorTable<MAX>` storage API. Generic
storage construction, full entry-slice copying and atomic/live entry observation
remain separate engineering. Their omission is not a language blocker. Existing
SegmentSelector, IST assignments and IDT gate/exception facts remain canonical.

## Upstream pin and licensing

[x86_64 0.15.5](https://github.com/rust-osdev/x86_64) at
`cc35c876d3badb57df54a66e22f7768a52be95f2`, MIT OR Apache-2.0.
See [licenses](../../../licenses/rust-osdev/x86_64),
[additional IDT source notice](../../../licenses/rust-osdev/x86_64/SOURCE-NOTICES.md),
[root notice](../../../THIRD_PARTY_NOTICES.md) and
[baseline provenance](../../libraries/x86_64/PORT.md). Derivatives identify the
pin/modification and preserve the IDT copyright notice for selector-error work.

## Source and public-symbol map

[x86_descriptors-inventory.json](x86_descriptors-inventory.json) binds four exact
source files and all 244 lexical anchors: 69 translated pure components, 175
explicit omissions, plus 15 supplemental enum/payload mappings. An anchor marked
translated can identify an extracted pure component of a larger Rust operation;
its reason names that narrower claim. A source mapping alone is not execution
or ABI evidence. One retained `was` anchor is a format string, not a declaration.

| Pinned source | Ported component |
| --- | --- |
| `src/structures/mod.rs` | DescriptorTablePointer raw fields; pointer size/offset tests |
| `src/structures/tss.rs` | Complete fixed TSS fields/default; InvalidIoMap error data; size test |
| `src/structures/gdt.rs` | Entry word, descriptor cases/words, all 15 flags, COMMON and six defaults, DPL, default descriptors, capacity/limit/raw-metadata/append plans, TSS packing and bitmap checks |
| `src/structures/idt.rs` | Selector-error validity/truncation, external/table/index/null codecs and explicit table codes |

The Rust `Descriptor` enum remains a semantic sum: `UserSegment(low)` or
`SystemSegment(low, high)`. One/two-word width follows from the active case;
there is no user-supplied word count or invalid high-word state. Its ordinary
sum layout is not the hardware one/two-word encoding. `TssDescriptorWords` is a separate exact two-word codec
carrier. `GdtEntry` describes detached numeric bits; it does not reproduce the
instructions-enabled Rust atomic entry operations.

## Primary specifications and representation vectors

[Intel SDM Volume 3A](https://cdrdv2-public.intel.com/835754/253668-sdm-vol-3a.pdf),
64-bit TSS Figure 9-11 and descriptor-table/task-management sections, is the
primary geometry reference. [AMD APM Volume 2](https://docs.amd.com/v/u/en-US/24593_3.44_APM_Vol2)
is the complementary system-programming reference. The measured oracle is the
exact Rust pin, not an assertion that every value was independently rederived
from both manuals.

| Representation | Requested size / alignment | Measured pinned Rust evidence |
| --- | --- | --- |
| Descriptor pointer | 10 / 2 | size 10, alignment 2, limit offset 0, base offset 2 |
| Fixed TSS | 104 / 4 | size 104, alignment 4, RSP array offset 4, IST array offset 36, bitmap base offset 102 |
| GDT entry word | 8 / 8 | size 8, alignment 8 |
| Encoded TSS descriptor words | 16 / 8 | Architectural pair, not the native Rust enum layout |

[x86_descriptors.vectors.json](x86_descriptors.vectors.json) records 32 actual
Rust host observations, each independently asserted for `x86_64-unknown-uefi`:
21 public flags/defaults and 11 size/alignment/public-offset values. COMMON is
private upstream and is checked as source-derived composition. TSS reserved
field offsets are source/specification-derived requested geometry, not invented
private-field `offset_of!` observations.

Little-endian codecs encode/decode every byte of the 10-byte pointer, 104-byte
TSS, eight-byte entry and 16-byte TSS descriptor pair. They preserve all raw
fields, including reserved bits; decoding is not architectural validation.
The typed TSS arrays remain arrays. An invented flattened semantic schema is
not used to hide the packed-array layout failure.

## Translated tests and fixtures

[Tooling](../../../tools/ports/x86_64-descriptors/README.md) reproduces source,
schema, plans, codec bodies, inventory and fixtures. Four Rust tests execute
actual upstream pure APIs and extracted pinned TSS encoder/bitmap-validation
bodies. Extraction replaces pointer/last-byte observation with explicit numeric
inputs; bit composition, comparisons and error order remain unchanged. Tests
include all 65,536 selector error codes, GDT append/full behavior, TSS defaults,
packing and all bitmap failures/boundaries. No privileged instruction is run.

The Omega positive fixture evaluates actual translated bodies in a constant and
requires result zero: 22 flag comparisons, 16 TSS descriptor inputs, 18 selector
error cases, every byte of four codecs and every decoded field/array element,
GDT selector/capacity/limit cases, bitmap error order, empty input and the full
8,193-byte input. Four body mutations change a system-case payload expectation, bitmap limit,
GDT selector and a TSS encoded byte; all must evaluate to one and fail the unchanged zero contract.

The IST helper consumes the existing `X86IstStackClass` type. Its four literal
pair tests execute `ist_slot`; a source audit binds those pairs to the four
unchanged canonical constants. Imported legacy aggregate constants currently
fail constant evaluation as described below, so these tests do not claim to
execute those imported aggregate constant expressions.

## Omega blockers and boundary seams

- **Imported plan-laid field visibility:** after splitting the pointer's base
  into four aligned 16-bit fragments, the actual imported pointer field consumer
  fails `selects private data DescriptorTablePointerLayout<DescriptorTablePointer>::limit`.
  A local equivalent pointer schema consumes the same policy and checks both
  limit and fragmented base field access. Local equivalent entry/two-word schemas
  also exercise their policies. This proves source plan/access checking, not
  native emitted bytes or the imported type's usable accessors.
- **TSS packed arrays:** the actual TSS consumer fails `field
  privilege_stack_table at offset 4 violates its alignment 8`. Current whole-array
  `At` placement preserves `u64` element alignment; scalar reserved fields can
  be fragmented into 32-bit pieces but that does not provide an aggregate-element
  fragmentation policy. The 104-byte explicit codecs are independent and pass.
- **Legacy aggregate constant evaluation:** `legacy_ist_constant_probe.omg`
  reproduces `retained initializer has no checked aggregate leaf correspondence`
  when evaluating the imported IST record constants. Constructing the existing
  type from literal pair values and running the actual helper succeeds.

`layout_probe.omg` names all four policies/types and passes, but this type-only
probe does not validate their actual field plans. The field-demand probes above
supply the stronger evidence. No layout statement grants backing, valid stack
addresses, live table lifetime, CPU root identity or descriptor loading.

## Deliberate deviations

All address carriers hold inert numeric bits, including noncanonical input.
Rust `VirtAddr` invariants, static borrows and pointer conversions are not
reconstructed from equal numbers. Descriptor packing accepts an explicit
16-bit limit; callers wanting the pinned TSS limit rule use the bitmap planner.
Descriptor shape validity checks encoded flags/reserved words, not live backing
or architectural usability of every supplied limit.

Rust panics become explicit geometry/full errors in append planning; invalid
descriptor shapes are excluded by the semantic cases.
The plan reports index, next length and the existing SegmentSelector carrier;
it does not perform a write. Bitmap validation retains the pinned order: too
long, before base, too far, terminator, declared base. Empty input ignores a
supplied last-byte value. As upstream does, the detached numeric checks can
accept an offset inside the fixed TSS; this is not evidence of valid overlapping
objects or a safe installed bitmap. The initialized fixed byte borrow is
supplied independently of numeric addresses and never fabricated from them.

The only existing fact change is `[copy]` on `X86IstStackClass`, a pair of pure
policy integers. Its legacy import surface, fields and constants are preserved.

## Cathedral integration and authority

New facts are [x86_descriptors.omg](x86_descriptors.omg), requested policies are
[x86_descriptor_layouts.omg](x86_descriptor_layouts.omg), and pure algorithms are
[descriptors.omg](../../libraries/x86_64/descriptors.omg) and
[descriptor_bytes.omg](../../libraries/x86_64/descriptor_bytes.omg). No production
core, competing IDT/PTE record, instruction provider or installed root changes.
Live GDT/TSS storage, pointer lifetime and CPU loading remain integration work.

## Verification commands and results

```sh
python3 tools/ports/x86_64-descriptors/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

Compiler SHA-256:
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
The producer checks 14 sources and the positive fixture checks 17. The canonical
runner also checks generated freshness, pin coverage, all Rust observations and
UEFI-x64 assertions, four Rust tests, local layout consumers, policy pairs and
four executed-body negative controls.

The original stack-set/profile harness uses obsolete build calls and removed
JSON artifact expectations. `check_existing.py` stages modern packaging and
only replaces its retired bundled std import with a declared std package alias.
With Cathedral `cbbf9b6c9c9365728c24fcedafe6cb6fdd3bfdda` original IST facts, actual checking reports four missing-copy
constant errors plus a `x86_scan_bootstrap_exception_table_policy` vector-range
proof error. With the copy-only edit, the same vector-range error remains and
all four constant errors disappear. The pre-existing profile is therefore not
claimed to pass; no core proof or policy was altered to conceal that result.
