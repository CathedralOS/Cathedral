# Chapter 02: Telemetry & Update Feedback

> The OS reports on itself to its vendor — bound by the same capability model it imposes on everyone else, so system-level observation is never ambient.

## The Legacy Model

Legacy telemetry is structurally hard to trust. It ships as an opaque, privileged channel the user cannot inspect, attenuate, or fully disable; "off" often means "less," and nobody outside the vendor can verify what crosses the wire. Crash reporters scoop up memory that may contain secrets. "Anonymized" analytics are routinely re-identifiable. An organization gets a coarse on/off knob, not control. The structural problem is that telemetry runs *outside* the OS's own authority model: the vendor is an unmodeled super-principal with ambient reach. That is the same ambient-authority problem Cathedral rejects elsewhere, here applied to the vendor.

## The Cathedral Model

An OS needs some telemetry — crash reports, performance data, upgrade health, security signals — but it must **not destroy trust**. The domains:

- crash reports and performance telemetry
- upgrade health and rollout signals ([[updates_and_hot_swap]])
- security events and capability-usage summaries
- privacy-preserving analytics (possibly differential privacy)
- organization/tenant-controlled telemetry policy ([[multi_user_and_org_control]])
- user-visible telemetry policy and local-first diagnostics

Because Cathedral already produces **structured events** ([[observability_and_introspection]]), telemetry can be far cleaner than the log-scraping of legacy OSes: a telemetry stream is a *defined projection* of the event graph, not unstructured log text. Most diagnostics can stay **local-first** — answerable on-device — with only explicit, attenuated summaries leaving.

**Critical principle:** the capability model applies to the **OS vendor too**. The vendor is a principal in the authority graph like any app. A telemetry upload is a network flow under a held, attenuated, revocable capability, visible in the same introspection surface. If system- and OS-level observation is not itself bound by the capability model, the proofs elsewhere are meaningless — the failure mode the vision explicitly forbids ([[vision_and_non_goals]] is wrong without this chapter; see also the vendor-principal note in [[capability_model]]).

## Concerns & Design Space

- **Vendor as principal.** The telemetry collector holds named, attenuated capabilities; its flows appear in the authority graph and are revocable by the device owner or org admin.
- **Local-first by default.** Compute the answer on-device; export aggregates, not raw events, unless a specific capability is granted.
- **Crash report hygiene.** Crash payloads must be scrubbed of secrets and capability material by construction, not by best-effort redaction ([[secrets_and_keys]]).
- **Privacy-preserving analytics.** Whether differential privacy or k-anonymity is worth its accuracy cost, and where aggregation happens.
- **Organization control.** An organization or tenant sets telemetry policy as enforced, attested policy, not a checkbox ([[multi_user_and_org_control]]).
- **User visibility.** The user can *see* exactly what telemetry is defined to leave and follow its authority path, like any other flow.

## Key Questions

- What is the default — opt-in, opt-out, or tiered — and is the default itself attested and inspectable?
- Which diagnostics are answerable purely locally, and what is the minimal exported summary for the rest?
- How are crash payloads proven free of secrets and capability material?
- Can the user/org *revoke* the vendor's telemetry capability and keep a working, updatable system?

## Omega Leverage

- The **vendor is a principal** in the same authority graph; its telemetry flows obey capabilities, effects, and boundaries like any component ([capabilities & boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- **Structured causal events** mean a telemetry stream is a typed projection of the event graph, not parsed text.
- **Effects** bound the telemetry component (e.g. read-summaries + network) so its ceiling is auditable.
- **Attenuation / leasing** make telemetry grants narrow and expiring by default ([[capability_lifecycle]]).
- What Omega may need to grow: standard differential-privacy / aggregation combinators if privacy-preserving export becomes a first-class requirement.

## Open Questions

- Differential privacy has a real utility cost; is it worth it, or is local-first aggregation enough for the signals the vendor actually needs?
- If the vendor's telemetry capability is fully revocable, what is the minimum telemetry required to keep *updates* safe, and is that floor coercive?

## Related
- [[observability_and_introspection]] — the event graph telemetry projects from.
- [[data_model_and_privacy]] — what may leave the device and how.
- [[capability_model]] — the vendor is a principal, not an exception.
- [[multi_user_and_org_control]] — organization/tenant-controlled telemetry policy.
- [[updates_and_hot_swap]] — upgrade health as a telemetry signal.
