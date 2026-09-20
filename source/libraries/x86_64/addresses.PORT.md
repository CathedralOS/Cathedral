# Pure x86 address arithmetic — X86-001/002 slice

Status: **tested** by Omega semantic evaluation and actual pinned Rust witnesses.
Scope is the complete numeric algorithm surface of `src/addr.rs` in
[x86_64](https://github.com/rust-osdev/x86_64/tree/cc35c876d3badb57df54a66e22f7768a52be95f2),
revision `cc35c876d3badb57df54a66e22f7768a52be95f2`. Modified derivative code retains
MIT OR Apache-2.0; see [notices](../../../THIRD_PARTY_NOTICES.md) and
[retained licenses](../../../licenses/rust-osdev/x86_64/).
Page/frame ranges and page-table algorithms remain subsequent slices.

## Ownership and representation

`addresses.omg` operates on explicit u64 integers and returns `NumberResult`
cases. It does not introduce another bootstrap-address, PTE, frame, or mapping
type. Cathedral's existing core decomposition and physical-frame validators
remain the owners of their target-selected candidates and contracts. This
general arithmetic package cannot depend on core; future adoption must preserve
those contracts. No production root imports the new module.

The virtual profile deliberately matches the pin's 48-bit sign extension and
four page-table index levels. It is not LA57 or LAM handling. The physical
profile is the pin's 52-bit numeric envelope, not a discovered processor's
MAXPHYADDR or a usable-memory claim. Intel's
[SDM Volume 3A](https://cdrdv2-public.intel.com/874249/253668-090-sdm-vol-3a.pdf),
chapters 4–5, distinguishes canonicality and paging modes. The existing core
facts and full [reconciliation](RECONCILIATION.md) document Cathedral's selected
bootstrap profile. Omega data/case, dependent-value, authority and layout
contracts were consulted before implementation.

## Preserved behavior and explicit changes

- Canonical checking rejects the hole; truncation sign-extends bit 47.
  Ordinary addition/subtraction rejects noncanonical results and overflow.
  Stepping uses a dense 48-bit ordinal and skips the hole. Distance counts only
  canonical positions. Overflowing-step results preserve the original value.
- Alignment rejects zero/non-power-of-two alignments and integer overflow.
  Virtual alignment preserves upstream's subsequent sign-extension behavior:
  aligning the last low address up by two yields the first high address;
  aligning the first high address down to 2^48 yields zero. This is deliberate
  compatibility behavior, not an alignment policy for admitted memory.
- Physical checking/truncation, arithmetic and alignment take an explicit
  encryption mask. Zero disables it; a nonzero mask must be one bit inside the
  52-bit envelope. The pin's hidden global atomic mask becomes an explicit
  argument. Mask validation is an added checked policy, not a CPUID fact or a
  memory-encryption transition implementation.
- Panic/unchecked construction becomes an explicit rejected result. Inputs
  remain with the caller; rejection does not duplicate the raw value in a Rust
  error-wrapper equivalent. Each address-specific helper validates ordinary
  numeric inputs. Generic checked integer addition/subtraction accepts u64 and
  establishes only arithmetic success.
- Rust nominal wrappers, formatting, trait operators, zero/null/identity
  conveniences and unsafe pointer conversions are not recreated. A returned
  numeric value is never a readable/writable pointer, physical frame grant,
  page-table reference or allocation capability.
- All operations and counts use explicit u64. The pin's 32-bit `usize`
  saturation/reporting branch is outside this UEFI-x64 numeric profile.

## Inventory and tests

[addresses-inventory.json](addresses-inventory.json) audits the entire pinned
file: 106 lexical anchors, 64 translated and 42 deliberate omissions, zero
pending/blocked. Every source byte is hash-bound. This is a lexical audit, not
macro expansion or proof that a Rust trait API has been reproduced.

The deterministic [fixture generator](../../../tools/ports/x86_64-addresses/generate.py)
retains **110 numeric cases**: canonical boundaries, truncation, physical
limits, alignment, malformed inputs, arithmetic overflow/underflow, all pinned
forward/backward/distance examples, indices and offsets, and added explicit-mask
checks. **102 cases execute the actual pinned Rust crate**, with panics caught
as rejected outcomes. The remaining eight are added input-validation policies.
Rust uses `nightly-2026-09-04`: the older installed January nightly lacks the
pin's `Step::forward_overflowing/backward_overflowing` methods. No pinned source
is patched to accommodate that toolchain mismatch.

`extras.omg` includes all five upstream overflowing-step assertions, alignment
predicates, and four sets of finite boundary instances for the six Kani
relations (zero step, composition, forward/backward reversal and distance).
Kani's universal proof infrastructure is explicitly omitted; finite semantic
tests are not claimed as universal proofs. The upstream pointer-array identity
test is deliberately omitted with pointer conversion.

All fixture bodies execute as constant initializers in fresh Omega
`eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
Three independent controls change actual expected behavior (zero address,
gap-step value, overflow flag); each must yield result 1 and fail the unchanged
success assertion. Source/typechecking alone is not the claimed test.

Some helper calls inside named-state transition edges triggered the compiler's
`source-less constant target is not its exact owning entry` diagnostic. Explicit
local results before those transitions compile and preserve behavior. This is
an avoided source-form limitation, not a blocker for the implemented algorithms.
No Omega source was changed. No native execution, emitted-layout comparison,
pointer validity, hardware instructions or production integration is claimed.

## Reproduction and pin updates

```sh
python3 tools/ports/x86_64-addresses/check.py --omega /path/to/omega
python3 tools/ports/x86_64-addresses/check.py --host-only
```

The checker verifies the exact pin and source bytes, all mappings, deterministic
fixture regeneration, Cargo.lock, actual Rust witnesses, Omega behavior and
three mutation controls. On a pin update, review the entire source and every
mapping first; regenerate the snapshot and fixtures only after reviewing changed
semantics. Expected numeric vectors are not measured native ABI geometry.
