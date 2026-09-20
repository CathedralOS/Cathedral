# CPUID result predicates

`check.py --omega /path/to/omega` audits the pinned source, runs the exact
extracted Rust condition expressions, executes 68 matching Omega bit checks,
and requires two expected-value mutations to fail. `--host-only` skips Omega;
`--controls-only` reuses a separately verified positive fixture.

Neither the reference nor the Omega module executes CPUID or creates a feature
provider. See `source/libraries/x86_64/instruction-observations.PORT.md`.
