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

## The decided mechanism

### Telemetry is not a primitive — it is two capabilities already in the system

"Telemetry" is not a special channel; it is **gather + send**: the **observability capability** (introspect the event graph — [[observability_and_introspection]]) composed with a **network capability** (the flow that carries it out). So the OS vendor's telemetry collector is *an ordinary app holding an observe-cap + a network-cap* — revocable, attenuable, and visible in the authority graph like any other principal. The entire content of the principle is that **the vendor's collector is not exempt from the capability model**; there is no new mechanism to invent.

That composition settles the rest:
- **Local-first is the zero default.** Most diagnostics are answerable on-device (your event graph is local, full-fidelity); with no telemetry cap, nothing leaves. What the vendor genuinely needs is **fleet-level aggregates** (is this update crashing across many machines), so the export is aggregates, not raw events.
- **Revocable and still updatable, not coercive.** You can cut the vendor's telemetry cap; the system stays updatable, because updates are fetch + verify + hot-swap ([[package_system]]) and need no vendor observation of you. What telemetry buys the *vendor* is safer staged rollout (catching a bad update on canary machines). The update-safety floor is therefore a **fleet aggregate, not an individual requirement** — any one machine opts out and still updates, relying on *others'* canary signal — so it is not coercive.
- **Differential privacy is an opt-in**, only for exports where *cross-user* aggregation could re-identify; on-device aggregation covers most signals without it.

### Crash reports and core dumps are a *separate, harder* problem — flagged open

A crash dump is not telemetry and is not solved by "it's a capability." A core dump is a **raw runtime memory snapshot** taken by the OS *bypassing the type system*, so a static `Secret<T>` discipline does **not** make it secret-free — a secret decrypted for use is plaintext in a buffer at crash time regardless of its static type. The honest state, grounded in what security-conscious systems do:

- **Known secrets** (key material, credential buffers) can be excluded **by memory region** — a no-dump region the dump mechanism skips (Linux `MADV_DONTDUMP`, mlock-excluded, enclave-held). This is the real "by construction," but only for *statically-known* secret regions.
- **Incidental sensitive data** (user data in a working buffer at crash time) is still in a raw dump, and no type or region fixes that. Mitigations are **structured-not-raw capture** (Breakpad/Crashpad-style: stack + typed state, not full memory — a fidelity tradeoff) and **local-first** (the dump stays on-device; only aggregates leave).
- **`reproduce-don't-dump` is a *native-Omega* benefit, not general.** For a native app, replay needs no memory dump at all — but replay is **Cathedral + Omega**: Omega *guarantees* the app is deterministic-given-its-declared-effects (no hidden nondeterminism — the app is *constrained*, not cooperating), and Cathedral records/re-serves those effect providers. For **foreign/walled code** that guarantee is gone (internal thread races, raw `rdtsc`), so clean replay isn't guaranteed and needs rr-style heavyweight capture — and foreign code is exactly where crashes-with-secrets-in-memory are most likely. So the crash-dump-secret-safety residual is **concentrated in foreign code**, the worst place for it.

This area is **open, needing real prior-art review** (Crashpad/minidump scrubbing, no-dump-region practice, `zeroize`/`secrecy`), and is deliberately *not* claimed solved.

## Concerns & Design Space

- **Vendor as principal.** The telemetry collector holds named, attenuated capabilities; its flows appear in the authority graph and are revocable by the device owner or org admin.
- **Local-first by default.** Compute the answer on-device; export aggregates, not raw events, unless a specific capability is granted.
- **Crash report hygiene.** Crash payloads must be scrubbed of secrets and capability material by construction, not by best-effort redaction ([[secrets_and_keys]]).
- **Privacy-preserving analytics.** Whether differential privacy or k-anonymity is worth its accuracy cost, and where aggregation happens.
- **Organization control.** An organization or tenant sets telemetry policy as enforced, attested policy, not a checkbox ([[multi_user_and_org_control]]).
- **User visibility.** The user can *see* exactly what telemetry is defined to leave and follow its authority path, like any other flow.
- **Zero value.** A zero vendor telemetry capability is the inert null-object capability ([[omega_substrate]]): the collector holds no authority, so nothing leaves the device and the projection of the event graph is empty rather than the upload erroring. Local-first becomes the literal zero default, and granting telemetry is an explicit, attenuated step away from it.

## Key Questions

- **Default — resolved:** local-first is the zero default (no cap → nothing leaves); telemetry is an explicit, attenuated, inspectable grant (a typed projection of the event graph, not an opaque channel).
- **Local vs exported — resolved:** most diagnostics are local + full-fidelity; only fleet-level aggregates leave.
- **Revoke and stay updatable — resolved:** yes; updates are fetch + verify + hot-swap and need no vendor observation; the update-safety floor is a fleet aggregate, not an individual requirement, so it is not coercive.
- **Crash payloads proven secret-free — NOT resolved; flagged open** (see "The decided mechanism"): a raw core dump bypasses the type system, so `Secret<T>` does not make it safe; known secrets can be no-dump-region-excluded, but incidental data in a raw dump is a real residual concentrated in foreign code — needs prior-art review, not a wishcasted claim.

## Omega Leverage

- The **vendor is a principal** in the same authority graph; its telemetry flows obey capabilities, effects, and boundaries like any component ([capabilities & boundaries](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- **Structured causal events** mean a telemetry stream is a typed projection of the event graph, not parsed text.
- **Effects** bound the telemetry component (e.g. read-summaries + network) so its ceiling is auditable.
- **Attenuation / leasing** make telemetry grants narrow and expiring by default ([[capability_lifecycle]]).
- What Omega may need to grow: standard differential-privacy / aggregation combinators if privacy-preserving export becomes a first-class requirement.

## Open Questions

- **Crash-report / core-dump secret-safety — the real open item.** Known secrets exclude by no-dump region; incidental sensitive data in a raw dump does not, and is concentrated in **foreign code** (where crashes are most likely and where `reproduce-don't-dump` — a native-Omega replay benefit, Cathedral+Omega — does not cleanly apply). Needs grounding in Crashpad/minidump-scrubbing / no-dump-region practice before a Cathedral-specific answer.
- **Differential privacy** stays an opt-in for re-identification-sensitive exports; local-first aggregation covers most signals without its utility cost.

## Related
- [[observability_and_introspection]] — the event graph telemetry projects from.
- [[data_model_and_privacy]] — what may leave the device and how.
- [[capability_model]] — the vendor is a principal, not an exception.
- [[multi_user_and_org_control]] — organization/tenant-controlled telemetry policy.
- [[updates_and_hot_swap]] — upgrade health as a telemetry signal.
