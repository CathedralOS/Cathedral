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
return to its heap and physical-frame baselines. The userspace supervision fixture
keeps one client alive across 32 service crashes/restarts and checks explicit
reconnection, retired task/endpoint tickets, wait copyout, parent-exit cleanup,
and failed admission with live peers.
The watchdog fixture then recovers four blocked and four non-yielding services
using clock grants, deadline waits and owner-authorized cancellation. An independent
observer must progress before cancellation; an all-blocked session must also wake
on its deadline. Completed outcomes, stale cancellation tickets and complete memory
reclamation are checked as well.

The display fixture launches a platform display-service ELF and a separate
distribution client. It verifies redraw after a provider fault, unrelated task
progress, exclusive device mappings, NX/guard faults and all failed-admission
budgets. The default QEMU display device must provide a linear RGB/BGR GOP mode.

Run `python tools/boot-harness-rs/run.py --screenshot` for a bounded ordinary boot
and an independent pixel-for-pixel scanout check through QMP. It saves
`build/boot-harness-rs/capture/display.png` and the original PPM, then stops QEMU.
No GUI or extra Python packages are required. `--window` instead opens the QEMU
display for an ordinary interactive boot; Ctrl+C stops that run.

See [the Rust lab guide](../../source-rs/README.md) for setup, ordinary boots,
firmware overrides, logs, checks, current limitations and the bring-up sequence.
The existing `tools/boot-harness/` continues to build and boot Omega.
