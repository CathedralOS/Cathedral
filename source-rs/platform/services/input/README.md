# Physical input provider

[main.rs](main.rs) selects keyboard-only, compositor-source or matrix-source
operation. [matrix.rs](matrix.rs) initializes the controller, reads physical
input and publishes coordinates/buttons/keys into the root matrix. Matrices
then attenuate and translate input recursively; apps never use the hardware
transport.

The matrix-source path uses the [shared connection client](../../libraries/user-runtime/connection/README.md):
one shared page and notification per direction, initialized from the owner's
explicit endpoint handoff. Input requests retain their identity through slow replies;
they do not time out and retry mutations. Nested forwarding is a separate migration.

The default distribution enables `absolute-pointer`. [absolute.rs](absolute.rs)
negotiates QEMU's VMMouse protocol, drains bounded packets and maps its 16-bit
absolute axes to the local screen extent. It ignores the accompanying PS/2
notification packets while active. Unsupported devices retain relative PS/2
input; a malformed/failed absolute stream disables it and resets logical input.
The older lab profiles keep their existing relative-input path.

The [runtime transport](../../libraries/user-runtime/keyboard.rs) traps through
INPUT_VIRTUAL_POINTER. The [kernel input boundary](../../../kernel/core/user_tasks/syscalls/devices/keyboard/virtual_pointer.rs)
requires the exclusive boot input grant and validates all 16 writable reply
bytes before consuming a packet. The [PC adapter](../../../kernel/arch/board/pc/vmmouse.rs)
exposes only version, pointer status/data and enable/disable/absolute commands,
under the recognized TCG/VMware hypervisor identity. There is no arbitrary port
or general VMware command interface. Retirement resets the virtual pointer
alongside the old raw-byte queue. ARM64 has no such board device.

Protocol references: [QEMU VMMouse](https://github.com/qemu/qemu/blob/master/hw/i386/vmmouse.c)
and [VM port](https://github.com/qemu/qemu/blob/master/hw/i386/vmport.c).
This is emulator integration, not USB/HID support. The desktop test checks QMP's
active absolute device and drives the actual screen using absolute events;
`--relative-pointer` disables VM port to exercise PS/2 fallback on the same UI.
