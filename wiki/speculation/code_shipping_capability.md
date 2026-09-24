# Speculative: Code-Shipping Capability

> **Status: SPECULATIVE.** A forward-looking application, not committed design. Captures the "execute verified code outside the Matrix" idea (2026-06-19). Sits next to [future_browser.md](future_browser.md); both ship verified Omega IR to run elsewhere, bounded and checked.

## The idea

Code-shipping moves the computation to the resource instead of the resource to the computation. Code originates inside a Matrix but executes outside it, in a context holding capabilities the inside should not, such as a credential. The inside ships verified Omega IR. The outside re-checks it (PCC) for capability-conformance and bounded termination, runs it bounded by what the outside grants, and returns a result. The inside never sees the outside's secret.

This is eBPF generalized. eBPF ships verifier-checked code from userspace into the kernel, to run with access userspace lacks. Code-shipping ships verifier-checked Omega IR from one world into another. eBPF proves the pattern works: Linux's networking, observability, and security stacks run on it.

## Why Cathedral is well-positioned

Code-shipping needs no new primitive. It is the provider-capability pattern, a `Capability<ExecuteIn<X>>` provider that accepts IR ([[capability_model]]), built on two pieces Cathedral already has:
- **Verified Omega IR** as the shippable artifact, the same artifact as the future browser's.
- **PCC** (proof-carrying code). The receiver re-verifies that the IR respects its declared capabilities and is bounded before running it. This is the narrow case where PCC earns its keep: admitting dynamic, foreign-origin code into a more-trusted context. The totality and bounded-computation discipline comes with it: no unbounded loop in someone else's context.

## The payoff (stronger agent-credential model)

Instead of exposing N redeemable operation-capabilities, an agent ships one verified program that performs a whole credential-using workflow and returns the result. The program never holds the credential, is bounded by the verifier, and is revocable. The computation moves to the credential ([[agents_as_principals]]).

## Scope and limits

- **Dynamic code only.** The value is in late-bound code: an agent composes a workflow on the fly, a user writes a query, a plugin arrives. If the code is static, pre-known, and you own both ends, just deploy a service outside.
- **Validation is the hard part.** It is bounded by the verifier, which is the small checker already in the TCB, plus the bounded-computation requirement. eBPF's verifier is restrictive because verifying arbitrary code is hard. Omega's proof-carrying is the stronger version.
- **Not an OS primitive.** It is a pattern or library over verified IR, PCC, the provider-capability, and IPC.

## The unification

"Ship verified Omega IR to run in a bounded capability context" is one mechanism with many applications: a web tab ([future_browser.md](future_browser.md)), an agent's credential-workflow, eBPF-into-a-service. The recurrence is a sign the architecture is coherent.

## Related
- [future_browser.md](future_browser.md) — the sibling application (ship verified IR to render/run).
- Cathedral [[agents_as_principals]] (credentials as operation-capabilities), [[capability_model]] (provider-capability / recursive provider).
- Omega `verified_gated_ml_optimizer.md`, `totality_and_bounded_computation.md` (the verified-IR + bounded-code substrate).
