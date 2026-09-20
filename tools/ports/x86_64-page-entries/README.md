# PTE/index/level witnesses

See the [port record](../../../source/libraries/x86_64/page-entries.PORT.md).
`check.py --omega /path/to/omega` audits the complete source file, verifies
deterministic fixtures, executes143 actual pinned Rust witnesses plus the
upstream index-step test, then evaluates151 Omega scenarios and three body
mutations. `--host-only` omits Omega execution. Rust requires
`nightly-2026-09-04` for the pinned Step API.

Each selected group is compiled separately; `main.omg` retains the full source
corpus but has no aggregate semantic assertion. Source-checking that file alone
does not execute the tests. `cases.json` is expected behavior, not native ABI
measurement. Table-level anchors remain explicitly pending in the inventory.
