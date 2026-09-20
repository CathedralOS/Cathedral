# pic8259 upstream notices

Source: https://github.com/rust-osdev/pic8259/blob/136052bcbf081b9382fc3d72ba04b83b30f664b4/README.md

The following paragraphs and licensing section are copied verbatim from that
revision. The excerpts are collected here by Cathedral; this file is not an
upstream NOTICE file.

> This project is a fork of the [`pic8259_simple` crate](https://github.com/emk/toyos-rs/tree/master/crates/pic8259_simple) created by [@emk](https://github.com/emk).

> This code is based on the [OSDev Wiki PIC notes][PIC], but it's not a
> complete implementation of everything they discuss.  Also note that if you
> want to do more sophisticated interrupt handling, especially on
> multiprocessor systems, you'll probably want to read about the newer
> [APIC] and [IOAPIC] interfaces.

[PIC]: http://wiki.osdev.org/8259_PIC
[APIC]: http://wiki.osdev.org/APIC
[IOAPIC]: http://wiki.osdev.org/IOAPIC

## Licensing

Licensed under the [Apache License, Version 2.0][LICENSE-APACHE] or the
[MIT license][LICENSE-MIT], at your option.

[LICENSE-APACHE]: http://www.apache.org/licenses/LICENSE-2.0
[LICENSE-MIT]: http://opensource.org/licenses/MIT

## Supplied license texts

No LICENSE, LICENCE, COPYING, or NOTICE file exists in the pinned pic8259
tree. Its Cargo manifest declares `Apache-2.0/MIT`; the README above makes the
choice explicit. `LICENSE-MIT` and `LICENSE-APACHE` in this directory supply
the standard terms by copying the corresponding files from the pinned
virtio-spec-rs tree byte for byte (see `../sources.json`). They are not claimed
to be files from pic8259. The MIT text has no copyright header; no holder or
year has been invented. The pinned Cargo manifest names Eric Kidd as author.
