# x86 IDT gate layout source canary

This compile-only harness checks Cathedral's pure x86-64 interrupt-gate schema
and programmable layout policy. It pins one fixed 16-byte, 16-aligned gate and
the exact three-fragment tiling of the logical 64-bit entry field across offset
bits 0..15, 16..31, and 32..63. The selector, IST, attributes, and reserved
word occupy the remaining architectural positions. All five logical inputs
remain runtime-relevant, and the checked layout constructor's complete write
frame is exactly its private `self.entries` planning buffer.

Run:

```sh
tools/x86-idt-gate-layout-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Typed-artifact validation uses
`jq` and fails with an explicit dependency error when it is unavailable.

The canary validates geometry only. It resolves no entry identity, writes or
publishes no IDT, provisions no stack, executes no `lidt`, and grants no
resolver, root-admission, or `IdtControl` authority.
