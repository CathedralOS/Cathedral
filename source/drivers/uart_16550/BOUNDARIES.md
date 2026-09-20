# UART access boundaries and behavior reconciliation

This audit translates pure work requests. It supplies no UART device object,
port-I/O executor, MMIO pointer wrapper, or capability constructor.

## PortIo read/write

`backend/pio.rs::_read_register` and `_write_register` require a granted x86
port range, byte-width input/output operations, and exclusive protocol ownership
across DLAB bank changes. A valid number is not a grant. Cathedral's existing
`source/boot/uefi/own_machine.omg` explicitly reaches `PortIo`; the audited
request data does not replace or expand that authority. Eight consecutive ports
fit only when base<=65528. Addition to a validated base must retain that bound.

## MMIO read/write

`backend/mmio.rs` requires valid mapped device backing, exclusive custody for
side-effecting register reads/writes, byte access width, alignment and ordering
appropriate to the admitted target. A numeric base and stride do not establish
an `Extent`, reference or volatile view. The upstream AArch64 ldrb/strb workaround
and host volatile operations remain explicit platform binding work; no unchecked
cast is translated. The pure x64 constructor predicate additionally rejects zero,
which Rust excludes through `NonNull`. It preserves upstream acceptance of power
of two strides through128; individual operations separately reject `index*stride`
above255, matching the pin's checked-u8 offset failure. Thus stride64 or128 can
construct geometry but cannot access all eight registers through that backend.

## Register observations

`register_read`, `receive_byte`, `transmit_byte`, `divisor_step`, `dump_step`,
and `send_probe` describe accesses only. A read may consume a byte or clear an
interrupt/error; it cannot be duplicated, prefetched or reordered as a pure
getter. LSR reads clear line/error flags, MSR reads clear change indicators,
and IIR reads can acknowledge THRE. Reading THRE differs from TEMT: send capacity
uses bit5; initialization waits for bit6. DLAB aliases DATA/IER with DLL/DLM.
`divisor_step` restores the exact old LCR, including an already-set DLAB.

## Initialization and presence

`init_step` preserves scratch tests42/115, LCR/IER disable, DLAB selection,
divisor low/high writes, LCR format, FIFO reset/enable, MCR11, TEMT polling,
LSR/MSR acknowledgement and final IER installation. `action_response` separates
advance, retry and scratch mismatch. The caller calculates and validates the
configuration/divisor first. Upstream calculates after selecting DLAB and can
leave that bit set when invalid baud returns early. Prevalidation is an explicit
pure planning deviation, not a silent rewrite of the existing boot driver.

The pin's optional prescaler enters arithmetic but init never writes PSD.
That behavior is retained. PSD is an extension of the cited IP core, not
standard16550 functionality. Initializing/flushing FIFOs can discard data;
a future executor must establish a suitable connection lifecycle first.

## Streaming and exact I/O

`ready_to_receive`, `ready_to_send`, `check_connected`, `send_count` and the
register requests describe nonblocking transfer decisions. Read LSR first; read
MCR only when CTS checking and transmit capacity require it; read MSR only
outside loopback. CTS need not be asserted when checking is disabled. Connection
checks require DSR then CTS, deliberately not CD as in the pin. FIFO bursts are
at most16 bytes; non-FIFO bursts are at mostone. `try_send_byte` preserves the
pin's collapse of every zero-byte send into NoCapacity, including CTS rejection.

`receive_bytes` repeats one readiness/data pair until the buffer is full or
readiness fails. `send_bytes` polls once then emits the selected bounded burst.
Their exact variants and embedded-io read/write repeat with updated remaining
buffer ranges; upstream can spin indefinitely. This audit omits those live
loops rather than invent a completion guarantee. A Cathedral executor must use
an explicit work budget/cancellation outcome and maintain exclusive buffer and
UART custody; existing boot already bounds polls at1000000. embedded-io flush
is an upstream no-op, not proof that the transmitter drained.

## Loopback and TTY

Upstream loopback temporarily selects MCR16, resets FIFOs, drains stale input,
round-trips byte66 and bytes `hello world!1337`, then restores old MCR on success.
Early errors can skip restoration, and receive loops can wait forever. These
live hardware tests remain deliberately unexecuted; their protocol is audited
here without claiming a safe integrated executor. `tty_byte` preserves pure
backspace/delete ->[8,32,8], LF ->[13,10], and literal byte behavior. TTY
constructors perform initialization and loopback and therefore remain integration
operations. Rust formatting/Error/Send/Sync trait plumbing is not a Cathedral
capability or concurrency proof; error payloads are retained separately as data.
