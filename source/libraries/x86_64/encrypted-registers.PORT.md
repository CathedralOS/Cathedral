# Explicit-profile CR3 and APIC expressions

Status: **tested**. The 20-file producer source check, 100 Rust reference rows,
800 Omega calls in ten profile fixtures, and twelve body-mutating controls pass. These seven helpers compose existing register operand types and recipes
with explicit memory-encryption configuration. They add no register access.

Modified pure expressions come from x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`, `registers/control.rs` and
`registers/model_specific.rs`. Retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/) apply. The exact
CR3/APIC read/write bodies, PhysAddr feature implementations, encryption
configuration contract and Cathedral library charter were reviewed first.

CR3's observed frame uses strict `PhysAddr::new` after the fixed bits12–51
extraction. A configured bit inside that frame returns `ObservedCr3::Rejected`,
replacing the pinned panic. Successful observations reuse `Cr3Observation` with
its unchanged low-u16 projection. No new register representation is introduced.
The enclosing result case only expresses the added input failure.

APIC's observed frame uses `PhysAddr::new_truncate`: it clears the latest
configured physical bit before taking the containing 4 KiB frame. All original
raw flags remain retained and the typed known flags are unchanged. This uses
the latest bit, not the PTE mask accumulated across past configurations. An old
PTE-only excluded bit remains valid in an APIC physical frame.

Raw, flags and PCID CR3 operands and raw/preserving APIC operands validate the
supplied physical frame against the current bit before calling existing recipes.
Raw low-u16 and flag OR behavior remains exact: supplied flags may introduce
address bits after typed-frame admission, as the pin does. Reserved APIC bits
including old address bits remain preserved by OR. No new mask sanitizes them.
PCID domains and frame alignment remain the existing checked input contracts.

Profiles must arise from `memory_encryption::initial/configure`; editable forged
records are outside pinned equivalence. These are detached supplied observations
and proposed operand words. CPU support, the configured bit's architectural
appropriateness, live CR3/MSR access, backing and invalidation are not established.
The original register helpers keep their default-profile interfaces.

The reference executes exact pinned pure expressions using actual public
PhysAddr/PhysFrame/flag APIs with `memory_encryption` enabled. It does **not** call
CR3 or MSR read/write methods. Source-fragment checks bind every relevant
expression; the inventory binds both complete pinned files. Ten isolated
processes configure before constructing physical values. Each supplies ten raw
observations/frame candidates, yielding 100 rows and eight Omega calls per row.
Cases include disabled state, both polarities, low/high bits, repeated
configuration, all-bit/misaligned inputs, no-flush and retained unknown flags.

Compiler is clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
Native Omega artifacts, actual register instructions and hardware are not run.
The [inventory](encrypted-registers-inventory.json) marks nine extracted
expression components across 161 anchors; other components retain their own
earlier inventory dispositions.

Run `python3 tools/ports/x86_64-encrypted-registers/check.py` for the complete
reference/semantic/control suite; `--host-only` checks fresh retained Rust values
and fixture generation. Ordinary implementation remains for the other encrypted
mapper/frame compositions in the parent closure audit.

The [verification record](../../../tools/ports/x86_64-encrypted-registers/verification.json)
binds current source/fixture hashes and retained reference output. Use
`verify_record.py` to check correspondence without claiming a fresh execution.
