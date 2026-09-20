# Explicit-profile register expression witnesses

`python3 tools/ports/x86_64-encrypted-registers/check.py` checks 100 pinned
pure-expression Rust observations, ten Omega profile fixtures, ten operand body
mutations and two observed-register body mutations. It uses the isolated eaa7993
Omega compiler by default; override `--omega` if needed. `--start`/`--end` select
a half-open fixture range, and `--host-only` stops before Omega evaluation.

This is an expression reference using real PhysAddr, PhysFrame and flag methods.
No CR3/MSR method or register instruction executes. The generator verifies pinned
expression fragments and the inventory binds the two complete source files.
Configuration happens in isolated subprocesses before address construction.
See the source PORT for the latest-only physical bit versus accumulated PTE-mask
distinction, API scope, measured status and authority limits.
