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
python tools/omega-user/run_test.py --matrix --compiler path/to/new/omega.exe
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

Two further boots restrict the same program's channel allowance. Zero channels
must return 102; one channel must close its first allocation and return 109.
These checks require the kernel's complete, exact exit report, not merely a
success message printed by the program. `--matrix` also boots the ordinary stock
matrix with a clock and enough image funding, but no service endpoints. Its
cleanup and kernel exit must both report INVALID_ARGUMENT (`2^64 - 22`),
including the upper bits. No application code is patched for these failures.

The packaging consumer's `main` is not the booted entry. The linker names the
independent component's checked `Program::run` export as `omega_app_step`;
the native `_start` calls it and passes its result directly to kernel EXIT.

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
`component/objects.omg` receives an immutable record after the producer drops
its source right and reads it after closing the connection. It checks exact data,
page-end and overflow bounds, preserved output tail, wrong-kind and expired
handles, and rejected writes. Three additional cases ensure plain services,
interaction acquisition and endpoint acquisition reject and release unexpected
objects. The parent checks full resource refunds after each case. Together these
make 29 connection cases.

`component/construction.omg` adds a thirtieth case: build a 46,080-byte image in
8 KiB chunks, seal it and send it to Rust, which checks every word and the zeroed
page tail. It rejects logical/physical overflow and writes through shared,
sealed or expired handles. A staging buffer can also be abandoned explicitly; a deliberately failed seal
must clear its handle and refund enough quota for the subsequent full image.
The client has thirteen private pages and the provider thirteen reader pages
for its mailbox plus this twelve-page image; the reverse direction has two each;
the fixture root has sixteen pages of each allowance;
this does not change kernel limits or the primitive runner above.

## Independent service clients

Run `python tools/omega-user/connection_test.py --service --compiler path/to/omega.exe`.
[service/main.omg](component/service/main.omg) alternates two independent clients
and owner control, then parks on all live interests. Its startup establishes
connections asynchronously; its request dispatcher holds or echoes sealed images
and reports test state to the owner. These are fixture policies, not platform
roles or a fixed client limit. The provider is a standalone Omega process; the
[Rust adversarial peers](../../../../source-rs/distribution/lab/channels/omega_peer/service/service.rs)
use the real kernel and existing v4 protocol.

Thirteen scenarios cover stalled setup, parent and sibling progress, a held
request surviving unrelated calls, disconnect cleanup, repeated image echoes,
both separately observed grant/frame arrival orders, incomplete-pair expiry,
mismatched/duplicate/zero grant IDs, undeclared attachments and wrong packet
families. Wrong connection-generation and request-ID tickets cannot consume the
pending request. A genuine ticket from another client is rejected even when
both clients have the same generation and request ID. This does not yet test reconnecting a replacement client into
the same slot.

After each failed client retires, the surviving client sends three-page images.
Together with its and the owner's mailboxes this consumes all five funded reader
pages: a leaked partial/held reader would prevent progress while the service is
still alive. Final teardown must restore logical pages, physical backing and
notification quotas. The fixture funds three notification objects for its three
connections; it changes no kernel limit or normal desktop grant.

## Child admission

Run `python tools/omega-user/connection_test.py --admission --compiler path/to/omega.exe`.
[supervisor/main.omg](component/supervisor/main.omg) is a standalone Omega
process that owns two lent launches. On owner command it admits a Rust peer into
a slot through the runtime's [admission](../../../platform/libraries/user-runtime/admission/main.omg),
answers the peer's service acquisition from the preallocated offers, echoes its
other requests, counts copied messages on the provider's storage end, and
reclaims the slot when the peer exits or dies. [owner.omg](component/supervisor/owner.omg)
decodes admit, stop, status and exit; [slots.omg](component/supervisor/slots.omg)
steps one slot per turn. Two slots, the echo protocol and the status words are
fixture policy, not platform roles.

The [Rust owner](../../../../source-rs/distribution/lab/channels/omega_peer/supervisor/supervisor.rs)
is the initial program. It lends launches 1 and 2 to the supervisor at launch 0,
because the kernel lends a launch only toward a lower task slot, and drives it
through the kernel-installed control pair. Twelve scenarios, each with a fresh
supervisor and full budget checks afterwards:

- Six echo children admitted into one slot in turn, each exiting by itself, under
  a fourteen-channel reservation that a per-child leak would exhaust.
- A storage child acquires both endpoint parts, sends three copied messages the
  supervisor must drain, then echoes.
- An idle child is stopped on command with a cancelled outcome; then the
  supervisor itself is cancelled with a live child and the kernel reclaims the tree.
- One child dies holding an unanswered request; another holds while six owner
  status calls still answer, then is stopped.
- Refused admissions (no interaction, unsupported capture, unknown launch, bad
  slot, occupied slot, launch already in use) leave nothing behind and a later
  storage admission still succeeds; stopping an empty slot is refused.
- A child checks every refusal code: taken interaction, incompatible range and
  version, invalid bounds and parts, unavailable contracts and reserved words;
  another child finds storage unavailable when it was not granted.
- Six rounds submit an echo child in each slot, then collect both through
  ordinary service turns while the owner queries status on its separate connection.
- Six immediate stop requests race asynchronous publication; both a pending
  rollback and an already-created child must release the bundle for reuse.
- Killing the supervisor after asynchronous submission reclaims its child and
  all channels, notifications and physical backing.

The booted kernel's staged-admission fixture separately holds construction at
known boundaries. It verifies cancellation and authority withdrawal before
publication, retained results on bad copyout, and that completing admission
cannot wake or overwrite a parent's unrelated blocked syscall. The live Omega
witness does not assume that construction must take a minimum amount of time.

The owner can also submit stop with command 5. The Omega supervisor retains a
stopping flag and calls the shared admission retirement poll once per ordinary
turn, releasing its listener/bundle only after rollback or collection. It keeps
answering owner requests. Six cycles stop a child with a held request, then reuse
its slot for an echo child. Another six race cancellation of submitted construction
with an independent echo child. Command 6 submits and cancels before the normal
admission poll, exercising the retained receipt path regardless of whether kernel
publication already won; six cycles reuse that slot afterwards. These exercise
the native nonblocking task bindings without
assuming cleanup must take long enough for a particular message overlap. The
kernel smoke fixture holds the owner in an unrelated IPC wait through real
subtree reclamation and verifies no premature wake or syscall-result overwrite.

Before connecting to its owner, [retirement.omg](component/supervisor/retirement.omg)
allocates a real bundle and deliberately supplies an invalid receipt. Both polling
and the blocking release convenience must retain it; sending through a retained
endpoint still works. After repairing the fixture receipt, release invalidates the
endpoint. This distinguishes rejected observation from completed rollback.

The peers use the ordinary Rust service bindings over the shared v4 transport.
This proves admission, offers and reclamation on the common runtime; it does not
place children, route input or claim a matrix.

### Overlapping requests

[component/pipeline/main.omg](component/pipeline/main.omg) dispatches two real
Omega peers. The client sends a sealed input; the provider holds that request
while answering the second, then accepts a third into the freed slot. The first
reply arrives after 150 ticks, still correlated to its original request. The
provider refuses forged connection tickets and duplicate settlement while the
other request stays live. It reads the first backing again after releasing the
second request's reader. Both sides enforce a two-request bound. A separate
fixture channel holds the provider before consuming request one until the
client has observed mailbox backpressure for its attached second request;
retry must keep the first request and its backing alive. A second scenario
acknowledges that first frame while leaving its typed grant queued, forcing
backpressure at the separate grant slot instead. Both must permit a safe retry.
The healthy exchange also holds client consumption until the provider has
failed to publish its final reply into the full mailbox. A fixture-only word
outside the wire header releases that hold after draining old notifications.
The provider then waits only for the reply-consumption notification and checks
the acknowledgement sequence. No new request or periodic retry can rescue it;
the unfixed client fails with provider status 536. This tests the wake needed
by event-driven providers, which the fixture's ordinary tick polling hid.

```powershell
python tools/omega-user/connection_test.py --pipeline
```

The same peers inject closure with two requests outstanding, an unknown reply
ID, a duplicate completed reply ID while two others remain outstanding, and a
sealed reply grant with no frame. The latter must expire as a partial reply,
release its reader and retire the whole connection. The Rust launcher compares
memory, physical-frame, channel and notification accounting after every case,
then repeats the healthy exchange. Fault construction stays in
[pipeline/faults.omg](component/pipeline/faults.omg), outside the runtime.

This qualifies transport. The matrix still needs continuations that dispatch
poll and visual replies by those IDs and retain child/view generations.
