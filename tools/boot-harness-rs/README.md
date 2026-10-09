# Rust boot harness

Run `python tools/boot-harness-rs/run.py --smoke` from the repository root.
This builds the separate `source-rs/` Cargo workspace and exercises its UEFI
handoff under QEMU/OVMF. Host-side Python only; no extra Python packages.

The smoke boot checks cooperative and timer-preempted tasks, dynamic admission,
task-slot exhaustion, allocation-failure rollback, stale IDs and repeated stack,
page-table and heap reclamation while a non-yielding peer stays alive. Use
`--release` for the optimized path. Expected-fault runs remain separate.

See [the Rust lab guide](../../source-rs/README.md) for setup, ordinary boots,
firmware overrides, logs, checks, current limitations and the bring-up sequence.
The existing `tools/boot-harness/` continues to build and boot Omega.
