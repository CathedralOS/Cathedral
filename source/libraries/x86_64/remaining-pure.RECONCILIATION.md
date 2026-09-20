# Remaining pure x86 work after the completed slices

This is an **audit**, not another implementation or execution claim. It reconciles
the 41-file X86-000 baseline at
`cc35c876d3badb57df54a66e22f7768a52be95f2` with the slice inventories present on
2026-09-20. [remaining-pure.json](remaining-pure.json) records every baseline
declaration's slice references, every module, retained test scenarios, and the
specific remaining families below. Older `pending` rows in a narrow inventory
do not override a later implementation. Conversely, a `translated` row can mean
an extracted numeric component; it does not make the containing live API complete.

No language blocker is asserted for any unfinished algorithm in this audit.
There are ordinary pure algorithms left, particularly inside methods previously
classified as instruction or policy boundaries. Layout/native limitations in
existing PORT files remain separate from this implementation queue.

Subsequent completion: `gdt-storage` is now implemented and tested in
[owned GDT storage](gdt-storage.PORT.md). The missing-family table below records
the original audit findings; subsequent completions supersede those rows.
[Register recipes](register-operands.PORT.md) also complete `register-merge`,
`cr3-cr8-operands`, `xcr0-validation` and STAR/CET/APIC parts of `msr-composition`;
the remaining raw MSR word-splitting component is completed by
[MSR word transport](msr-words.PORT.md).
[Numeric topology](mapper-topology.PORT.md) completes `offset-address`,
`recursive-coordinates` and `recursive-constructor` at their numeric boundary.
[Explicit encryption state](memory-encryption.PORT.md) completes the detached
`memory-encryption` family. Default-profile captured walkers are not silently
changed into encryption-profile walkers.
[Recursive cleanup](recursive-cleanup.PORT.md) completes `recursive-cleanup`
with tested self-link exclusion, bounded resumption and full-budget handling.
[Recursive routes and translation](recursive-routes.PORT.md) complete the final
two historical families with 232 route and 90 translation cases, four additional
checks and nine body mutations. All twelve original families now have subsequent
completion records. Cross-feature composition is under fresh review; this audit
does not silently generalize default-mask walkers to encryption configurations.

## Work that already has a reusable implementation

| Family | Canonical implementation and evidence |
| --- | --- |
| Canonical48/physical52 arithmetic, explicit physical encryption-mask input | [addresses](addresses.PORT.md); raw values and checked results, not pointer capabilities |
| Pages, frames, indices, sizes, dense canonical stepping and checked ranges | [pages](pages.PORT.md), [PTE/index/level helpers](page-entries.PORT.md); iterator deviations are explicit |
| PTE words, initialized 512-entry tables, size-discovering captured translation | [entry codecs](page-entries.PORT.md), [tables and translation](tables.PORT.md); existing facts remain canonical |
| MappedPageTable map/unmap/update/set-parent/translate-page decisions and ordering | [leaf plans](mapping-plans.PORT.md), [complete captured routes](mapping-routes.PORT.md); later route inventory supersedes earlier pending method rows |
| Single branch and whole-range cleanup | [branch](cleanup-branch.PORT.md), [range cursor](cleanup-ranges.PORT.md); root-owned range evidence now records tested bounded orchestration |
| Default map, identity map, mapped-frame projection and Translate default | [mapper conveniences](mapper-conveniences.PORT.md); the explicitly named pinned offset wrapper preserves the permissive default separately from the stricter helper |
| Register identifiers/flags, selectors, debug encoding, STAR write validation | [register slice](../../drivers/facts/x86_registers.PORT.md); this does not cover every pure expression embedded in live register methods |
| Descriptor words, TSS, I/O-map checks, GDT metadata/append plan | [descriptor slice](../../drivers/facts/x86_descriptors.PORT.md); complete owned GDT mutation is still below |
| Gate/options/error/frame facts, complete detached IDT and byte codecs | [interrupt slice](../../drivers/facts/x86_interrupts.PORT.md); native layout and live handlers remain distinct |
| PCID/INVPCID/INVLPGB inputs, bounded ranges, two CPUID predicates | [TLB operands](tlb-operands.PORT.md), [observed CPUID bits](instruction-observations.PORT.md); AMD-defined and pinned range encodings are deliberately distinct |

`MapToError`, `UnmapError`, `FlagUpdateError`, `TranslateError`, private walk/create
errors and their seven `From` conversions are already expressed by `Outcome`,
`ChildPlan`, `LeafPlan` and their tested branches. Separate Rust-shaped error
wrappers would duplicate that information. `TranslateResult::InvalidFrameAddress`
cannot arise after the selected default physical mask in the generic captured
walk; arbitrary public Rust result construction is not a new reachable walk case.
`MapperFlush::page` is a numeric page projection, already represented by the
operation input. Recreating its freely constructed/discarded token is excluded.

## Concrete missing pure families

The identifiers below are stable keys in the machine-readable audit. Suggested
tests are future work, not results of this audit.

| ID | Missing work, reuse and useful next evidence |
| --- | --- |
| `offset-address` | Extract `PhysOffset::frame_to_pointer`'s **numeric** `offset + frame.start_address()` using existing checked `virtual_add` and 4 KiB frame validation. Test canonical-hole and integer-overflow boundaries against actual pinned addition. No pointer or table reference follows. The 21 size-specific methods plus Translate/CleanUp delegate to MappedPageTable and reuse existing routes/cleanup; they are not 24 independent algorithms. |
| `recursive-coordinates` | Implement private `p3_page`, `p2_page`, `p1_page`: indices `(r,r,r,p4)`, `(r,r,p4,p3)`, `(r,p4,p3,p2)`, respectively, using `pages::from_indices`. Retain source page-size restrictions. Test all recursive indices and low/high-half page indices against exact pinned bodies. Index 511 is numerically representable; the upstream end-of-address-space pointer safety warning remains an access boundary. |
| `recursive-constructor` | Extract repeated-index recognition and comparison of an explicitly observed CR3 frame with the recursive entry's `frame()` result. Preserve NotRecursive before NotActive and PRESENT/HUGE frame errors, using existing address indices and PTE classifier. Return a semantic observation result, never RecursivePageTable access. |
| `recursive-routes` | Implement captured recursive map/update/set-parent/translate-page ordering. Existing leaf codecs and unmap branches can be reused after checking exact error order. The ordinary mapped route is not equivalent: recursive allocation forces `PRESENT|WRITABLE`; an existing child is checked for HUGE but not PRESENT; update and translate-page ancestors use whole-word nonzero tests. Three sizes and all parent levels need their own malformed/non-present/huge/partial-write witnesses. |
| `recursive-translation` | Implement the recursive size-discovering walk: all levels first test whole-word zero; P4 HUGE and P1 HUGE are explicit failures corresponding to pinned panics. Intermediate non-present words can still be traversed numerically. Huge frames use containing-address truncation. Existing `translation::translate_words` tests PRESENT at parents and accepts a PT bit7 word, so it cannot be reused unchanged. |
| `recursive-cleanup` | Adapt the tested cleanup model to skip exactly the recursive P4 entry. Its nonzero self-link also participates in whole-root emptiness. Reuse range geometry, bottom-up retire ordering and full-table scans, but add root-index exclusion and witnesses proving the self-link is never traversed or retired. The existing ordinary cleanup cursor does not provide this topology policy. |
| `memory-encryption` | Supply explicit EncryptedBit/SharedBit configuration and detached state, set/query encryption flags, and profile-dependent PTE address/flag extraction. Physical-address helpers already accept a mask; PTE helpers still use the fixed default mask. Preserve or explicitly deviate from repeated configuration: upstream `PHYSICAL_ADDRESS_MASK.fetch_and(!new_bit)` accumulates cleared address bits while ENC_BIT_MASK retains only the latest bit. Test both polarities, disabled state, invalid positions, repeated configuration and raw flags; global atomics/mapping re-admission remain excluded. |
| `gdt-storage` | Implement ordinary owned-array initialization, import/copy, entries projection and append over existing GdtEntry/Descriptor plus metadata/append plans. Preserve null entry, zero-filled capacity, length and low/high word order; reject insufficient capacity before either system-descriptor write. Existing translated empty/from_raw_entries/append rows describe the extracted metadata or plan, not this missing storage layer. Test contents before and after successful and failed appends, capacities 1/2/8/8192, imports and full-table boundaries. |
| `register-merge` | Extract reserved-bit-preserving `(old & ~KNOWN) | supplied` recipes in CR0/CR4/EFER/RFLAGS/XCR0 writes. Existing known masks are canonical. Retained unknown bits in `from_bits_retain` input must not silently be truncated. Actual old-register reads/writes and update closures stay excluded. Exact source-body witnesses can test every mask and reserved bit without executing instructions. |
| `cr3-cr8-operands` | Extract CR3 raw split and frame/low-u16/no-flush composition, checked typed-PCID specialization, and CR8 observed-value conversion. `Cr3::write_raw` accepts all u16 bits, including bits12–15 that OR into the frame; do not silently constrain it to PCID. CR8 read casts raw u64 to u8 before PriorityClass::new, so raw257 selects class1. Reuse existing frame, PCID, priority and flag facts. |
| `xcr0-validation` | Extract the ordered assertions in XCr0::write: X87 required; AVX requires SSE; MPX pair complete; any AVX512 requires AVX then the complete OPMASK/ZMM_HI256/HI16_ZMM group. Flags exist, but these predicates are absent. Exhaustively test combinations of the eight relevant low bits and retained high flags against exact source logic. This is pin validation, not complete CPU support/state admission. |
| `msr-composition` | Extract STAR read selector expansion, UCet/SCet bitmap+flag split/composition and APIC-base split/reserved-merge/OR. Reuse selector/address/page/APIC facts. STAR base+8/+16 needs an explicit overflow policy. CET uses canonical `VirtAddr::new`, not truncation. APIC preserving write retains old address bits in old_flags and ORs the requested frame; test this pinned behavior rather than assuming replacement. Plain MSR u64↔two-u32 register splitting is ordinary value arithmetic, not another authority type. |

The small IF and AC predicates in `interrupts::are_enabled` and
`Smap::is_enabled` reuse existing RFLAGS bits and ordinary bit tests. A convenience
wrapper could be added, but there is no missing flag representation or complex
algorithm: IF set and AC clear, respectively. The latter does **not** inspect CR4.
Likewise zero/null/getter/equality/hash/formatting and generic bitflags operators
do not require duplicate nominal hardware representations.

## Test reconciliation and limits

The baseline lists 85 marked scenarios and four integration roots. The JSON
retains each name and points to the relevant later evidence; it does not rewrite
the baseline into an inaccurate claim that all 85 original functions execute in
Omega. Address/page/frame/index scenarios have pinned Rust and Omega boundary
evidence. Original full 1000-iteration page loops run in Rust; Omega tests selected
positions. The 34 Kani harnesses remain universal-proof infrastructure, with only
finite semantic relations/examples supplied locally. That difference remains
explicit, not an implementation blocker.

Descriptor and interrupt geometry has Rust measurements plus byte/source-plan
evidence. The subsequent owned GDT storage slice executes actual Rust and Omega storage
mutation, including full capacity and failed append preservation. IDT default
handler installation and interrupt-frame volatile mutation include live CPU/ABI
operations even though their baseline triage category was pure-or-detached.
Formatting/derive/hash tests test Rust conveniences deliberately not reproduced.
MXCSR/RFLAGS/RDRAND and bootloader/interrupt/port integration tests remain hardware
or harness scenarios, not missing pure algorithms.

The audit examined complete pinned bodies and current Omega modules, including
pure expressions embedded in excluded live methods. It used existing manifests,
source hashes and PORT evidence; it did not rerun compiler, Rust or hardware
tests. No new source or native-layout acceptance follows. Existing imported
layout visibility and TSS placement diagnostics stay documented in their owning
PORT files and do not prevent these remaining detached algorithms.

## Integration boundary

Raw address reconstruction, active CR3 reads, recursive hierarchy access,
allocator/deallocator ownership, CPU feature discovery, register I/O, table
installation, handler ABI/return, volatile live frames and invalidation settlement
remain explicit provider/custody work. A copied numeric result or completed
cursor grants none of them. Rust marker/unsafe traits, pointer conversions and
must-use tokens are deliberately not reproduced as authority facsimiles.

The original suggested sequence (topology, encryption, registers, GDT storage,
recursive algorithms) has now been implemented at each documented boundary.
A fresh closure review must distinguish remaining cross-feature composition
from live provider integration before completing the overall X86-001/002 tasks.

## Subsequent encryption-profile closure review

A fresh read-only review checked all 41 pinned source hashes and reconciled 23
slice inventories. It found no additional default-profile pure family. The
unoverlaid `gdt.rs:127:MAX` anchor is a const-generic parameter represented by
checked owned-table capacity, not a missing hardware constant.

Encryption support still changes ordinary pure compositions. These are queued
implementation and test work, with no compiler blocker:

- Mapped/recursive leaf, child and routes: profile-dependent frames, flag
  replacement, redundant writes, capture IDs and typed allocation validation.
- Generic mapped/recursive translation: profile frames and flags, preserving
  their different parent zero/PRESENT/HUGE rules.
- Mapped/recursive cleanup: profile child identities and retirement frame IDs;
  cursor geometry and self-link exclusion remain reusable.
- Physical frame PFNs, arithmetic and range selection: crossing the latest
  encryption bit can invalidate a result. Existing masked address operations
  supply the primitive arithmetic.
- Recursive constructor observations: profile-decoded PTE frame versus observed
  CR3 frame, with the existing ordered errors.
- Translate-address projection: frame plus offset can hit the configured bit,
  including an offset inside a huge frame.
- CR3/APIC register expressions: strict observed CR3 frame rejects a configured
  bit; APIC truncation removes it. Operand frame admission uses the current bit.

The accumulated PTE address mask and latest physical-address exclusion are
intentionally different after repeated configuration. Offset addition, recursive
coordinates, default-parent flags, byte codecs and owned table storage do not
need duplicate algorithms. Feature-dependent admission/delegation is sufficient.
Native layout limits, live provider authority and absent universal proofs remain
separate from this finite numeric implementation queue.

The generic mapped-translation component now passes 330 actual Rust/Omega
observations and nine body controls: [explicit-profile translation](encrypted-translation.PORT.md).
Its separate captured-path API validates profile-decoded child identities.
The other profile compositions above remain queued or under implementation.

[Encryption-profile register expressions](encrypted-registers.PORT.md) now pass
100 Rust rows, 800 Omega calls and twelve controls. CR3/APIC composition is
complete. No additional pure representation family remains; X86-001 can close
at its numeric/schema boundary while X86-002 retains the algorithm queue.
