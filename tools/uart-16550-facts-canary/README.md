# UART 16550 fact-table source canary

This compile-only harness checks the pure UART facts consumed by Cathedral's
first post-firmware serial path. It pins the COM1 base, the byte-wide register
map (including DLAB bank aliases at offsets 0 and 1), and the control/status
masks used to initialize and poll a 16550-compatible UART.

Run:

```sh
tools/uart-16550-facts-canary/run.sh
```

Set `OMEGA_BIN` to test with a specific compiler binary. Otherwise the harness
uses an installed `omega`, a built sibling `../Omega/target/debug/omega`, or
builds that sibling with Cargo, in that order. Typed-artifact validation uses
`jq` and fails with an explicit dependency error when it is unavailable.

The canary evaluates constants only. It performs no port I/O, initializes no
device, polls no hardware, emits no serial data, and grants no `PortIo` or
interrupt authority.
