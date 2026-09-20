# x86_64 source reconciliation

## Scope and status

X86-000 inventories every Rust module in pinned `src/` and the separate
`testing/` package. Its baseline stage is **inventoried**. The baseline artifacts
record initial source triage; subsequent implementation and execution evidence
belong to separately audited slices. No baseline count is a native ABI or
production-integration claim.

Completed slices: [tested numeric address algorithms](addresses.PORT.md),
[page/frame geometry and ranges](pages.PORT.md),
[tested register facts and transformations](../../drivers/facts/x86_registers.PORT.md),
[descriptor cases and TSS codecs](../../drivers/facts/x86_descriptors.PORT.md),
[PTE word/index/level operations](page-entries.PORT.md),
[detached tables and captured translation](tables.PORT.md),
[child/leaf mapping decisions](mapping-plans.PORT.md),
[interrupt values and table codecs](../../drivers/facts/x86_interrupts.PORT.md),
[TLB operand recipes](tlb-operands.PORT.md), and
[single-page cleanup branches](cleanup-branch.PORT.md).
Baseline reviewed 2026-09-20 against
Cathedral `77544518fc947973a5607f28b278aca2e2ffa462` and
Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`. X86-001/002 are implementation work,
not language blockers merely because their code has not yet been authored.

## Upstream pin and licensing

[x86_64 0.15.5](https://github.com/rust-osdev/x86_64) at
`cc35c876d3badb57df54a66e22f7768a52be95f2`; checkout
`reference_code/rust-osdev/x86_64`. Preserve MIT OR Apache-2.0:
[MIT](../../../licenses/rust-osdev/x86_64/LICENSE-MIT),
[Apache](../../../licenses/rust-osdev/x86_64/LICENSE-APACHE),
[AUTHORS](../../../licenses/rust-osdev/x86_64/AUTHORS),
[source notices](../../../licenses/rust-osdev/x86_64/SOURCE-NOTICES.md), and
[root attribution](../../../THIRD_PARTY_NOTICES.md). This audit is Cathedral
work; subsequent derivative Omega files must identify their exact source mapping.

## Source and public-symbol map

[MODULES.md](MODULES.md) and [module-map.json](module-map.json) classify all
41 Rust files (33 library, eight test-package files;13,316 source lines).
[inventory.json](inventory.json) binds 1,307 lexical declarations/anchors to
exact file hashes; [lexical-supplement.json](lexical-supplement.json) adds124
named enum variants, payload fields and macro definitions. Macro expansion is
not claimed; complete source hashes cover every unexpanded branch/private field.
One format-string false anchor is explicitly identified as non-code.

Classifications distinguish existing representation, pure facts/layouts, pure
algorithms, instruction boundaries, Cathedral policy, deliberate rejection and
test fixtures. Baseline inventory dispositions are0 translated,205 omitted
(including reuse/scaffolding),5 confirmed compiler-blocked and1,097 pending
implementation. **At baseline, pending means reviewed future work, not an unclassified
module. Subsequent slice inventories supersede those implementation statuses.** Every row has a classification and reason. The checker reproduces
all artifacts and validates comparison anchors; a missing file/row fails.

[RECONCILIATION.md](RECONCILIATION.md) maps collisions, unsafe invariants and
bounded implementation order. Existing PTE, IDT gate, exception/IST facts and
bootstrap walk validators remain canonical; their presence does not imply
complete upstream API coverage. Pure addresses confer no mapping/access grant.

## Primary specifications and representation vectors

Consult Intel's [SDM publication index](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html):
Volume3A chapters3–4 for address/segment/page geometry, chapter6 for interrupt
structures, chapter7 for TSS, and Volume4 for MSRs. Consult
[AMD APM Volume2, revision3.44](https://docs.amd.com/v/u/en-US/24593_3.44_APM_Vol2)
for AMD-specific system features, INVLPGB/TLBSYNC and memory-encryption context.
This audit does not assert all upstream constants have already been verified
against those manuals. Exact representation vectors belong to each subsequent
bounded translation slice. Existing facts cite their own specification editions.

## Translated tests and fixtures

No upstream x86_64 implementation tests translated or executed in X86-000.
[tests.json](tests.json) retains85 marked test/Kani scenarios and four separate
integration entry roots. Source triage identifies44 pure/detached cases,34 Kani
proofs and seven hardware-related marked tests; inline mixed cases still require
statement-level separation before execution. Existing Cathedral canaries are
comparison targets, not fresh passing evidence. The three bounded compiler
catalog probes below exercise the compiler only and perform no hardware I/O.

## Omega blockers and boundary seams

Numeric validation, flags, fixed records and explicit layout policies remain
ordinary translation work. Two live-operation limitations were independently
confirmed with the freshly built exact Omega revision:

| Blocker | Affected upstream | Exact observed source diagnostic | Independent work |
| --- | --- | --- | --- |
| `omega:x86-port-widths` | instructions/port.rs u16/u32 read/write primitives | `in` destination requires an exact `u8` writable place for `al`, found `u16`. The otherwise equivalent u8 fixture passes. | Numeric ports/access intentions, all pure values, existing u8 PortIo adapters. |
| `omega:x86-tlb-instructions` | instructions/tlb.rs::flush | `unknown asm instruction invlpg`; no known contract in current catalog. | Detached TLB commands, PCID/range validation, mapping recipes. |

[Catalog probes](../../../tools/ports/x86_64/catalog-probes) carry source markers
and reproduce the diagnostics. These block those live operations, not X86-000
or the missing pure corpus. The current
[instruction catalog](../../../../Omega/omega-rust/psi/foundation/language-core/src/inline_assembly/mod.rs)
and its [implementation notes](../../../../Omega/omega-rust/psi/foundation/language-core/inline_assembly.md)
also distinguish deriver-only descriptor/return instructions from ordinary
source operations. Other absent upstream instruction families require their
own exact contracts and checks; catalog presence never establishes native
realization or provider authority.

PortIo/MachineControl and qualified Extent/placed access own live operations.
Unsafe pointers, constructors, flush-ignore methods and global mutable memory
encryption state must not be mistaken for portable authority models.

## Deliberate deviations

Do not duplicate existing PTE/IDT schemas. Do not import Rust Port ownership,
ambient frame allocators, arbitrary frame-to-pointer casts, Rust interrupt ABI
handler pointers, or freely dismissible flush promises as Cathedral authorities.
Numeric plans and licensed algorithms remain useful separately.

## Cathedral integration and authority

The [library charter](../CHARTER.md), [driver charter](../../drivers/CHARTER.md),
[core charter](../../core/CHARTER.md), and [porting policy](../../../wiki/architecture/prior_art_and_hardware_facts.md)
own the split. New pure reusable algorithms must not depend on core. Existing
bootstrap CPU selection, page-table placement, interrupt roots and admitted
instruction services remain Cathedral policy.

## Verification commands and results

Run from Cathedral root; the source map requires the exact upstream checkout.

| Command | Result and exact scope |
| --- | --- |
| `python3 tools/ports/x86_64/inventory.py` | All41 files,1,307 source anchors,124 supplements,85 marked scenarios and4 integration roots reproduce exactly; local comparison targets exist. |
| `python3 tools/ports/inventory.py check source/libraries/x86_64/inventory.json --checkout reference_code/rust-osdev/x86_64` | Shared schema/pin/hash/target checks pass;0 translated/205 omitted/5 blocked/1,097 pending implementation. |
| `python3 tools/ports/x86_64/check_catalog.py /tmp/cathedral-omega-eaa7993/release/omega` | u8 input fixture source-checks11 files; u16 input and invlpg fixtures reject with the exact catalog diagnostics above. No hardware instructions executed. |
| Upstream or translated x86 implementation tests | Not run; retained scenarios only. |
| Native layouts, firmware, hardware, whole production root | Not run; no claims from an inventory or expected rejection. |

The named binary was freshly built from Omega eaa7993; catalog runner accepts
another exact compiler path as its optional positional argument. No Omega
source was changed by this audit.
