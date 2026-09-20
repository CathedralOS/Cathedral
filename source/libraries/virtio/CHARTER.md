# CHARTER — VirtIO protocol library

Reusable, pure VirtIO facts, byte transformations, queue arithmetic, and bounded
validation. A descriptor address, register offset, allocation request, or feature
selection is data. It conveys no right to access memory or a device.

The package owns neither DMA nor mappings, discovery, interrupts, queue activation,
volatile access, allocation, barriers, or concurrency. Those require separately
named owner interfaces. There is no production import. `PORT.md` records the exact
pinned-source coverage and distinguishes semantic tests from physical ABI evidence.
