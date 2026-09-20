# Pinned TLB operand witnesses

```sh
python3 tools/ports/x86_64-tlb/check.py --omega /path/to/omega
```

`--host-only` runs source freshness/inventory, 43 Rust measurements, six actual
upstream UEFI-x64 PCID assertions and four Rust tests. Rust needs
`nightly-2026-09-04` with the `x86_64-unknown-uefi` target. Host tests cover all
65,536 PCID inputs and exact extracted pure bodies; hardware tails never run.

The full command checks the production Omega module, executes its pure
algorithms through const evaluation, consumes the actual descriptor layout
with a local equivalent schema, rejects four body mutations, and reproduces
the imported generated-field visibility diagnostic. Passing the local consumer
does not measure native Omega ABI.

`range-cases.json` lists 15 explicit pairs of pinned and architectural outcomes.
AMD's encoded count means additional pages. The pinned source advances by
max(encoded,1); the separately named architectural recipe advances by the
addressed page count. Both behaviors are retained visibly and tested.

See `source/libraries/x86_64/tlb-operands.PORT.md` for exact provenance,
representation limits, source mapping and the remaining instruction-family audit.
