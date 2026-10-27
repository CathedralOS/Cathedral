# x86-64 mechanisms

[Memory](memory/memory.omg) starts at `PageTableBuilder::prepare`: reserve aligned
backing, derive the selected address coordinates, populate four tables, and check
the retained result. Its siblings own [CPU profile checks](memory/profile.omg),
[numeric path cloning](memory/clone.omg) and [walk classification](memory/walk.omg).
The [validation entrance](memory/validation/validation.omg) checks a complete
four-page candidate before accepting it; descend into endpoint/walk policy,
page images, entry encoding or address geometry. These ordinary values do not
prove that hardware may walk the tables.

[Interrupts](interrupts/interrupts.omg) validates the exception floor and its
stack assignments. [entry.omg](interrupts/entry.omg) owns the normalized maskable
arrival ABI; [fatal.omg](interrupts/fatal.omg) owns the fixed exception diagnostic
leaf. The PC timer's vector and EOI remain in the board owner.
[timer.omg](timer.omg) contains the separate x2APIC one-shot operations.

The paging probes exercise numeric policy. Interrupt prototypes retain older
Omega syntax/contracts and need separate compiler qualification; this split does
not claim live IDT installation, timer delivery or SMP support.
