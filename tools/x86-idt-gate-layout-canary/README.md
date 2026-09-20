# x86 IDT gate layout source canary

The current harness checks the canonical gate schema and separate static Layout
policy, then checks field access through local equivalent schemas. A source
regression audit compares all five fields, constants and seven placements with
commit `d4b8fa5aae189e9ee10768a1e6de5c1370fb5dcb`: size16, alignment16, offset
fragments16/16/32 at bytes0/6/8, selector2, IST4, attributes5, reserved12.

```sh
OMEGA_BIN=/path/to/omega tools/x86-idt-gate-layout-canary/run.sh
```

Without OMEGA_BIN, the sibling release binary is used. The old dependency API,
nonexistent omega-cli fallback and retired JSON dump checks were replaced;
`assert-layout.jq` remains a historical artifact and is not claimed to pass.
The plan now uses a local entries array and a named Layout witness.

Imported generated field access has a separate reproducer in
`tools/ports/x86_64-interrupts/layout_imported_probe.omg`. The successful local
consumer verifies plan normalization/source field access; it does not measure
native Omega bytes. The upstream Rust Entry has alignment4; Cathedral's existing
selected alignment16 is preserved. No instruction or live table operation runs.
