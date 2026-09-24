# Chapter 02: Telemetry & Update Feedback

> The OS reports on itself to its vendor under the same capability model it imposes on everyone else, so system-level observation is never ambient.

## The Legacy Model

Legacy telemetry is structurally hard to trust. It ships as an opaque, privileged channel the user cannot inspect, attenuate, or fully disable. "Off" often means "less", and nobody outside the vendor can verify what crosses the wire. Crash reporters scoop up memory that may contain secrets. "Anonymized" analytics are routinely re-identifiable. An organization gets a coarse on/off knob, not control. The structural problem is that telemetry runs outside the OS's own authority model: the vendor is an unmodeled super-principal with ambient reach. That is the same ambient-authority problem Cathedral rejects elsewhere, applied to the vendor.

## The Cathedral Model

An OS needs some telemetry, such as crash reports, performance data, upgrade health, and security signals, but it must not destroy trust. The domains:

- crash reports and performance telemetry
- upgrade health and rollout signals ([[updates_and_hot_swap]])
- security events and capability-usage summaries
- privacy-preserving analytics (possibly differential privacy)
- organization/tenant-controlled telemetry policy ([[multi_user_and_org_control]])
- user-visible telemetry policy and local-first diagnostics

Because Cathedral already produces structured events ([[observability_and_introspection]]), telemetry is far cleaner than the log-scraping of legacy OSes. A telemetry stream is a defined projection of the event graph, not unstructured log text. Most diagnostics stay **local-first**, answerable on-device, and only attenuated summaries leave, under a grant.

The principle that carries the chapter is that the capability model applies to the OS vendor too. The vendor is a principal in the authority graph like any app. A telemetry upload is a network flow under a held, attenuated, revocable capability, visible in the same introspection surface. If system-level and OS-level observation are not bound by the capability model, the proofs elsewhere are meaningless. That is the failure mode the vision forbids: [[vision_and_non_goals]] is wrong without this chapter, and [[capability_model]] carries the vendor-principal note.

## The decided mechanism

### Telemetry is not a primitive: it is two capabilities already in the system

"Telemetry" is not a special channel. It is gather plus send: the observability capability that introspects the event graph ([[observability_and_introspection]]) composed with a network capability for the flow that carries it out. So the OS vendor's telemetry collector is an ordinary app holding an observe capability and a network capability. It is revocable, attenuable, and visible in the authority graph like any other principal. The entire content of the principle is that the vendor's collector is not exempt from the capability model. There is no new mechanism to invent.

That composition covers the rest:

- **Local-first is the zero default.** Most diagnostics are answerable on-device, since your event graph is local and full-fidelity. With no telemetry capability, nothing leaves. What the vendor needs is fleet-level aggregates (is this update crashing across many machines), so the export is aggregates, not raw events.
- **Revocable and still updatable, not coercive.** You can cut the vendor's telemetry capability and the system stays updatable, because updates are fetch plus verify plus hot-swap ([[package_system]]) and need no vendor observation of you. What telemetry buys the vendor is safer staged rollout: catching a bad update on canary machines. The update-safety floor is therefore a fleet aggregate, not an individual requirement. Any one machine opts out and still updates, relying on other machines' canary signal, so opting out is not coerced.
- **Differential privacy is an opt-in.** It applies only to exports where cross-user aggregation could re-identify. On-device aggregation covers most signals without it.

### Crash reports and core dumps are a separate, harder problem, still open

A crash dump is not telemetry and is not solved by "it's a capability". A core dump is a raw runtime memory snapshot taken by the OS bypassing the type system, so a static `Secret<T>` discipline does not make it secret-free. A secret decrypted for use is plaintext in a buffer at crash time regardless of its static type. What is known, grounded in what security-conscious systems do:

- **Known secrets** (key material, credential buffers) can be excluded by memory region: a no-dump region the dump mechanism skips (Linux `MADV_DONTDUMP`, mlock-excluded, enclave-held). This is the real "by construction", but only for statically known secret regions.
- **Incidental sensitive data** (user data in a working buffer at crash time) is still in a raw dump, and no type or region fixes that. Mitigations are structured-not-raw capture (Breakpad/Crashpad-style: stack plus typed state, not full memory, which is a fidelity trade-off) and local-first handling (the dump stays on-device and only aggregates leave).
- **`reproduce-don't-dump` is a native-Omega benefit, not a general one.** For a native app, replay needs no memory dump at all. But replay is Cathedral plus Omega. Omega guarantees the app is deterministic given its declared effects, with no hidden nondeterminism, so the app is constrained rather than cooperating, and Cathedral records and re-serves those effect providers. For foreign or walled code that guarantee is gone (internal thread races, raw `rdtsc`), so clean replay is not guaranteed and needs rr-style heavyweight capture. Foreign code is exactly where crashes with secrets in memory are most likely. So the crash-dump secret-safety residual is concentrated in foreign code, the worst place for it.

This area is open and needs real prior-art review (Crashpad/minidump scrubbing, no-dump-region practice, `zeroize`/`secrecy`). It is not claimed solved.

## Concerns & Design Space

- **Vendor as principal.** The telemetry collector holds named, attenuated capabilities. Its flows appear in the authority graph and are revocable by the device owner or org admin.
- **Local-first by default.** Compute the answer on-device. Export aggregates, not raw events, unless a specific capability is granted.
- **Crash report hygiene.** Crash payloads must be scrubbed of secrets and capability material by construction, not by best-effort redaction ([[secrets_and_keys]]).
- **Privacy-preserving analytics.** Whether differential privacy or k-anonymity is worth its accuracy cost, and where aggregation happens.
- **Organization control.** An organization or tenant sets telemetry policy as enforced, attested policy, not a checkbox ([[multi_user_and_org_control]]).
- **User visibility.** The user can see exactly what telemetry is defined to leave and follow its authority path, like any other flow.
- **Zero value.** A zero vendor telemetry capability is the inert null-object capability ([[omega_substrate]]). The collector holds no authority, so nothing leaves the device and the projection of the event graph is empty rather than the upload erroring. Local-first becomes the literal zero default, and granting telemetry is an attenuated step away from it.

## Key Questions

- What is the default? Local-first is the zero default: with no capability nothing leaves, and telemetry is an attenuated, inspectable grant over a typed projection of the event graph, not an opaque channel.
- What stays local and what is exported? Most diagnostics are local and full-fidelity. Only fleet-level aggregates leave.
- Can a machine revoke telemetry and stay updatable? Yes. Updates are fetch plus verify plus hot-swap and need no vendor observation, and the update-safety floor is a fleet aggregate, not an individual requirement.
- Can crash payloads be proven secret-free? Not with what is known. A raw core dump bypasses the type system, so `Secret<T>` does not make it safe, and incidental data in a raw dump is a residual concentrated in foreign code.

## Omega Leverage

- The vendor is a principal in the same authority graph. Its telemetry flows obey capabilities, reach, and boundaries like any component ([capabilities & boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- Structured causal events mean a telemetry stream is a typed projection of the event graph, not parsed text.
- Reach bounds the telemetry component (for example, read-summaries plus network) so its ceiling is auditable.
- Attenuation and leasing make telemetry grants narrow and expiring by default ([[capability_lifecycle]]).
- What Omega may need to grow: standard differential-privacy and aggregation combinators, if privacy-preserving export becomes a requirement in its own right.

## Open Questions

- Crash-report and core-dump secret-safety is the real open item. Known secrets are excluded by no-dump region. Incidental sensitive data in a raw dump is not, and it is concentrated in foreign code, where crashes are most likely and where `reproduce-don't-dump` (the native-Omega replay benefit, Cathedral plus Omega) does not cleanly apply. It needs grounding in Crashpad/minidump-scrubbing and no-dump-region practice before a Cathedral-specific answer.
- Whether differential privacy earns its utility cost for the re-identification-sensitive exports it is opt-in for. Local-first aggregation covers most signals without it.

## Related
- [[observability_and_introspection]] — the event graph telemetry projects from.
- [[data_model_and_privacy]] — what may leave the device and how.
- [[capability_model]] — the vendor is a principal, not an exception.
- [[multi_user_and_org_control]] — organization/tenant-controlled telemetry policy.
- [[updates_and_hot_swap]] — upgrade health as a telemetry signal.
