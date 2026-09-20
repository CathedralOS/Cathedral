# PCI facts and pure plans

This package owns configuration byte decoding, BDF values, standard header and
register facts, BAR arithmetic, bounded capability traversal, MSI/MSI-X facts,
and finite configuration operation plans. Its only inputs are supplied values
and snapshots. It has no service reach, allocator, MMIO, port I/O, interrupt
binding, firmware calls, or authority-bearing references.

A driver or kernel resource owner must acquire configuration access and bind
operations to a specific BDF with width, lifetime, serialization, and completion
semantics. These remain outside this library. A plan is data and cannot discharge
those obligations. Reserved/unknown register values remain observable in raw
facts; an `ok` flag reports only the validation described beside that helper.
