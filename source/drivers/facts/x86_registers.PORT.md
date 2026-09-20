# x86 register encodings and pure transformations

## Scope and status

Tested bounded X86-001/002 slice: CR0/3/4, DR6/7, EFER/CET/APIC-base,
MXCSR, RFLAGS and XCR0 numeric encodings; MSR identifiers; privilege/selector,
CR8 priority and debug-register codes; detached DR7 mutation and STAR selector
validation/packing. There are 19 raw fact carriers, 177 constants, two result
carriers and 31 pure machines. Source and semantic checks passed with Omega
`eaa7993a23623cd8fabf45350340479c5c9c7879`. No native execution, aggregate ABI,
CPU feature discovery or live register instruction is claimed.

## Upstream pin and licensing

[x86_64 0.15.5](https://github.com/rust-osdev/x86_64) at
`cc35c876d3badb57df54a66e22f7768a52be95f2`.
Derivative files preserve MIT OR Apache-2.0 and identify modifications.
[Full source map/provenance](../../libraries/x86_64/PORT.md),
[licenses and notices](../../../licenses/rust-osdev/x86_64),
[root notice](../../../THIRD_PARTY_NOTICES.md).

## Source and public-symbol map

[x86_registers-inventory.json](x86_registers-inventory.json) binds eight source
files and 345 lexical anchors to the exact revision: 203 translated, 142
explicitly omitted, zero pending or blocked within this slice. Its supplement
maps all 31 numeric enum variants; [schema.json](../../../tools/ports/x86_64-registers/schema.json)
records every flag/MSR declaration and source line. Omitted methods include live
instructions, Rust traits/formatting and root modules outside this slice; the
full-crate baseline retains their wider dispositions.

| Pinned source | Translated facts or pure work |
| --- | --- |
| `src/lib.rs` | Four privilege codes and checked conversion |
| `src/registers/control.rs` | CR0/3/4 flags; 15 CR8 priority codes and validation |
| `src/registers/debug.rs` | DR6/7 flags, register/condition/size codes, raw `Dr7Bits`, field and flag transformations |
| `src/registers/model_specific.rs` | Ten MSR identities, EFER/CET/APIC flags, `Msr`, extracted STAR fields and selector validation |
| `src/registers/mxcsr.rs` | MXCSR flags and actual pure `Default` value |
| `src/registers/rflags.rs` | Complete pinned RFLAGS flag family |
| `src/registers/segmentation.rs` | Raw selector, bounded construction, index/RPL projections and RPL replacement |
| `src/registers/xcontrol.rs` | Complete pinned XCR0 flag family |

The facts consist of 122 flag constants, 11 known-bit masks, 31 code constants,
ten MSR identities, null selector, MXCSR reset bits and the DR7 raw-value mask.
IA32_APIC_BASE identity and three flag positions reuse `local_apic.omg`.

## Primary specifications and representation vectors

[Intel SDM](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
Volumes 1, 3 and 4 supply the architectural context: RFLAGS/MXCSR, control and
debug registers, selectors, system calls and model-specific registers.
[AMD APM Volume 2, revision 3.44](https://docs.amd.com/v/u/en-US/24593_3.44_APM_Vol2)
covers system-register and system-call state. These are the primary references;
the machine-readable numeric oracle is the exact pinned Rust source, not a
claim that every encoding was independently rederived from both manuals.

[x86_registers.vectors.json](x86_registers.vectors.json) contains 177 actual
upstream Rust value observations. Of these, 176 const-compatible values are
also asserted for `x86_64-unknown-uefi`; the pure MXCSR `Default` body is executed
on the host. The ten private single-field Rust MSR carriers are inspected with
test-only `transmute`; Cathedral does not adopt their memory representation.
Omega semantic fixtures compare all 177 actual fact bindings. This is numeric
agreement, not native layout or calling-convention evidence.

## Translated tests and fixtures

[Fixture tooling](../../../tools/ports/x86_64-registers/README.md) executes the
actual translated machines through `const TEST_RESULT = test_result()` and a
required `TEST_RESULT == 0` contract. The positive fixture covers all 31 helpers,
177 fact comparisons, 64 combinations of debug-register index/condition/size
with exact preservation of other bits, selector boundaries, flag mutations,
invalid codes, STAR success, four upstream errors and the local underflow error.
Three controls change expected values inside the executed test bodies: DR7 size,
STAR underflow and MXCSR reset value. Each must evaluate to one and fail the
unchanged final contract with `1 == 0`.

Three Rust tests exercise pinned pure methods, including all 256 input bytes for
code validation and the 64 DR7 combinations. The pinned STAR `write` body is
extracted unchanged with only its `write_raw` leaf replaced by a recording stub;
this tests original validation without executing WRMSR. A regression test
observes its underflow. Upstream's `mxcsr_default` test actually calls `read()`;
all three upstream MXCSR tests remain hardware-dependent and are omitted. The
pure `Default` constructor has an authored executable observation instead.

## Omega blockers and boundary seams

No blocker remains for this bounded pure slice. A retained minimal pair isolates
an avoidable Omega copy-resolution limitation: qualified local type and
constructor `x86_registers::SegmentSelector` yields `affine value 'value' was
already transferred or consumed` on the second by-value helper call; direct
`use facts::x86_registers::SegmentSelector` and unqualified local names pass.
See `copy_probe.omg` and `copy_direct_probe.omg` in the fixture directory.
Tests use the passing direct-import form.

Ordinary helper calls nested in named-state terminal expressions also triggered
`source-less constant target is not its exact owning entry` / contradictory
retained-source-custody diagnostics. Explicit locals or direct pure expressions
avoid that source form; the final producer and tests pass. Neither limitation
is used to classify this finished slice as blocked.

Raw carriers permit unknown bits and are not CPU configuration proofs.
`DR7_VALID_BITS` preserves the upstream detached `Dr7Value` mask: in particular,
bit 10 is excluded even though the upstream live writer separately sets its
hardware-required value. Feature compatibility, live reserved-bit handling,
instruction admission and MachineControl authority belong to instruction/policy
work. No GDT, IDT, PTE or core representation is duplicated here.

## Deliberate deviations

Rust panicking conversions become explicit validity/error data where useful.
`Dr7Value` becomes visibly raw `Dr7Bits`, with a separate validity predicate;
its public constructor does not establish the invariant of the Rust type.
`selector_new` accepts only a complete 13-bit index and two-bit RPL; upstream's
oversized-index truncation is rejected through bounded inputs and the companion
predicate. Detached transformations preserve supplied bits according to their
numeric operation and confer no authority.

STAR validation is separated from WRMSR. The pinned body accepts selectors
`(11, 3, 8, 16)` through its widened offset/RPL comparisons, then subtracts eight
from the `u16` value three: debug Rust panics, while unchecked release arithmetic
wraps. `star_plan` returns local error 5 before forming that payload. Errors 1–4
preserve upstream check order. Tests execute both the unchanged upstream body
with a recording leaf and the corrected Omega body.

## Cathedral integration and authority

Facts live in [x86_registers.omg](x86_registers.omg); algorithms in
[registers.omg](../../libraries/x86_64/registers.omg), under the shared
`cathedral-x86-values` package. Neither imports core or grants machine authority.
The new modules remain available for future integration; no production root or
instruction provider is added.

## Verification commands and results

From repository root:

```sh
python3 tools/ports/x86_64-registers/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega
```

The runner verifies generated source/inventory freshness, pinned source hashes,
177 Rust observations, 176 UEFI-x64 assertions, three Rust tests, standalone Omega
producer checking, positive semantic checking and three body mutations. The
compiler SHA-256 is
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
Standalone producer: 12 sources; positive fixture: 14 sources. The retained copy
probes are separate diagnostic evidence, not expected failures in the canonical
runner. No native execution or aggregate ABI comparison is claimed.
