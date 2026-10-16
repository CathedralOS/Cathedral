# Synchronous block-device boundary

[lib.rs](lib.rs) defines the fixed-sector `Device` trait and its completion,
flush and failure semantics. It is an in-process adapter boundary, not an IPC
protocol, syscall ABI or device grant. The package has no implementation imports.

The [ATA driver](../../drivers/ata-pio/lib.rs) implements it; the
[object store](../object-store/lib.rs), [volume adapter](../storage-volume/lib.rs)
and [engine backend](../storage-volume/engine/backend.rs) consume it. Host fake
devices exercise the same boundary without depending on the driver or runtime.
A future transport can implement this trait without importing the storage engine.
