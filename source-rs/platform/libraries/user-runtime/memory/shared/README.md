# Reusable shared buffers

Start at [shared.rs](shared.rs): consume a private allocation into a `Writer`,
delegate a read view, and access aligned words without lending references into
changing memory. [transfer.rs](transfer.rs) handles typed setup through an
authorized endpoint. The [kernel memory owner](../../../../../kernel/core/user_tasks/memory/README.md)
retains the existing names, funding, escrow, mapping and death rules.

This first implementation gives one process the writable mapping and any accepted
peers read-only, non-executable mappings. Readers can forward read authority, but
cannot promote it to write authority. Two independently owned buffers make a
duplex connection. It does not implement arbitrary multi-writer mappings or
transfer of the sole writer. Sharing is irreversible, even after all readers
leave; a shared object cannot become a frozen image or executable.

Producer release or death removes its mapping, not accepted readers. Readers can
unmap/remap and retain the last bytes. The original sponsor remains charged until
all rights, mappings and queued grants are gone. Mapping slots and names remain
caller-local. Numeric handle bytes do not install a grant.

## Publication protocol

[mailbox.rs](../../mailbox.rs) owns a deliberately small platform protocol:
one in-flight message in each direction, at most 256 bytes. Each buffer contains
a published sequence, acknowledgement of the opposite direction, payload length
and payload words. The writer publishes the sequence last. A consumer copies into
private storage before acknowledging. The producer waits for that acknowledgement
before overwriting its slot; full returns `WOULD_BLOCK`, never silent overwrite.
Sequence exhaustion fails instead of wrapping. A too-small destination does not
acknowledge the message.

The [machine access leaf](../../arch/shared.rs) uses aligned load/store
instructions with acquire/release ordering and compiler memory barriers. No Rust
slice, reference, enum or pointer is constructed from peer-owned contents. Length
and sequence checks use captured local values. A dishonest peer can still supply
inconsistent or nonsensical payloads; a repeated sequence check is not a proof of
immutability. Applications validate the private copied message before acting.
Acquire/release instructions exist for both ISAs, but the guest evidence is
single-CPU and does not validate an SMP kernel.

Notifications are independent hints: publish before signalling, check/drain state
before parking, and recheck after every wake. Several signals may collapse. A peer
that only receives should signal after acknowledgement to wake a blocked producer.
The mailbox does not own a notification or an event loop. Process death need not
signal a surviving notification; service owners still need their explicit
lifecycle/deadline path. Timeout is not acknowledgement or cancellation and must
not authorize overwriting an unconsumed slot or retrying an ambiguous mutation.

## Evidence and scope

[The channels proof](../../../../../distribution/lab/channels/shared/shared.rs)
exchanges 128 messages of varying sizes over real mappings and notifications.
It checks foreign handles, wrong typed acceptance, invalid copyout, frozen-image
and executable rejection, exact RO/NX faults, malformed headers, backpressure,
unmap/remap, death, replacement-account funding, queued discard and final refunds.
ARM64 runs 32 full-size exchanges with the same mailbox and checks exact session
frame/heap reclamation. Kernel host tests cover irreversible mode, typed escrow,
forwarding after creator death, name reuse prevention and quotas.

This is an experimental transport proof, not an ordinary service migration.
Connection setup still uses copied grant messages. F5 services and Omega host
bindings still use their existing transport; service request correlation,
cancellation, attachments and fair multiplexing are the next integration work.
