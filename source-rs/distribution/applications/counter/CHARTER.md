# Private counter application

A second isolated client demonstrates that the same object numbers refer to
separate app-owned data. `main.rs` enters `session.rs`, which restores and advances
a durable boot counter, then verifies it on init health requests. `state.rs`
updates COUNTER and MIRROR records in one transaction and checks their agreement.
Its only platform data link is storage. It has no display, keyboard, disk,
launch or cancellation grant. Status cannot access its private root either.

Init replaces this app independently and checks its progress while the visible
status app runs. Replacing storage invalidates the connection; read-only health
checks reconnect and verify data. Replacing counter advances its own saved count
without changing status state.

The optional `recovery-lab` probes create/list/delete temporary records, reject
out-of-range IDs and stale revisions, fill the bounded transaction and response
queues, then exit the first incarnation with uncommitted changes. Its replacement
checks that staging did not survive. Normal builds contain no such probes.
