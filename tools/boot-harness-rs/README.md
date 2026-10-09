# Rust boot harness

Run `python tools/boot-harness-rs/run.py --smoke` from the repository root.
This builds the separate `source-rs/` Cargo workspace and exercises its UEFI
handoff under QEMU/OVMF. Host-side Python only; no extra Python packages.

`source-rs/distribution/profile.json` selects the boot package, target and named user ELF programs. The
Cargo workspace keeps kernel implementation under `kernel/` and the UART under
`platform/drivers/`; replacing the distribution does not require moving those.

Ordinary boots build only `startup.initial` and the optional `startup.launches` array.
The kernel admits that initial executable; `distribution/init` launches and uses
display and input providers and keeps all three tasks alive. The tests below are compiled
only with `--smoke`. `--kernel-only --smoke` runs kernel and raw-user tests without
building any platform service or distribution executable. `--kernel-only` boots
the kernel alone. `--profile tools/boot-harness-rs/profiles/minimal.json` supplies
only a hello initial program, with no child or display grant.

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
Capture waits for userspace startup readiness and rejects accidental lab execution.
The ordinary scene shares its distribution-owned drawing code with the smoke fixture.

See [the Rust lab guide](../../source-rs/README.md) for setup, ordinary boots,
firmware overrides, logs, checks, current limitations and the bring-up sequence.
The existing `tools/boot-harness/` continues to build and boot Omega.

`--input-test` boots normally, injects keyboard events over QMP and checks every
pixel after navigation, toggling and ten alternating service restarts. Arrows
select a panel, Enter toggles it, F1 restarts input and F2 restarts display. The
final capture is `build/boot-harness-rs/input-test/display.png`. Smoke also tests
multiple launch grants, surviving sibling queues, failed launch rollback, device
access denial and parent-exit reclamation of a blocked input provider.
