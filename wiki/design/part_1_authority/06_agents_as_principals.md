# Chapter 06: Agents as Principals

> Autonomous software that acts for the user is a principal like any other: it holds a scoped, leased, revocable bundle of capabilities, and every action it takes is in the authority graph. The OS owns none of the AI; it governs the authority.

## The Legacy Model

No mainstream OS has a notion of an autonomous agent. An "AI assistant" today is just an app, which means it runs with one of two bad authority models. Either it has the user's full ambient power (on a desktop, it can read every file the user can), or it is sandboxed like any app and therefore cannot do the cross-app things an agent exists for. Neither models the thing that actually matters: the *agent is acting on the user's behalf*, and what it is allowed to do should be a delegated, scoped, time-bounded subset of the user's authority, not all of it and not a fixed app sandbox. Worse, the agent's inputs are untrusted (a web page, an email, a tool result), so an agent with broad authority and untrusted input is a confused deputy waiting to be steered. Prompt injection is not a model bug to be filtered away; it is an authority-design problem the OS layer below the model has to answer.

## The Cathedral Model

First, the boundary: **the OS owns none of the AI.** The inference runtime and the model weights are userspace libraries, exactly like the transport stack ([[networking]]) and the typed IPC layer ([[ipc_and_service_invocation]]). The accelerator (NPU/GPU) is a shared device, multiplexed, isolated, and budgeted by the driver and scheduler ([[driver_model]], [[scheduler_and_resources]]). There is no "AI subsystem" in the kernel. A model is a tool an agent runs, not a service the OS provides.

What the OS does own is the **authority** of the thing doing the acting:

- **An agent is a principal** ([[identity_and_principals]]). It holds a capability bundle delegated by the user, strictly narrower than the user's own authority, and ideally leased (time- or task-bounded) so it expires by default ([[capability_lifecycle]]). Every action it takes is an edge in the authority graph ([[capability_model]]) and a recorded event ([[audit_compliance_provenance]]). "What did the agent do, under whose authority, and why" is a query over the authority and event graphs.
- **Prompt injection is the confused-deputy problem, and least authority is the structural defense.** An agent can only misuse authority it holds. If it was handed a capability to *one* file via the picker ([[human_permission_ux]]) rather than ambient access to all files, a malicious instruction in its input cannot make it touch anything else, because there is no capability to abuse. Filtering the model's inputs helps; bounding the agent's authority is what actually contains the blast radius.
- **Context is purpose-scoped data.** What a model is allowed to read to build its context (your files, contacts, messages) is a purpose-scoped capability use ([[data_model_and_privacy]]), recorded as such, not an ambient slurp. "This agent may read these contacts for the purpose of drafting this reply" is expressible and auditable.
- **High-authority actions are minted by a human gesture.** Spending money, sending a message, or deleting data should require the user's act of authority-minting: a confirmation that *is* the capability grant ([[human_permission_ux]]), issued by the human and outside the agent's reach.
- **Credentials are operation-capabilities the agent can *use* but never *read*.** An API key, token, or signing key is never handed to the agent as bytes; the agent holds a `Capability<Use…>` and the secret stays sealed (the operation-capability model, [[secrets_and_keys]]). So a prompt-injected agent can be steered into *using* the credential — bounded by its attenuation (e.g. GET-only, one endpoint, rate-capped) — but **cannot exfiltrate it, because it never holds the bytes**. This is categorically stronger than labeling a held secret (information-flow control): there is nothing to leak. *(For a complex credential-using workflow, the stronger form is to ship a verified program to run next to the credential rather than expose each operation — a speculative extension, [code-shipping capability](../../speculation/code_shipping_capability.md).)*

## Concerns & Design Space

- **Agent identity & lifecycle.** An agent instance is a principal with a birth, a task, and a death; its capability bundle is reclaimed when the task ends or the lease expires.
- **Bundle scoping.** What authority an agent gets by default (nothing ambient), and how a task widens it through explicit, attenuated grants rather than a standing role.
- **Sub-agents & delegation.** An agent that spawns helpers delegates *attenuations* of its own authority, never more; the delegation chain is in the graph.
- **Untrusted tool output.** A tool result fed back into the model is untrusted input; it must not be able to escalate the agent's authority, only inform its next (still capability-bounded) action.
- **Human-in-the-loop cost.** Where confirmation is required for a high-authority action vs. where standing authority is acceptable, and how to keep prompts un-spoofable ([[windowing_and_compositor]] trusted path) and not so frequent the user clicks through blind.
- **Provenance of outputs.** Which model, which version, which inputs produced an output or an action ([[audit_compliance_provenance]]).
- **Inference as a metered resource.** An agent's NPU use is budgeted and accounted like any resource ([[scheduler_and_resources]]).

## Key Questions

- What authority does an agent hold by default? (The answer should be: none ambient, everything delegated.)
- How is "the user approved this action" expressed as a capability mint rather than a dismissible prompt?
- How do you bound an agent that composes many tool calls, where no single call is dangerous but the sequence is?
- Is the model itself ever a principal, or always just a tool the agent (the principal) invokes?

## Omega Leverage

- **Capabilities as values** are the agent's authority bundle: held, attenuable, leasable, revocable, with no ambient power ([../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md](../../../../Omega/wiki/language_guide/chapter_18_capabilities_effects_boundaries.md)).
- **Authority flow** already reports what a component accepts, uses, stores, and acquires; for an agent that report *is* the safety story.
- **Effects ceilings** bound what an agent can reach regardless of what its model is told to do.
- **The audit/event graph** makes every agent action recorded and attributable by construction.
- Omega may need to grow **purpose-tagged authority** so "read for this purpose" is a checkable fact, not a convention ([[data_model_and_privacy]]).

## Open Questions

- Can prompt injection be *fully* contained by least authority, or is there residual risk whenever an agent holds any standing authority at all?
- How is an agent's current authority presented to a human comprehensibly, so consent is informed rather than reflexive?
- Where is the line between an agent autonomous enough to be useful and bounded enough to be safe, and is that line per-task policy?

## Related
- [[capability_model]] — the agent's actions are edges in the authority graph.
- [[identity_and_principals]] — an agent is a principal type.
- [[capability_lifecycle]] — leased, attenuated, revocable agent authority.
- [[data_model_and_privacy]] — context as purpose-scoped data.
- [[human_permission_ux]] — high-authority actions minted by a human gesture.
- [[audit_compliance_provenance]] — every agent action recorded and attributable.
- [[scheduler_and_resources]] — the accelerator as a budgeted resource; inference is scheduled.
