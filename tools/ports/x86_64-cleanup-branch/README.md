# Detached cleanup branch witnesses

`check.py --omega /path/to/omega` reproduces original fixtures, calls actual
pinned Rust cleanup on owned tables, evaluates matching Omega plans and full
entry scans, then demands body-mutating failures. `--host-only` skips Omega;
`--start`/`--end` select branch batches; `--controls-only` selects additional
cases and controls.

The singleton-page profile is explicit. Complete range traversal remains work;
deallocation callbacks only record detached test observations. See the
[port record](../../../source/libraries/x86_64/cleanup-branch.PORT.md).
