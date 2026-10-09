# Kernel

Owns boot/ firmware entry, core/ memory/scheduling/authority mechanisms, and arch/ CPU backends. Contracts stay at the shared source root. No dependency on distribution implementations. Pure platform hardware facts and the explicitly recorded bootstrap console are the only current platform exceptions.

See [the source layout](../../wiki/architecture/repository_layout.md).

Ordinary Rust boot initializes mechanisms and optionally admits one supplied
initial executable with bounded boot-issued grants. It neither names Cathedral
services nor implements their startup/restart policy. `boot/uefi/lab/` is compiled
only for smoke tests. The kernel-only image needs no platform service or
distribution executable; the bootstrap UART exception above remains explicit.
