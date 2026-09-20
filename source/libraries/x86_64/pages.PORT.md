# Numeric page/frame geometry and ranges

This X86-001/002 slice translates numeric algorithms from `src/structures/paging/page.rs`
and `frame.rs` at x86_64 revision `cc35c876d3badb57df54a66e22f7768a52be95f2`.
Modified derivatives retain MIT OR Apache-2.0; see the [licenses](../../../licenses/rust-osdev/x86_64/)
and [source reconciliation](RECONCILIATION.md). It extends the existing
[address arithmetic](addresses.PORT.md), using its `NumberResult` cases and
`OverflowingStep` carrier. No new frame grant, mapping or page-table owner exists.

## Profile and behavior

The accepted sizes are 4 KiB, 2 MiB and 1 GiB. Virtual geometry uses the pin's
48-bit canonical, four-level profile; physical geometry uses its default
52-bit envelope without an ambient encryption mask. These are numeric profiles,
not discovered CPU capabilities. Intel's [SDM Volume 3A](https://cdrdv2-public.intel.com/874249/253668-090-sdm-vol-3a.pdf),
chapters 4–5, and Omega's data/case, dependent-value and authority contracts were
consulted. LA57, machine-selected MAXPHYADDR and memory-encryption state are
outside this profile.

`pages.omg` validates aligned starts, computes containing pages, converts frame
numbers, composes checked page-table indices, and provides checked arithmetic,
distances, stepping, range counts/byte sizes and forward/reverse indexed lookup.
Unused lower indices must be zero for huge-page construction. Every public
operation validates its numeric inputs; successful values convey no backing,
allocation, pointer validity or permission to access memory.

Ordinary page arithmetic uses raw addresses and rejects results in the canonical
hole. Step uses dense canonical ordinals and skips that hole. Raw subtraction
and Step distance remain distinct. Multiplication and addition are checked even
where Rust release arithmetic could wrap. Counts are u64; the upstream 32-bit
usize saturation branch is outside the UEFI-x64 profile.

Ranges use explicit bounds and caller-owned indices rather than mutable Rust
iterator objects. Reversed ranges and equal exclusive bounds have count zero.
Gap-spanning virtual ranges are rejected at admission. Indexed selection never
advances past the accepted interval. In particular, a singleton at the last low
page or the first high page can be selected successfully; the corresponding
pinned forward/reverse iterator panics while updating its cursor after selection.
Six Rust witnesses record these differences explicitly, across all three sizes.
This is a deliberate checked policy, not equivalence to the original panic state.

## Source audit and evidence

[pages-inventory.json](pages-inventory.json) binds both complete source files and
184 lexical anchors: 113 translated and 71 deliberate omissions, no pending or
blocked anchors within this numeric slice. Nominal wrappers, operators,
formatting, unchecked/pointer APIs and Kani proof infrastructure are explicitly
accounted for. Finite examples do not establish universal Kani proofs.

The deterministic fixture has 267 numeric cases. Of these, 253 execute the actual
pinned Rust implementation; six separately record the deliberate iterator
differences above. The remaining cases test added validation policies. The
filtered upstream page tests (14) and frame test (1) also execute with
`nightly-2026-09-04`, without the hardware instruction feature. Pinned source is
unchanged. All upstream 64-bit forward/backward/distance table examples are
retained. Additional Omega fixtures select representative positions in the
upstream 1000-page ranges and test overflowing-step flags. The original complete
1000-iteration loops execute only in the Rust test run.

The canonical checker compiles each numeric group separately and evaluates its
actual body as a constant, then checks the extra scenarios. It changes three
actual expected results independently and requires each to compute 1 and fail
the unchanged success contract. Splitting fixtures bounds proof-checking cost;
the aggregate source is retained as the deterministic corpus. No source-only
check or expected numeric vector is evidence of native layout or execution.

Compiler revision: `eaa7993a23623cd8fabf45350340479c5c9c7879`.
Binary SHA-256: `2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
Verification status: **tested**. Rust witnesses and filtered upstream tests pass;
all 267 Omega numeric scenarios, the extra scenarios and three body mutations
pass. No production build
root imports the new module; native ABI and mapping integration are not claimed.

```sh
python3 tools/ports/x86_64-pages/check.py --omega /path/to/omega
python3 tools/ports/x86_64-pages/check.py --host-only
```

On pin updates, review complete source changes, each disposition and all explicit
deviations before regenerating inventory or vectors.
