# Numeric MSR word transport

Status: **tested**. All 136 Rust-derived observations, Omega split/combine
comparisons, 130 round trips and two body-mutating controls pass.
This completes the small raw-word component left beside the tested
[register operand recipes](register-operands.PORT.md).

The pinned `Msr::read` combines supplied high/low u32 register outputs into a u64;
`Msr::write` splits a supplied u64 into low/high u32 inputs. The three machines in
[msr_words.omg](msr_words.omg) implement only those expressions. They construct
no register object and execute no read/write instruction. Source:
`src/registers/model_specific.rs` at x86_64
`cc35c876d3badb57df54a66e22f7768a52be95f2`,
[MIT OR Apache-2.0](../../../licenses/rust-osdev/x86_64/).
The exact pinned bodies and existing register representation were reviewed;
ordinary u32/u64 parameters suffice, with no new nominal transport type.

The host harness requires exact upstream checkout bytes and extracts the three
unchanged expressions into a small Rust witness. These are pure expression
mirrors, not calls to unsafe public MSR methods. It measures 130 splits (every
single bit and low-bit prefix, zero, all bits, and mixed patterns) plus six joins.
Omega evaluates actual helper bodies, compares both halves to Rust observations,
and round-trips every split. Two controls alter the expected low or high output
while retaining the zero-result success contract.

The [inventory](msr-words-inventory.json) binds the complete source file and maps
the two enclosing methods only at their numeric component boundary. Native code,
CPU observation, MSR availability and live access authority remain unmeasured.
Compiler: Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

```sh
python3 tools/ports/x86_64-msr-words/check.py --omega /path/to/omega
python3 tools/ports/x86_64-msr-words/check.py --host-only
```
