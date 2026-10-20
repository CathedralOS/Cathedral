# Resource subdivision

Start at [resource entrance](resources.rs): snapshot local capacity and usage,
validate every resource family, then commit a conserving owner/child transfer.
[The syscall entrance](../syscalls/resources.rs) resolves the
child through an owned launch grant before reading its requested limits.
There is no arbitrary account selector or global free-capacity query.

`RESOURCE_GROW` accepts an owned launch, including a live child, and five
increments: private pages, reader pages, channels, unused descendant slots and
physical frames. It validates all five before publishing any ledger change.
Runtime mapping tables already cover the fixed mapping arena; this operation
changes allowances, never mappings or maximum object/handle counts. Existing
charges remain intact. The owner spends only its unused capacity. Process slots
must also lie above the child's ordering boundary; `NO_PROCESS_CAPACITY`
distinguishes that refusal without exposing account numbers. There is no live
withdrawal, global capacity query or implicit permission to request more.

The platform `matrix::FUND` packet asks an immediate owner to use this mechanism.
Stock policy and recursive forwarding live in
[children/funding.rs](../../../../distribution/matrix/service/children/funding.rs).
Each edge is atomic; a multi-hop request is not a distributed transaction.

`RESOURCE_ASSIGN` sets the child's private-page, accepted-reader-page and channel
allowances. Increases consume the owner's unused allowance; decreases return it.
All three change atomically. This absolute reassignment requires an idle child,
with its previous outcome collected; live growth uses `RESOURCE_GROW`. Each matrix
can repeat this operation on its own children. Lending a launch account lends its
reserved capacity too; it is removed from the lender's local grant enumeration.
The resulting subtree envelope is the sum of its exclusively held reservations,
not an additional independent allowance at every depth.

[IPC storage](../../ipc/budget/budget.rs) is physically backed by its fixed channel arena.
Page quotas bound runtime objects and accepted mappings. A separate
[physical reservation](physical/README.md) now backs executable pages, stacks,
page tables and runtime buffers from an eagerly reserved frame bank. Page/IPC
quota changes alone do not transfer that physical funding. Dynamic reservation
commits both together; idle launches also support explicit physical subdivision.
Per-account maxima are defined in `kernel/contracts/memory.rs` and `user.rs`,
with separate object/handle limits. This is not a general memory-pressure policy.

[Page objects](../memory/regions/regions.rs) now retain their original backing sponsor through
producer death, just as channels already retain their creator account. An accepted
reader can pin that allowance; neither restart nor attempted budget withdrawal
removes its live mapping. Final release/death ends the charge. Moving custody does
not transfer funding. Reservations themselves persist with launch accounts and
return through existing loan ownership; reclaiming unused capacity is explicit.
Production sponsorship/adoption choices remain in OWNER_QUESTIONS.md.

[Host tests](tests.rs) cover nested conservation, atomic failure, an independent
sibling, pinned endpoints and accepted pages after creator death. The ordinary
[channels fixture](../../../../distribution/lab/channels/resources/resources.rs) additionally
subdivides real page/channel budgets twice, exhausts both descendants, checks bad
copy addresses and authority, grows a running child's exhausted page allowance,
cancels the subtree and recovers unused capacity.
Run `cargo test --locked` and `python tools/boot-harness-rs/run.py --ipc-test --release`.
