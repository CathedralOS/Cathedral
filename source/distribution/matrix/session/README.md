# Stock matrix service session

Start at [main.omg](main.omg). Its completion dispatch shows the service turn:
upstream interaction and storage, child workflows and reclamation, ready work,
waiting, cancellation, then deferral or event dispatch. Admission waits for the
upstream interaction owner; completed reads remain retained meanwhile. A retained
snapshot is applied after publication ends and before parking.

This executes in the normal F5 stock matrix at every level. The small
[Rust entrance](../../../../source-rs/distribution/matrix/service/service.rs)
initializes the host, asks Omega for one command, executes it and reports completion.
It no longer contains the service-turn sequence. Omega selects only observations
needed by the next decision; ordinary progress steps do not scan deferred requests.
[operations.omg](operations.omg) names the private command/fact vocabulary. It is
not a kernel ABI, generic service protocol or requirement on other matrices.

The ordering preserves these obligations:

- Progress input and storage independently; a blocked admission cannot steal an
  interaction reply or stop a separate storage exchange.
- Finish child work before considering another launch or recovery. A ready deferred
  request gets priority over those operations and over another upstream input poll.
- A read-only loss query may run while interaction waits, but not while a completed
  read is still waiting for admission.
- Observe selection/launch cancellation before deferring an owner mutation. Keep
  scene custody until commit, abort or compensating restoration finishes.
- Notification-driven input runs at the start of the turn. Do not poll it again
  ahead of an already-selected request; positional fixtures without that wake keep
  their on-demand refresh.
- A wake without a received event restarts progress. A handled loss request,
  deferred event or dispatched event is consumed exactly once.

The [host](../../../../source-rs/distribution/matrix/service/host/host.rs) retains
all authority and backing: exact requests, connection identities, child handles,
storage completions and immutable images. Omega sees observation bits and the
authenticated event category, never an app-supplied PID or a raw kernel handle.
Only the issued command can complete. The host refuses to replace a retained event
and rejects dispatch/event-variant mismatches; disconnect handling still checks the
exact connection incarnation before touching a replacement.
Omega and its native host share this process: the bridge is a porting boundary,
not a new isolation boundary between mutually untrusted programs.

Descend into the host's `initialization.rs` for supplied connections and persistent
arenas, `progress.rs` for retained storage completion, `wait.rs` for actual wait
rights/deadlines, and `events.rs` for event movement. The large server and child
records occupy process-owned arenas, alongside the existing world/parent arenas;
returning them together by value would duplicate them across the guarded stack.
The stack allowance and kernel are unchanged.

The `Request` operation continues into [Omega request routing](../requests/README.md).
Detailed child-control dispatch, launch/startup role choices, local world mutations
and transport implementations remain Rust. This
milestone moves orchestration and event-category dispatch; it does not claim a
fully Omega matrix executable, a new platform runtime or a native Omega kernel.

Verification starts with `cargo test --locked -p cathedral-stock-matrix --lib`:
compiled Omega checks ordering, wrong/duplicate completion, progress restarts,
event disposition, input fairness and bounded decisions across every fact-bit
combination. Guest regression uses `desktop_test.py`: ordinary F5/cold reopen,
four nested desktops with `--interaction-delay`, launcher recovery, delayed launch
reads/binding, and late-stop launch/selection restoration. Consult `TASKS_RUST.md`
for the current result and exact scope.
