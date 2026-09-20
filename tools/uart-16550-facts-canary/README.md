# UART 16550 fact-table canary

This harness evaluates the existing UART fact snapshot through Omega's constant
semantic evaluator. It pins COM1, register offsets, DLAB aliases and the
control/status values used by the existing post-firmware serial path.

```sh
OMEGA_BIN=/tmp/cathedral-omega-eaa7993/release/omega tools/uart-16550-facts-canary/run.sh
```

Without `OMEGA_BIN`, the runner uses the installed compiler, sibling debug
binary, or Cargo fallback. Python3 is required. The temporary source copy avoids
package snapshots rejecting symlinks that escape the source root.

The computed snapshot must satisfy16 expected values. A second invocation
changes expected COM1 from1016 to1017 inside the test body and must reject the
computed result1. This replaces the obsolete numbered JSON artifact checks;
current `--check` does not export those files. It performs no port I/O and makes
no native execution, hardware, `PortIo` grant or foreign-layout claim.
