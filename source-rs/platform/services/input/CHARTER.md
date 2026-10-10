# Bootstrap input provider

Owns PS/2 configuration and translated set-1 decoding in userspace. The kernel
supplies exclusive byte transport, a bounded queue and IRQ wakeups. The provider
exports physical key events over IPC; shortcuts, selection and appearance belong
to the distribution. It imports contracts and the user runtime, never kernel or
distribution implementations. The pure decoder is host-testable independently.

This q35 experiment recognizes arrows, Enter and F1/F2/F3. It distinguishes press,
release and repeat; startup/overflow RESET clears held keys. One NEXT request
consumes one event, or returns IDLE after 25 ticks without an event. Buffers are bounded and may lose bursts; restarting the
provider discards previous raw/IPC queues. It does not implement text layouts,
USB, hotplug, focus, seat routing, trusted shortcuts or application authorization.
Controller replies have a 50-tick deadline; init also bounds startup. This
provider requires separately granted clock access. The recovery-lab feature adds
explicit startup/request failures for tests; normal builds exclude them.

Controller behavior is checked against the primary
[QEMU i8042 implementation](https://github.com/qemu/qemu/blob/master/hw/input/pckbd.c).
The fixed-port kernel interface rejects reset/A20 commands. This temporary PC
bootstrap route must give way to explicit device/interrupt resource admission;
the experimental contract is not a frozen OS input standard.

The private supervisor connection supports HEALTH without consuming an input
event. NEXT on the independent application link delivers events. A shared runtime
server multiplexes both, prioritizing pending control requests and bounding each
raw keyboard wait. App replacement refreshes only its data connection; input
configuration and the provider's task stay live. Recovery fault commands are
accepted only on the private control channel in test builds.
