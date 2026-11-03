# Timer wake publication and consumption

A timer interrupt records an observation and sets a pending wake bit. Ordinary
kernel code takes that wake and clears the bit. This test executes those two
helpers and checks that a wake is consumed once, several pending observations
collapse to the latest one, and a later wake can be consumed again. Nonzero
values on either side of the wake state must remain unchanged.

Run with an optimized Omega compiler:

```sh
OMEGA_BIN=/path/to/omega tools/timer-root-work-canary/run.sh
```

The script updates the local package lock and runs the program in both the
Terminal Psi interpreter and native execution on the host. Both must exit zero.
It uses an installed `omega` when `OMEGA_BIN` is unset. It does not build a
compiler implicitly.

The producer stores the observation before publishing the bit with `Publish`
ordering. The consumer claims and clears the bit with `Receive` ordering before
reading the observation. These operations remain fixed work over preallocated
atomic fields; the helper does not allocate or call a scheduler.

This is a sequential execution test of the helpers. It does not establish
interrupt delivery, concurrent interleavings, installed handler state, or safe
interrupt return. Those require the kernel boot test with real timer arrivals.
