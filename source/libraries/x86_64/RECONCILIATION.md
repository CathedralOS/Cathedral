# x86_64 collisions, authority and implementation queue

The [complete module table](MODULES.md) and [source inventory](inventory.json)
cover all 41 Rust files at `cc35c876d3badb57df54a66e22f7768a52be95f2`,
including optional features and the separate testing crate. This document
records decisions that a flat declaration index cannot express.

## Existing representations remain canonical

| Upstream surface | Existing Cathedral surface | Reuse boundary |
| --- | --- | --- |
| PageTableEntry, flags, address extraction | [X86PageTableEntry](../../drivers/facts/x86_page_table_entry.omg) | Complete 64-bit field schema and explicit 8-byte plan exist. Extend pure codecs over this type; do not add a second PTE record. Existing plan syntax still needs its own fresh consumer check before any new layout claim. |
| PageTable | [X86PageTablePageCandidate](../../core/x86_page_table.omg) | Detached 512-entry candidate exists. Generic library algorithms cannot depend on core; factor reusable pure work downward and keep core policy/admission upward. No direct-map table reference is established. |
| VirtAddr, PhysAddr and page/frame indices | [bootstrap address/PFN validators](../../core/x86_page_table.omg) | Core already validates canonical48-bit addresses, four indices, 4KiB alignment and52-bit physical envelope. These are target-selected candidates, not the entire general arithmetic/range API. Preserve the original contracts when extracting common helpers. |
| Entry<F>, InterruptDescriptorTable | [X86IdtGate](../../drivers/facts/x86_idt_gate.omg) | Gate schema fragments one entry identity into hardware fields. Upstream Rust Entry is repr(C); Cathedral's selected plan requests alignment16. Do not assert identical native alignment from identical byte size; measure both target policies in a later slice. |
| ExceptionVector and named IDT exception fields | [exception facts](../../drivers/facts/x86_exception_vectors.omg) | Numeric exceptions and delivery categories exist. AMD slots28–30 stay CPU-profile-dependent; generic enum values cannot silently select Cathedral delivery policy. |
| TSS stack slots and gate IST option | [IST facts](../../drivers/facts/x86_interrupt_stacks.omg), [interrupt profile](../../core/x86_interrupt_profile.omg) | Reuse hardware-index/analysis-class coupling. Complete TSS carrier/descriptor encoding is still missing; bytes do not establish backing, live stack, CPU root or loading authority. |
| ApicBase::MSR, ApicBaseFlags | [local APIC facts](../../drivers/facts/local_apic.omg) | Reuse IA32_APIC_BASE and BSP/x2APIC/global-enable bit facts. Existing timer MSRs extend beyond upstream ApicBase and need no duplicate aliases masquerading as new facts. |
| Msr read/write and port wrappers | [x2APIC](../../core/x2apic_timer.omg), [PIC](../../core/pic_8259.omg) | Existing checked wrmsr/out leaves retain MachineControl/PortIo. Their narrow timer operations are precedents, not generic register/port authority. |

## Important source distinctions

- `VirtAddr` at this pin enforces 48-bit sign extension; it is not a generic
  LA57 address type. Core's four-level bootstrap profile agrees by explicit
  choice. Address stepping can cross the canonical hole, while ordinary raw
  arithmetic can overflow or panic; tests distinguish those operations.
- `PhysAddr` limits bits52–63. With `memory_encryption`, truncation also clears
  the globally configured encryption bit. Port the explicit configuration and
  transformation separately; do not silently mutate the meaning of all address
  values when a global flag changes.
- PTE bit7 is named HUGE_PAGE upstream, while Cathedral preserves
  `page_size_or_pat` because PT leaves use PAT there. Upstream retains bits59–62
  as generic BIT flags; Cathedral gives them a bounded `protection_key` field.
  Role and CPU-feature checks remain policy, not consequences of bit decoding.
- `DescriptorTablePointer` is repr(C,packed(2)), with a 16-bit limit and 64-bit
  base. TSS is repr(C,packed(4)) and upstream asserts size104. These are useful
  future target vectors; this audit has not measured Omega storage for them.
- `SegmentSelector::new` shifts a u16 index by3 and inserts RPL; an eventual
  checked constructor must explicitly choose rejection versus upstream
  truncation for oversized indices. Selector table-index bit2 and null-selector
  rules must not be erased by a GDT-only convenience API.
- TSS descriptor helpers mix pure bit composition with borrowed pointer
  lifetime and I/O bitmap byte checks. Keep the encoder pure; any dereference or
  persistent descriptor use needs admitted storage and lifetime evidence.
- `Star::write` checks SYSRET/SYSCALL selector spacing and privilege before its
  MSR write. Extract those pure checks and their error categories; success does
  not authorize WRMSR or establish valid loaded descriptor tables.
- Rust `PageTableEntry::is_unused` means the entire word equals zero, not only
  PRESENT clear. Cathedral's empty-page validator also checks every field;
  non-present metadata cannot pass as an untouched page.

## Rust unsafe and authority decisions

| Upstream invariant | Cathedral treatment |
| --- | --- |
| Canonical/aligned numeric constructors and unchecked PFNs | Ordinary pure validation/domain obligation; no reason to block on hardware authority. |
| Raw VirtAddr pointer conversion | Preserve numeric identity only. Dereference requires qualified Extent/placed access, bounds, alignment, initialization and provenance. |
| FrameAllocator uniqueness and FrameDeallocator unused-frame promise | Replace ambient unsafe trait ownership with admitted qualified backing/allocation lifecycle. Copying PhysFrame creates no frame authority. |
| PageTableFrameMapping arbitrary mutable pointer | Deliberately reject as an access grant. Detached address arithmetic can port; live table traversal needs hierarchy ownership and alias/lifetime proof. |
| OffsetPageTable/RecursivePageTable | Keep mapping topology as Cathedral policy; no raw address reconstruction becomes an ordinary mutable reference. Recursive index511 has a documented upstream pointer-end safety caveat. |
| MapperFlush/MapperFlushAll plus ignore() | A must_use warning and public discard are not linear invalidation settlement. Keep numeric flush intent separate from exact mapping-era/CPU completion obligations. |
| Interrupt enable/disable closures, SMAP bypass closure | Preserve exact prior-state and nested restoration through admitted guards; no unrestricted safe closure wrapper confers control. |
| HandlerFunc, set_handler_fn, general-handler macros, iretq | Use installed entry identity and admitted root/return plans. Rust ABI function pointers and mutable interrupt frames do not replace Cathedral entry contracts. |
| Register read/write/update and feature probes | Checked instruction contracts, target features, reserved bits, CPU state and explicit provider authority. Pure flags or a zero-sized Rust type grant none. |
| enable_memory_encryption global atomics | Explicit profile/configuration input plus mapping transition policy; no hidden global redefinition of address semantics. |

The relevant Omega contracts are
[checked assembly](../../../../Omega/wiki/spec/language/assembly.md),
[authority](../../../../Omega/wiki/spec/resources/authority.md),
[placed access](../../../../Omega/wiki/spec/resources/placed_access.md),
[device custody](../../../../Omega/wiki/spec/resources/device_access.md), and
[layout plans](../../../../Omega/wiki/spec/layouts/plans.md). Their specified
contracts do not themselves prove every current compiler/native consumer exists.

## Bounded implementation order

1. Register flags/identifiers, privilege/selector/PCID/debug encodings, MXCSR
   default and pure STAR/Dr7 transformations. Reuse APIC constants. None needs
   live register access; primary value vectors and mutated-test negative
   controls can establish useful progress independently.
2. Descriptor-pointer/TSS/GDT fixed records and explicit layout policies,
   descriptor/selector/error-code codecs and I/O bitmap validation on admitted
   byte inputs. Reuse IDT gate and IST/exception facts; preserve IDT source notice.
3. Generic numeric address, page and frame geometry, checked alignment and
   overflow/canonical-gap stepping; extract/reuse core's proven subset without
   introducing a library dependency on core or a competing bootstrap policy.
4. Pure PTE word codecs and detached page-table algorithms over the existing
   schema. Separate 4KiB/2MiB/1GiB role constraints and feature-dependent bits;
   extend or reuse existing validators rather than adding parallel types.
5. Extract detached mapping/translation/cleanup plans. Live frame allocation,
   table access, root publication and cross-CPU invalidation remain explicit
   integration work. Their size is not a language blocker; exact missing
   compiler mechanisms must be recorded only where established.

`tests.json` retains 85 marked upstream scenarios plus four integration entry roots: 44 pure/detached cases, 34 Kani
proof harnesses and seven hardware-integration-marked tests. The separate Rust
bootloader/spin/lazy_static runner is deliberately not copied. Inline IDT tests
that contain a hardware operation need further case-level separation before
execution; their current pure/detached label is a source triage category, not
permission to execute the entire function on a host. No upstream or translated
x86 test was executed by the inventory check.
