# Owned GDT storage witnesses

Run `python3 tools/ports/x86_64-gdt-storage/check.py --omega /path/to/omega` from
Cathedral. `--host-only` checks actual pinned Rust observations and generated
fixtures; `--case NAME` evaluates one fixture; `--controls-only` checks the four
negative body controls. The source inventory uses the complete pinned gdt.rs.

The exact scope, selected-word comparisons, fixed backing profile and native
limits are in [gdt-storage.PORT.md](../../../source/libraries/x86_64/gdt-storage.PORT.md).
No reference operation installs a table or executes a hardware instruction.
