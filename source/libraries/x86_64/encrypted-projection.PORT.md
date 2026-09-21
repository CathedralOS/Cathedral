# Explicit-profile physical address projection

Status: **tested**. Producer source check passes 16 files; all 240 Rust/Omega
projection cases across eight profiles and eight changed-body controls pass. This single helper composes the existing numeric Translation and
masked physical-address arithmetic; no new frame type or access grant is added.

Modified pure `Translate::translate_addr` behavior from x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`, under the retained
[MIT OR Apache-2.0 notices](../../../licenses/rust-osdev/x86_64/).
The pinned default method, physical addition/validation, encryption configuration
contract and existing mapper convenience were consulted before implementation.

`translated_address_pinned(profile, observation)` rejects every nonmapped case.
For a mapped case it validates the 4 KiB/2 MiB/1 GiB frame shape, then calls
`addresses::physical_add` with the latest encryption bit intersected with the
physical52 mask. Both input frame and sum must exclude that bit, and arithmetic
overflow rejects. It deliberately preserves the public trait default's arbitrary
offset, including offsets greater than or equal to frame size. The existing
strict default-profile `translation::translated_address` remains unchanged.

Even an offset inside a huge frame can reach the configured bit: with bit12
configured, a zero 2 MiB base and offset4096 reject. Repeated configuration uses
only the latest physical-address exclusion. Old bits in the accumulated PTE
mask are not silently forbidden as physical addresses. Profiles must come from
`memory_encryption::initial/configure`; forged masks and architectural admission
are outside the equivalence contract.

The reference calls the actual public `Translate::translate_addr` default on a
synthetic implementation returning initialized typed frame values. Eight isolated
processes configure before physical value construction. The retained 240 calls
include all sizes, offsets before/at/beyond size, current-bit offsets, invalid
input geometry, physical overflow, disabled/both-polarity/high-bit profiles and
repeated configuration. `catch_unwind` records both failures while constructing
the typed input inside `translate` and failures during the default's addition;
Omega turns both into explicit rejection. These are not live mapper calls.

The result-only projection normalizes the Rust nonmapped outcomes to rejection.
Omega's local invalid-virtual case represents another nonmapped projection input;
it is not claimed to be the upstream InvalidFrameAddress variant. No fabricated
error payload or physical authority is introduced for that correspondence.

The [inventory](encrypted-projection-inventory.json) retains the complete mapper
trait source and marks only the extracted default projection component. Numeric
observations prove no actual mapping, usable physical address, ownership or native
ABI. The default-profile helpers retain their earlier evidence unchanged.

Reproduce with `python3 tools/ports/x86_64-encrypted-projection/check.py`.
`--host-only` checks fresh Rust observations and fixture generation. Compiler:
clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

The [verification record](../../../tools/ports/x86_64-encrypted-projection/verification.json)
binds tested source/fixture hashes and exact reference output. `verify_record.py`
checks that correspondence without claiming a fresh semantic run.
