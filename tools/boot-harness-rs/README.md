# Rust boot harness

Run `python tools/boot-harness-rs/run.py --smoke` from the repository root.
This builds the separate `source-rs/` Cargo workspace and exercises its UEFI
handoff under QEMU/OVMF. Host-side Python only; no extra Python packages.

`source-rs/distribution/profile.json` selects the boot package, target and named user ELF programs. The
Cargo workspace keeps kernel implementation under `kernel/` and the UART under
`platform/drivers/`; replacing the distribution does not require moving those.

Ordinary boots build only `startup.initial` and the optional `startup.launches` array.
The kernel admits that initial executable; `distribution/init` launches and uses
display/input/storage providers and the separate status and counter applications. The kernel fixtures below are compiled
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

The memory fixture allocates private pages, seals buffers to a boot-approved peer,
and tests RO/NX permissions, guards, stale tokens, zeroing, budgets and both peer
failure paths. It injects allocation failures at each backing/table boundary and
requires complete heap/frame reclamation. Ordinary screenshot and interactive
checks include the app's shared pixel tile. Recovery/storage builds also require
malformed pixel buffers to complete with errors and release their read leases.

The [rendering fixture](../../source-rs/distribution/lab/rendering/README.md) then
compares direct rows and flattened shared leaves on real guest pages. It checks
that a native writer can bypass a software span inside its granted page, while
an unmapped page boundary faults. Two sessions assert page-edit/IPC counts and
full reclamation. Host pixel-copy and padding measurements have a separate runner
at `tools/rendering-lab/run.py`; these experiments do not adopt a surface protocol.

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
The status application shares its distribution-owned drawing code with the smoke fixture.

See [the Rust lab guide](../../source-rs/README.md) for setup, ordinary boots,
firmware overrides, logs, checks, current limitations and the bring-up sequence.
The existing `tools/boot-harness/` continues to build and boot Omega.

`--input-test` boots normally, injects keyboard events over QMP and checks every
pixel after navigation/toggling. Arrows select a labeled panel; Enter toggles it.
The final capture is `build/boot-harness-rs/input-test/display.png`.

`--recovery-test` enables init/provider/app `recovery-lab` features. It wedges both
first-generation providers, then injects a crash, busy loop and blocked request
loop into each provider. It checks automatic recovery, unchanged sibling and app
identities, queued keyboard input during display loss and retained scene state.
F3 then crashes/hangs the app and fills its reply queue, proving init and providers
survive all three replacements.
The replacement app probes denied task/device authority and rejected fault
commands on data connections. Exact scanout checks include all text, generations
and last-recovery labels. Healthy idle must not restart any task. Logs and the
final screenshot go in `recovery-test/`. Add `--release` for optimized verification.
Fault handlers and F1/F2/F3 recovery shortcuts are absent from normal builds.

Smoke additionally checks a full three-child/two-link graph across 16 alternating
app/provider replacements and complete memory reclamation. The readiness fixture
checks two receive channels, first-ready precedence, no-consume observations,
denied clock/rights, peer closure and all-blocked timeout. Existing deadline,
copyout, late reply, exclusive-device and admission-failure coverage remains.
