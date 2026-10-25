# Matrix child admission

Start at [admission.rs](admission.rs): obtain the supplied launch's control port,
construct a startup bundle, move the child endpoints with `spawn_tree`, then
connect control. [connections.rs](connections.rs) owns channel/notification allocation and
the distinction between retained parent ends and transferred child ends.

An allocation failure closes channels already created, including when wake
notification creation fails after channel allocation. A spawn failure retains
all child endpoints, so rollback closes both sides. After successful spawn those
child names are invalid in the parent; a control-connect failure cancels and
collects the child before closing only the retained parent ends.

Success returns `Admitted`, carrying the task, control pair, interaction pair,
wake source, optional storage and optional producer pairs. Positional admission has
no producer pair. The child reads the corresponding three-
or five-endpoint environment through [matrix.rs](../matrix.rs). The caller must
install the retained endpoints and explicitly grant local view/storage authority.
No placement, storage scope or management right is conferred by this helper.

The caller also owns executable selection, funding/reservation release, launch
loans, child-slot selection, teardown ordering and recovery. There is no implicit
Drop cancellation. `close_connections` is used after task reclamation and server
detachment; callers with separately retired control connections can retain their
own explicit cleanup, as the stock matrix does.

The [stock admission coordinator](../../../../../distribution/matrix/service/children/admission.rs)
uses this path for template, catalogue and stored-object admissions. The
[independent matrix](../../../../../distribution/lab/matrix-composition/matrix/children.rs)
uses three launch loans without any stock policy. Its QEMU proof checks repeated
busy-spawn rollback, partial channel-allocation rollback, replacement and whole-
subtree reclamation. Control-connect failure cleanup is preserved from the stock
implementation but is not independently fault-injected by that proof.

`spawn_brokered` installs only the request/reply pair. The remaining child endpoints
and a notification wait right stay in `services::Offer` until acquired or reclaimed.
The owner keeps the notification's signal/control rights. Callers preserve local
input hints until `Offer::interaction_ready`, then signal the independent hint.
`close_connections` closes the notification and releases retained rights, including
unacquired wait rights. The notification replaces one queue, with the same object
charge; local notification-handle metadata also has a finite limit. Positional
startup still uses a copied wake queue. See [the bootstrap contract](../../../../contracts/services/README.md).
An explicitly offered input producer adds two channels, retained separately from
interaction and storage. Its provider must install that pair, grant bounded
production and keep its reply queue empty until both parts have transferred.
Allocation, spawn rollback and teardown include both sides of these channels.
