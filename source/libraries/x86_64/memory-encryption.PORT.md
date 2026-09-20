# Explicit memory-encryption state and entry recipes

Status: **tested**. The 13-file producer source check, all 133 Omega profiles
covering 1,315 Rust-observed inputs, the additional reconfiguration/malformed
fixture and four body mutations pass. This X86-001/002 slice translates the pure arithmetic in
`structures/mem_encrypt.rs` and the encryption-aware parts of `addr.rs` and
`structures/paging/page_table.rs` from x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`, under the retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/).
The exact pinned bodies, Omega's data/case and borrowing contracts, existing
physical/PTE helpers and the library authority charter were reviewed first.

## Explicit state

`Configuration` has EncryptedBit and SharedBit cases. `State` is an ordinary
copyable simulation record containing the address mask, current bit mask and
polarity. `initial` reproduces the disabled state. `configure` accepts shift
positions 0..63, including positions which are inappropriate for actual CPUs;
this is the pin's representable arithmetic envelope, not architectural admission.
Positions >=64 reject without changing any field, consistently replacing a
panic-prone or build-dependent Rust shift. No global atomic is read or written.

Repeated configuration preserves a subtle pinned rule: the PTE address mask
accumulates cleared bits using AND-NOT, while the current encryption mask is
replaced. For example, EncryptedBit47 then SharedBit48 removes both positions
from PTE addresses but only position48 controls encryption. Configuring bit47
again does not restore bit48 to the PTE mask. There is no invented reset/disable
operation. `initial` creates a new detached simulation, not a hardware reset.

`encryption_flag` returns the current bit or Rejected when disabled.
`set_encrypted` preserves all other supplied raw flags and sets/clears the bit
according to polarity; disabled state rejects. `is_encrypted` returns false
when disabled and otherwise applies the same polarity. Raw unknown flags are
retained just as with upstream `from_bits_retain`.

The PTE address, flags, frame classification and flag-replacement helpers consume
this explicit state. PRESENT is tested before HUGE, and extracted physical bits
remain separate from flags. Setter helpers validate a physical52 aligned input
and then preserve the pin's raw OR behavior, including flag bits that contaminate
address positions. The canonical PTE field schema and byte representation are
unchanged; these are numeric interpretations of the same word under a profile.
Existing default-profile mapper/translation modules remain default-profile APIs;
this slice does not silently make all captured walkers encryption-aware.

`physical_check` and `physical_truncate` reuse the existing physical52 helpers.
They intersect the active mask with the physical52 envelope first: an encryption
bit above bit51 cannot occur in a valid physical52 value and therefore imposes no
extra physical-address restriction. The accumulated PTE address mask and latest
physical-address exclusion intentionally differ after reconfiguration.

The public State record is editable data. Pinned equivalence describes states
produced by initial followed by successful configure calls. Supplying a forged
mask is a caller-authored arithmetic profile, not evidence of valid architecture
or the pinned global-state invariant. Results do not authenticate observations,
validate CPU encryption support, select an actual C/S bit, adapt existing page
tables, grant backing or establish invalidation/alias obligations. Any live
provider must resolve those obligations independently before reconfiguration.

## Reference evidence and limits

The Rust harness calls actual pinned public APIs with the memory_encryption
feature enabled. Each of 133 profiles runs in a fresh process: disabled, both
polarities at all 64 representable positions, and four repeated-configuration
sequences. Configuration occurs before any PTE/address construction. These
isolated arithmetic probes never install page tables, access physical memory,
change CPU configuration or preserve a live table across reconfiguration.

The 1,315 observed raw words exercise encryption query/set, PTE address/flags,
frame classification, flag replacement, physical check/truncation and checked
address/frame setting. Expectations come from those actual calls, including
caught disabled-set, address-construction and alignment panics. Fresh processes
avoid accidental test-order dependence from upstream global atomics. Omega
fixtures call the actual authored helper bodies against the same observations;
source/vector reproduction alone is not a semantic test.

An additional actual Rust reconfiguration test and Omega fixture establish that
a previous encryption bit becomes admissible in physical addresses while still
being excluded from PTE address extraction. They also check ordinary nonzero
aligned address setting. Extra cases cover disabled flag lookup, invalid
shifts64/255 with complete state preservation, and set_frame's HUGE rejection. Four body controls alter
encryption setting, SharedBit polarity, repeated-configuration address extraction
and invalid-shift preservation. Native Omega code, CPU instructions and production
integration are outside this evidence.

The [inventory](memory-encryption-inventory.json) retains three whole source
files and maps only the encryption components. The existing physical and PTE
slices remain canonical for their default-profile behavior. No language or design
blocker is asserted for future explicit-profile captured-walk composition.

Compiler: Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

```sh
python3 tools/ports/x86_64-memory-encryption/check.py --omega /path/to/omega
python3 tools/ports/x86_64-memory-encryption/check.py --host-only
```
