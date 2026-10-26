# Direct display recovery proof

[main.rs](main.rs) dispatches the boot-selected roles. The
[supervisor](supervisor.rs) coordinates a [drawing client](client.rs) and an
independent [observer](probes.rs), then starts two incarnations of the separately
compiled [display provider](../../../platform/services/display/direct/direct.rs).
The [kernel smoke composition](../../../kernel/boot/uefi/lab/display/display.rs)
owns this bounded test graph and the provider's exclusive framebuffer grant.
No distribution process receives a device mapping.

The supervisor/client and supervisor/observer connections use shared mailboxes
and notifications. The client connects once per provider incarnation through its
granted port. It checks INFO, malformed packets and overflowing geometry, draws
the distribution's [test pattern](../../libraries/boot-scene/lib.rs), then asks
for INFO again. The provider deliberately faults on that ninth request in the
first incarnation. The client observes closure; the supervisor collects the fault
and starts the replacement. Both old endpoint names must remain invalid.

The replacement draws the same pattern and answers INFO. Its connection stays
alive until the supervisor cancels the process; dropping it early would allow a
clean service exit and invalidate this proof. The client then observes closure
on the retained stream. Both rounds must complete inside the original 200-tick
deadline. The observer independently samples until that deadline and answers an
explicit finish request. Transport maintenance waits are not provider timeouts;
the fixture checks the actual child outcome and time rather than the old aggregate
copied-channel wait/send counters.

The supervisor and client each receive two private pages, two readers and two
notification allowances. The observer and provider each receive one of each.
The client deliberately yields after its final reply, before dropping its shared
reader. Supervisor retirement waits at most 100 ticks for zero writer/reader/channel
charges and notification objects/rights: reply receipt alone cannot prove backing
has been released. The host also requires exact heap/frame recovery
after the entire session, a contained ungranted framebuffer fault and the expected
spawn/reap/cancel counts.

The [mapping probes](../../../kernel/boot/uefi/lab/display/probes.rs) separately
exercise checked syscall copies, NX and both framebuffer guards, then fail
admission at every frame-allocation boundary. Their owner/peer completion also
uses shared transport, funded independently of the device grant. Deliberate raw
send/receive calls remain only to reject stale handles; setup still uses typed
kernel grant transfers. This is not proof that copied payload syscalls have been
removed from the whole OS.

Run from the repository root:

```powershell
python tools/boot-harness-rs/run.py --smoke --release --timeout 240
python tools/boot-harness-rs/run.py --smoke --timeout 300
```

The full smoke suite checks both provider-connection markers, shared coordination
setup/retirement and the existing fault/reclamation markers. The production
desktop uses the retained compositor protocol; this direct pattern is a mechanism
test, not a replacement desktop. The optional direct `recovery-lab` hooks are
compile-checked separately; this smoke profile selects `lab`, not those hooks.
