# Rust boot harness

Run `python tools/boot-harness-rs/run.py --smoke` from the repository root.
This builds the separate `source-rs/` Cargo workspace and exercises its UEFI
handoff under QEMU/OVMF. Host-side Python only; no extra Python packages.

`source-rs/distribution/profile.json` selects the boot package, target and named user ELF programs. The
Cargo workspace keeps kernel implementation under `kernel/` and the UART under
`platform/drivers/`; replacing the distribution does not require moving those.

The smoke boot checks cooperative and timer-preempted tasks, dynamic admission,
task-slot exhaustion, allocation-failure rollback, stale IDs and repeated stack,
page-table and heap reclamation while a non-yielding peer stays alive. Use
`--release` for the optimized path. Expected-fault runs remain separate.
The composed image also checks isolated ELF programs and capability-addressed
IPC: echo exchanges, stale/foreign handles, rights, backpressure, checked copyout,
and blocked-receiver wakeups on revoke or peer exit/fault. Each session must
return to its heap and physical-frame baselines.

See [the Rust lab guide](../../source-rs/README.md) for setup, ordinary boots,
firmware overrides, logs, checks, current limitations and the bring-up sequence.
The existing `tools/boot-harness/` continues to build and boot Omega.
