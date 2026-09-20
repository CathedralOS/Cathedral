# Pinned x86 interrupt values

```sh
python3 tools/ports/x86_64-interrupts/check.py --omega /path/to/omega
```

Use `--host-only` for source mapping, generator freshness, actual pinned Rust
measurements/target assertions, four Rust tests and canonical gate regression
audit. The full command also checks production Omega source, executes pure
algorithms in const evaluation, rejects six body mutations, and reproduces the
two explicitly retained diagnostics.

`main.omg` covers112 option combinations,256 exception/index classifications,
existing exception constants, every byte of4/40-byte records,16-byte gate
fragments, reserved-byte errors and ranges. `table_impl.omg` copies the exact
production codec source, changing only module/import names, and appends a
fixture-only bridge into its private scan machines. `table_main.omg` executes
three final table slots, successful decode, and latched reserved-byte errors.
The generator checks this copy against production on every run.

`table_encode_main.omg`, `table_decode_main.omg` and `table_reject_main.omg`
split full256-entry encoding, successful decoding and invalid-byte rejection
into independent evaluator budgets. `storage_main.omg` checks replacement and
reset. `full_table_budget_probe.omg` retains the combined roundtrip/error fixture
that reaches the100,000-step budget; the split fixtures solve that harness issue.
The production scan bodies also pass source/termination/index checking.

`layout_local_projection.omg` consumes actual plans with local equivalent
schemas. `layout_imported_probe.omg` reaches the generated-field privacy issue
on the real imported schemas. Neither claims native Omega layout measurements.
See `source/drivers/facts/x86_interrupts.PORT.md` for provenance and scope.
