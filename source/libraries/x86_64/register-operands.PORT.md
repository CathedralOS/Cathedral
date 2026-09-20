# Detached register observations and operand recipes

Status: **tested**. All 1,323 extracted Rust observations, 14 Omega batches,
four additional rejection cases and seven body mutations pass. This bounded
X86-001/002 slice closes the pure components identified as `register-merge`,
`cr3-cr8-operands` and `xcr0-validation`, plus STAR/CET/APIC composition from
`msr-composition` in the remaining-pure audit. Raw MSR two-u32 transport arithmetic
remains a separate small pending component. It changes no existing source or production root.

Modified logic comes from x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`,
`src/registers/{control,model_specific,rflags,xcontrol}.rs`; the host witness also
extracts the PCID type from `src/instructions/tlb.rs`. Retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/) apply.
The exact pinned bodies, current Omega data/case and machine guide, authority
contract and canonical register/address/page/PCID representations were consulted.

[register_operands.omg](register_operands.omg) has 17 public numeric machines.
It reuses existing flags, PriorityClass, SegmentSelector, Pcid, physical masks,
page validation and address results. New cases distinguish XCR0 assertion
failures, optional observed priority, STAR expansion and CET observation. These
are ordinary semantic results with no asserted hardware layout or authority.
Records describe numeric observations or selector tuples; no register provider,
CPU token, unsafe-pointer conversion or live instruction wrapper is introduced.

## Exact behavior and changes

`register_merge` selects the canonical CR0/CR4/EFER/RFLAGS/XCR0 known mask and
computes `(old & ~known) | supplied`. It does not truncate supplied unknown bits:
Rust flags built with `from_bits_retain` also carry them. The XCR0 composition
option alone does not validate its operand; `xcr0_plan` separately implements
the ordered pin assertions. It requires X87, then AVX's SSE dependency, the
complete MPX pair, AVX for any AVX512 state, and the complete AVX512 group.
The assertions inspect supplied flags, not the merged value. Distinct failures
are semantic cases, while success carries the merged word. CPU support,
OSXSAVE, current state, reserved-bit legality and future XCR0 requirements are
not established by this pin-specific computation.

CR3 observation extracts the default physical bits12–51 and all twelve low
bits. Separate projections return canonical Cr3Flags or the existing bounded
Pcid. Raw composition validates a supplied numeric 4 KiB frame and ORs the full
u16 argument, with optional bit63. Bits12–15 in that u16 can therefore change
address bits. Flags composition follows the upstream u64-to-u16 truncation;
PCID composition accepts the existing bounded type. These recipes never prove
CR4.PCIDE, a valid hierarchy, CR3 installation or completed invalidation.

CR8 observation truncates raw u64 to u8 before the existing priority check:
257 yields class 1, while zero and low-byte values above 15 yield None. Composition
maps None to zero and an ordinary Class payload to its value. Because the reused
raw PriorityClass carrier is publicly constructible, composition rejects forged
Class payloads outside 1–15. That extra check replaces reliance on Rust enum
construction; it does not interpret a None observation as an error or grant.

STAR expansion reads the already implemented two 16-bit fields and derives the
four selectors using +16/+8. An overflowing addition returns Overflow. The
reference runs with checked Rust arithmetic, where the original body panics;
Cathedral rejects consistently even in builds where Rust might wrap. No DPL,
GDT membership, descriptor lifetime or loaded-segment validity follows from
these numeric selectors. Existing STAR write validation remains unchanged.

CET observation separates known flags from the 4 KiB bitmap address and uses
strict canonical48 validation, matching `VirtAddr::new`. It rejects a
zero-extended address with bit47 set; it does not silently sign-extend it.
Composition checks the input bitmap's page geometry and ORs all supplied flag
bits, including retained bits outside the nominal mask. The resulting raw
operand is deliberately not reclassified as a valid bitmap or CET policy.
UCet and SCet share identical pinned pure logic.

APIC observation retains the entire raw word as raw_flags, extracts its default
physical52 frame, and projects only known flags for the typed view. Raw
composition validates the requested 4 KiB frame then ORs flags. Preserving
composition clears only known APIC flag positions in the old word before the
OR. Consequently old address bits remain: old 4096 and requested 8192 compose to
12288 with zero flags. This preserves the actual source, not an assumed address
replacement policy. The default non-encryption profile is explicit; configuring
memory encryption and admitting APIC backing remain separate work.

Panic-prone typed frame/page construction becomes rejected NumberResult inputs.
Observed raw registers and resulting words are numbers supplied by a caller,
not authenticated observations of the current CPU. No helper executes CPUID,
MOV-to-control-register, RDMSR/WRMSR, XGETBV/XSETBV or flags mutation.

## Reference and Omega evidence

The [inventory](register-operands-inventory.json) binds five complete pinned
files and 238 lexical anchors: 27 extracted/reused components and 211 omissions
outside this slice. Translating a pure component does not translate its enclosing
live read/write method. Other representations remain in their earlier slices.

The host generator extracts exact statements and assertion blocks into
`tools/ports/x86_64-register-operands/src/pinned.rs`. Explicit parameters replace
CPU reads and return values replace instruction writes. Small wrappers compose
these fragments with actual pinned flag, address, frame, page, selector and
priority APIs. The architecture-gated PCID representation/constructor/getter is
copied exactly into this witness. These are **pure-body mirrors**, not calls to
the public live register methods. No unsafe register API runs on the host.

There are 1,323 measured observations: 345 reserved-merge cases, 274 XCR0 cases,
68 CR3 projections,73 CR3 compositions,261 CR8 observations,16 CR8 operands,
35 STAR expansions,94 CET observations/compositions and157 APIC observations/
compositions. The XCR0 cases cover all 256 low-byte combinations, with additional
MPK/LWP/retained high bits. CR8 covers all 256 low-byte values plus truncation
examples. Every bit of each reserved mask is exercised. Source-bound generator
checks prevent silently replacing a fragment with a handwritten expectation.

The same observation tuples drive 14 bounded Omega batches. Each constant
initializer calls actual library helper bodies, and a success precondition
requires its computed result to be zero. Additional tests reject three forged
priority payloads and an invalid frame supplied with a bounded PCID. Seven
controls change actual expected behavior: reserved merge, ordered XCR0 error,
CR3 raw address contamination, CR8 truncation, STAR overflow, CET noncanonicality
and APIC old-base preservation. Each must compute 1 and fail the unchanged
success requirement. Expected vectors or source-only checks are not substituted
for semantic execution.

Compiler: Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
The complete canonical checker passed, including the producer (18 sources),
all batches (20 sources each), extra cases (19 sources) and all seven controls.
No native Omega execution, ABI measurement, actual CPU observation, register
mutation, production build integration or hardware acceptance is claimed.

```sh
python3 tools/ports/x86_64-register-operands/check.py --omega /path/to/omega
python3 tools/ports/x86_64-register-operands/check.py --host-only
```

`--positive-only`, `--controls-only` and `--batch N` select bounded verification
parts while retaining generator/inventory/reference checks. Pin changes require
review of whole source files, extracted fragments, assertion ordering, raw OR
semantics and explicit rejection policies before regenerating observations.
