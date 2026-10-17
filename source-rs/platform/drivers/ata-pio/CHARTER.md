# ATA PIO lab driver

Userspace, single-sector LBA28 read/write and FLUSH CACHE for the harness's
dedicated secondary ISA ATA master. Kernel wrappers expose only registers
0x171–0x177, alternate status/control 0x376, and one checked 512-byte transfer
through 0x170. The boot profile grants this controller to one child; arbitrary
ports, slave selection, multi-sector and DMA commands are unavailable.

Opening resets any interrupted command, disables device IRQs, identifies the
device, and checks LBA, flush support, capacity and `CATHEDRAL-LAB-DATA` serial.
Each readiness wait is bounded by 50 nominal PIT ticks. The driver imports only
contracts and the user runtime. The storage provider owns its lifetime.

[lib.rs](lib.rs) sequences commands; [polling.rs](polling.rs) waits for each named
phase and [status.rs](status.rs) classifies status without guest dependencies.
Failures and successful waits of ten ticks or more report opcode, phase, LBA,
status, elapsed ticks and poll count. The deadline is unchanged and the driver
does not retry uncertain writes. The original intermittent busy timeout remains
undiagnosed; `objects_test.py --release --ata-trace` captures QEMU ATA/block events
alongside these diagnostics. See the [proof guide](../../../distribution/lab/objects/README.md).

The optional `trace-storage` feature exposes [metrics.rs](metrics.rs) counters for
read/write/flush commands and cumulative readiness ticks/polls. The database
provider samples differences around requests. These describe driver activity,
not physical-media latency; ordinary builds omit the counters.

The pure status check can run on the host without linking guest syscalls:

```powershell
New-Item -ItemType Directory -Force build/ata-pio | Out-Null
rustc --edition 2024 --test source-rs/platform/drivers/ata-pio/status.rs -o build/ata-pio/status-tests.exe
./build/ata-pio/status-tests.exe
```

This models QEMU's [ISA IDE controller](https://github.com/qemu/qemu/blob/master/hw/ide/isa.c)
and [ATA implementation](https://github.com/qemu/qemu/blob/master/hw/ide/core.c).
It is temporary non-DMA bootstrap hardware, not PCI discovery, AHCI/NVMe, an
IOMMU path, or a driver validated on physical disks. The installed firmware
finishes before this exclusive controller grant is exercised. The harness
attaches only a dedicated regular raw image, never the boot ESP or a host device.
