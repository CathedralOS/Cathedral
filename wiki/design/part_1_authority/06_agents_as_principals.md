# Chapter 06: Agents as Principals

> Autonomous software that acts for the user is a principal like any other: it holds a scoped, leased, revocable bundle of capabilities, and every action it takes is in the authority graph. The OS owns none of the AI; it governs the authority.

## The Legacy Model

No mainstream OS has a notion of an autonomous agent. An "AI assistant" today is just an app, which means it runs with one of two bad authority models. Either it has the user's full ambient power (on a desktop, it can read every file the user can), or it is sandboxed like any app and therefore cannot do the cross-app things an agent exists for. Neither models the thing that matters: the agent is acting on the user's behalf, and what it is allowed to do should be a delegated, scoped, time-bounded subset of the user's authority, not all of it and not a fixed app sandbox. Worse, the agent's inputs are untrusted (a web page, an email, a tool result), so an agent with broad authority and untrusted input is a confused deputy waiting to be steered. Prompt injection is not a model bug to be filtered away. It is an authority-design problem the OS layer below the model has to answer.

## The Cathedral Model

The OS owns none of the AI. The inference runtime and the model weights are userspace libraries, like the transport stack ([[networking]]) and the typed IPC layer ([[ipc_and_service_invocation]]). The accelerator (NPU or GPU) is a shared device, multiplexed, isolated, and budgeted by the driver and scheduler ([[driver_model]], [[scheduler_and_resources]]). There is no "AI subsystem" in the kernel. A model is a tool an agent runs, not a service the OS provides.

What the OS does own is the authority of the thing doing the acting.

- **An agent is a principal** ([[identity_and_principals]]). It holds a capability bundle delegated by the user, strictly narrower than the user's own authority, and ideally leased (time- or task-bounded) so it expires by default ([[capability_lifecycle]]). Every action it takes is an edge in the authority graph ([[capability_model]]) and a recorded event ([[audit_compliance_provenance]]). "What did the agent do, under whose authority, and why" is a query over the authority and event graphs.
- **Prompt injection is the confused-deputy problem, and least authority is the structural defense.** An agent can only misuse authority it holds. If it was handed a capability to one file via the picker ([[human_permission_ux]]) rather than ambient access to all files, a malicious instruction in its input cannot make it touch anything else, because there is no capability to abuse. Filtering the model's inputs helps. Bounding the agent's authority is what contains the blast radius.
- **Context is purpose-scoped data.** What a model is allowed to read to build its context (your files, contacts, messages) is a purpose-scoped capability use ([[data_model_and_privacy]]), recorded as such, not an ambient slurp. "This agent may read these contacts for the purpose of drafting this reply" is expressible and auditable.
- **High-authority actions are minted by a human gesture.** Spending money, sending a message, or deleting data requires the user's act of authority-minting: the action-confirm (WYSIWYS) gesture of [[human_permission_ux]], a confirmation that is the capability grant, issued by the human and outside the agent's reach.
- **Credentials are operation-capabilities the agent can use but never read.** An API key, token, or signing key is never handed to the agent as bytes. The agent holds a `Capability<Use…>` and the secret stays sealed, which is the operation-capability model of [[secrets_and_keys]]. A prompt-injected agent can be steered into using the credential, bounded by its attenuation (GET-only, one endpoint, rate-capped), but it cannot exfiltrate it, because it never holds the bytes. This is categorically stronger than labeling a held secret with information-flow control: there is nothing to leak. For a complex credential-using workflow, the stronger form is to ship a verified program to run next to the credential rather than expose each operation, a speculative extension described in [code-shipping capability](../../speculation/code_shipping_capability.md).

### The decided mechanism

The model is a tool and the agent is the principal, and that split is what contains injection. The model (weights plus inference) is a **pure function** with no authority and no identity, a userspace library on the budgeted accelerator. The **agent**, the loop that holds the capability bundle and acts, is the principal. Sub-agents it spawns are principals too, holding attenuations. "Prompt injection is an authority problem, not a model problem" is therefore structural: the model can be fully adversarial and still do nothing, because it holds no authority. Only the agent does, and the agent's every action is capability-checked. The model is never a principal.

Injection is contained, not eliminated. Least authority shrinks the blast radius to the held authority, never to zero, so a prompt-injected agent can be steered to misuse whatever it legitimately holds. The residual is the agent's standing authority. Minimize it (leased so it expires, volume-capped, monitored) and keep the high-stakes mint outside the agent. The human gesture that mints an irreversible action ([[human_permission_ux]] action-confirm) is unreachable by the agent, so even a fully hijacked agent cannot trigger it. The answer to "can least authority fully contain injection?" is no. It is the same confine-don't-eliminate ceiling.

The "dangerous sequence" is not an agent problem. It is general authority abuse, inherited. A sequence of individually safe calls that composes into harm (read-send ×1000 = exfiltration; 100 approved charges = drained) is the same abuse a malicious process commits, and nothing about it is agent-specific. It is handled by the general machinery of [[security_policy_and_sandboxing]]:

- **Containment.** Reads of scoped or synthetic data and egress to a scoped peer, so a malicious sequence cannot reach real harm.
- **Aggregate task-calibrated budgets.** ≤$X total per session, ≤N files per hour, calibrated to the declared task, so exceeding the budget is the anomaly signal.
- **Plan review for irreversible batches.** The agent declares its plan and the human approves the aggregate, not each call. This is the output-commit pattern applied to agent actions.
- **LLM-advised anomaly monitoring.**

The only agent overlay is to assume worst-case intent, because the model is steerable, and that changes nothing about the bounding.

Multi-principal collusion is contained at the boundary, not per principal. Two confined principals can launder authority across IPC: A holds the secret, B holds egress, and A pipes the bytes to B. The per-principal conjunction ceiling does not catch this. It is bounded by pushing the ceiling to the containment boundary. Collusion inside a Matrix is harmless because the Matrix does not hold real-sensitive ∧ real-egress, so nothing leaves. The boundary the ceiling applies to is the Matrix, not the individual agent. Where the flow stays within a proven graph, reach-ceiling composition can even prove the cross-component flow safe ([[security_policy_and_sandboxing]]).

An agent is a seated principal, and raw input control is being the user. "The user" is not an entity. It is whoever holds the input ([[identity_and_principals]], [[windowing_and_compositor]]). An agent therefore acts through a scoped capability bundle plus its own attributed, labeled cursor: a seat fed by virtual input, indistinguishable to the surface but labeled to the observer. It never gets raw input control. An agent handed the raw cursor and keyboard is unbounded by construction, because holding the input is being the user, which is fine if you meant it and catastrophic if you did not. Letting an agent drive is delegating an input capability, visible and revocable, and its labeled cursor makes the action concurrent and watchable rather than a hidden takeover. The hardware-rooted backstop is the **OS key**. It reclaims the physical-device-to-seat binding, routing real input exclusively to the operator's seat and severing the agent, and it is the one input a software-capability agent provably cannot forge. The strongest human-only consent gate is therefore an action confirmable only through the OS-attested physical seat.

## Concerns & Design Space

- **Agent identity and lifecycle.** An agent instance is a principal with a birth, a task, and a death. Its capability bundle is reclaimed when the task ends or the lease expires.
- **Bundle scoping.** What authority an agent gets by default (nothing ambient), and how a task widens it through attenuated grants rather than a standing role.
- **Sub-agents and delegation.** An agent that spawns helpers delegates attenuations of its own authority, never more. The delegation chain is in the graph.
- **Untrusted tool output.** A tool result fed back into the model is untrusted input. It must not be able to escalate the agent's authority, only inform its next, still capability-bounded, action.
- **Human-in-the-loop cost.** Where confirmation is required for a high-authority action versus where standing authority is acceptable, and how to keep prompts unspoofable ([[windowing_and_compositor]] trusted path) and not so frequent that the user clicks through blind.
- **Provenance of outputs.** Which model, which version, and which inputs produced an output or an action ([[audit_compliance_provenance]]).
- **Inference as a metered resource.** An agent's NPU use is budgeted and accounted like any resource ([[scheduler_and_resources]]).

## Key Questions

- What authority does an agent hold by default? None ambient; everything delegated.
- How is "the user approved this action" expressed as a capability mint rather than a dismissible prompt?
- How do you bound an agent that composes many tool calls, where no single call is dangerous but the sequence is?
- Is the model itself ever a principal, or always a tool the agent (the principal) invokes?

## Omega Leverage

- Capabilities as values are the agent's authority bundle: held, attenuable, leasable, revocable, with no ambient power ([Omega Chapter 19: Capabilities, Reach, And Boundaries](https://github.com/CathedralOS/Omega/blob/main/wiki/language_guide/chapter_19_capabilities_effects_boundaries.md)).
- Authority flow already reports what a component accepts, uses, stores, and acquires. For an agent that report is the safety story.
- Reach ceilings bound what an agent can reach regardless of what its model is told to do.
- The audit and event graph makes every agent action recorded and attributable by construction.
- Omega may need to grow purpose-tagged authority so "read for this purpose" is a checkable fact, not a convention ([[data_model_and_privacy]]).

## Open Questions

- Can prompt injection be fully contained by least authority, or is there residual risk whenever an agent holds any standing authority at all?
- How is an agent's current authority presented to a human comprehensibly, so consent is informed rather than reflexive? This is the legibility problem, and it is the main open residue of this chapter.
- Where is the line between an agent autonomous enough to be useful and bounded enough to be safe, and is that line per-task policy?

## Related
- [[capability_model]] — the agent's actions are edges in the authority graph.
- [[identity_and_principals]] — an agent is a principal type.
- [[capability_lifecycle]] — leased, attenuated, revocable agent authority.
- [[data_model_and_privacy]] — context as purpose-scoped data.
- [[human_permission_ux]] — high-authority actions minted by a human gesture.
- [[audit_compliance_provenance]] — every agent action recorded and attributable.
- [[scheduler_and_resources]] — the accelerator as a budgeted resource; inference is scheduled.
