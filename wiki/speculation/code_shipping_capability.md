# Speculative: Code-Shipping Capability

> **Status: SPECULATIVE — a forward-looking application, not committed design.** Captures the "execute verified code outside the Matrix" idea (2026-06-19). Sits next to [future_browser.md](future_browser.md); both are "ship verified Omega IR to run elsewhere, bounded and checked."

## The idea

Move the *computation* to the resource instead of the resource to the computation. Code originates **inside** a Matrix but executes **outside** it — in a context that holds capabilities (e.g. a credential) the inside doesn't and shouldn't. The inside ships **verified Omega IR**; the outside re-checks it (PCC) for capability-conformance and bounded termination, runs it bounded by what the outside grants, and returns a result. The inside never sees the outside's secret.

This is **eBPF generalized**: eBPF ships verifier-checked code from userspace into the kernel, to run with access userspace lacks; this ships verifier-checked Omega IR from one world into another. eBPF is the proof the pattern works — Linux's networking / observability / security stack runs on it.

## Why Cathedral is well-positioned

It needs **no new primitive** — it is the **provider-capability** pattern (a `Capability<ExecuteIn<X>>` provider that accepts IR, [[capability_model]]) built on two pieces Cathedral already has:
- **Verified Omega IR** as the shippable artifact (the same artifact as the future browser).
- **PCC** (proof-carrying code): the receiver re-verifies the IR respects its declared capabilities and is bounded *before* running it — and this is precisely the narrow case where PCC earns its keep (admit *dynamic, foreign-origin* code into a *more-trusted* context). Plus the totality / bounded-computation discipline (no unbounded loop in someone else's context).

## The payoff (stronger agent-credential model)

Instead of exposing N redeemable operation-capabilities, an agent ships **one verified program** that performs a whole credential-using workflow and returns the result — never holding the credential, bounded by the verifier, revocable. *Move the computation to the credential.* ([[agents_as_principals]].)

## Scope and honesty

- **Dynamic code only.** The value is *late-bound* code — an agent composes a workflow on the fly, a user writes a query, a plugin. If the code is *static / pre-known* and you own both ends, just deploy a service outside; no runtime code-shipping needed.
- **Validation is the hard part**, bounded by the verifier (= the small checker already in the TCB) plus the bounded-computation requirement. eBPF's verifier is restrictive *because* verifying arbitrary code is hard; Omega's proof-carrying is the stronger version.
- **Not an OS primitive** — a pattern/library over (verified IR + PCC + provider-capability + IPC).

## The unification

"Ship verified Omega IR to run in a bounded capability context" is **one mechanism, many applications**: a web tab ([future_browser.md](future_browser.md)), an agent's credential-workflow, eBPF-into-a-service. The recurrence is the sign the architecture is coherent.

## Related
- [future_browser.md](future_browser.md) — the sibling application (ship verified IR to render/run).
- Cathedral [[agents_as_principals]] (credentials as operation-capabilities), [[capability_model]] (provider-capability / recursive provider).
- Omega `verified_gated_ml_optimizer.md`, `totality_and_bounded_computation.md` (the verified-IR + bounded-code substrate).
