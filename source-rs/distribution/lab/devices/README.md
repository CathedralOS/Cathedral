# Device lifecycle lab

[main.rs](main.rs) selects the supervisor or driver role. Only the driver
receives the COM2 register/IRQ grant. [supervisor.rs](supervisor.rs) launches,
communicates over ordinary IPC, checks that a copied name confers no authority,
then faults/cancels/replaces the child. [driver.rs](driver.rs) binds the
[platform UART protocol](../../../platform/drivers/uart_16550/interrupt.rs)
to the syscall transport and probes hostile input and abandoned hardware state.

Run the full x86 QEMU suite:

```text
python tools/boot-harness-rs/run.py --smoke --release
python tools/boot-harness-rs/run.py --smoke --timeout 90
```

The [kernel guide](../../../kernel/core/user_tasks/devices/README.md) records
the exact grant, notification, reset and test boundaries. This executable is
a bounded driver/supervisor proof; it is not a desktop serial service.
