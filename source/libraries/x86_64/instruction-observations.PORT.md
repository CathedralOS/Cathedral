# Detached CPUID observation predicates

Current stage: **tested** by Rust witnesses and Omega semantic evaluation.

This slice completes the two ordinary predicates left for later extraction in
the TLB instruction-family audit. `instruction_observations.omg` interprets
caller-supplied u32 values: RDRAND's bit 30 in leaf 01 ECX and SMAP's bit 20 in
leaf 07 EBX. These boolean answers do not execute CPUID, verify the observation's
origin, check CR4 state or grant an RNG/SMAP provider.

Source: x86_64 `cc35c876d3badb57df54a66e22f7768a52be95f2`,
`src/instructions/{random,smap}.rs`, MIT OR Apache-2.0; see
[retained licenses](../../../licenses/rust-osdev/x86_64/).
[instruction-observations-inventory.json](instruction-observations-inventory.json)
records all 14 lexical anchors in both files: two predicate components
translated, 12 hardware/provider/lifecycle/scaffolding anchors omitted. The
entire original constructors are not claimed as callable Omega providers.

The reference generator extracts the exact two Rust condition expressions and
checks their source spelling against the pin. Those expressions execute on
explicit observation records; the original CPUID operations and zero-sized
provider markers are absent. All 32 single-bit values plus zero/all-ones are
checked for each predicate: 68 Rust observations and matching Omega body checks.
Two mutations invert an expected true result and must compute failure under an
unchanged success contract. Both controls and the positive Omega fixture
(11 sources) pass on `eaa7993a23623cd8fabf45350340479c5c9c7879`.

```sh
python3 tools/ports/x86_64-instruction-observations/check.py --omega /path/to/omega
```

Compiler SHA-256:
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
No native ABI, live feature detection or instruction execution claim is made.
The previous TLB source and audit remain unchanged; this is a later bounded
completion overlay for those two predicates.
