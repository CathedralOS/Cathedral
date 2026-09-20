# Numeric MSR word witnesses

`check.py --omega /path/to/omega` extracts exact pinned pure expressions, compiles
an isolated Rust witness, then checks actual Omega split/join/round-trip bodies
and two expected-value mutations. `--host-only` stops after the Rust observations.
All temporary artifacts are isolated and removed. No hardware instruction runs.
Scope: [msr-words.PORT.md](../../../source/libraries/x86_64/msr-words.PORT.md).
