# Standalone Omega process witness

[component/main.omg](component/main.omg) implements the common platform
`Program` entry. It checks the installed environment, yields, descends into
[channel.omg](component/channel.omg) for message/custody checks and
[shared.omg](component/shared.omg) for page ownership,
[notification.omg](component/notification.omg) for wake rights, and
[timing.omg](component/timing.omg) for clock authority and parking, then returns
an exit status. [native/main.omg](native/main.omg) is the compiler's independent
component consumer; its executable is packaging input, never a host program to
run. The actual process enters through the platform's native entry adapter.

Run from the repository root:

```powershell
python tools/omega-user/run_test.py --compiler path/to/new/omega.exe
```

The runner boots the same Omega ELF twice under the ordinary Rust kernel,
with and without a clock grant. Each process has two transport objects of quota, no
initial endpoints, one private page and one reader page of allowance. It checks:

- The environment snapshot and yield return.
- A full 64-byte message, including its last word, and unchanged send backing.
- A third channel allocation is refused while two consume the quota.
- Both endpoints close; a retired sender fails; another exchange can reuse the quota.
- A live shared view survives producer release, retains its funding until the last
  reader releases, and then permits a new allocation.
- Wrong-type receives leave the grant queued; reader writes and retired names are
  refused; last-page access succeeds, crossing and overflowing spans fail.
- Delegated notification names are distinct; wait rights cannot signal; signal
  delivery, wait revocation, stale names and repeated cleanup/reuse work.
- Without a clock grant, both acquisition and parking are denied.
- With the grant, a two-tick deadline parks and resumes at or after the deadline.
- The kernel reports normal exit with status zero.

Artifacts and serial logs live in `build/omega-user/tests/<timestamp>/`.
The guest's proof is actual process execution, not a mocked table callback.

This witness establishes the entry/syscall foundation used by installed standalone
apps. The separate connection run below adds cross-process shared transport;
actual app runners and the desktop harness check their event loops.

## Rust service interoperability

`component/connection.omg` drives sixteen requests through the ordinary Rust
`Server` in `source-rs/distribution/lab/channels/omega_peer/peer.rs`. Alternating replies
carry delegated notification rights. `component/connection_faults.omg` then
exercises malformed correlation, grant-only and frame-only expiry, a wrong-kind
grant and a delayed valid pair against the existing malicious-peer fixtures.
The parent also tests malformed setup and a live stalled handshake, and checks
memory, channel and notification budgets return to baseline after each case.

```powershell
python tools/omega-user/connection_test.py --compiler path/to/new/omega.exe
```

Use the standalone-runtime producer documented in
[the build guide](../../../../tools/omega-user/README.md), not F5's older compiler.
Logs live in `build/omega-user/connections/<timestamp>/`.

`component/control.omg` also accepts the kernel-installed owner connection from
a Rust `Client`. Its sixteen requests check full payloads, BUSY while a request
is retained, correct replies and duplicate-reply refusal. Four additional cases
cover malformed OPEN, a live stalled handshake, a wrong-type setup grant and a
copied request after setup. The parent verifies channel, memory and notification
refunds after each child; the runner requires both client and owner PASS markers.
No environment endpoint substitutes for the real owner control pair.

`component/services.omg` drives the interaction acquisition path against the
ordinary Rust server in `omega_peer/services.rs`. Six cases cover a valid grant,
refusal followed by retry, missing grant, wrong notification role, nonzero
reserved reply words, and a healthy response delayed past one parking interval.
The successful cases cache the grant, request optional storage and receive a
refusal, then poll presentation on the same connection and request sequence.
The provider checks the exact packets and request count; the parent checks all
three resource budgets return to baseline. The runner also requires the service
acquisition PASS marker. These are live IPC tests, not a real matrix launch yet.

`component/endpoints.omg` adds five broker endpoint cases: a usable pair,
second-part refusal with half-pair cleanup, a wrong notification grant, corrupt
reserved fields and a missing grant. `component/sealed.omg` sends a six-byte
immutable object over the shared connection; the Rust receiver checks its exact
payload and zero tail, and the sender checks that sealed backing refuses writes.
The parent checks full resource refunds after each case. Together these make
25 connection cases. The connection fixture grants the client two private pages
and the server two reader pages to cover the mailbox and one sealed object;
this does not change kernel limits or the primitive runner above.
