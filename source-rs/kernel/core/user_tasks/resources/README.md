# Resource subdivision

Start at [resource entrance](mod.rs): snapshot local capacity and usage,
validate every resource family, then commit a conserving owner/child transfer.
[The syscall entrance](../syscalls/resources.rs) resolves the
child through an owned, idle launch grant before reading its requested limits.
There is no arbitrary account selector or global free-capacity query.

`RESOURCE_ASSIGN` sets the child's private-page, accepted-reader-page and channel
allowances. Increases consume the owner's unused allowance; decreases return it.
All three change atomically. The child must be idle, with its previous outcome
collected, so admission can prepare the appropriate mapping tables. Each matrix
can repeat this operation on its own children. Lending a launch account lends its
reserved capacity too; it is removed from the lender's local grant enumeration.
The resulting subtree envelope is the sum of its exclusively held reservations,
not an additional independent allowance at every depth.

[IPC storage](../../ipc/budget/mod.rs) is physically backed by its fixed channel arena.
Page quotas bound runtime objects and accepted mappings. A separate
[physical reservation](physical/README.md) now backs executable pages, stacks,
page tables and runtime buffers from an eagerly reserved frame bank. Page/IPC
quota changes alone do not transfer that physical funding. Dynamic reservation
commits both together; idle launches also support explicit physical subdivision.
Per-account lab maxima remain 16 private pages, 16 reader pages and 16 channels,
with separate object/handle limits. This is not a general memory-pressure policy.

[Page objects](../memory/regions/mod.rs) now retain their original backing sponsor through
producer death, just as channels already retain their creator account. An accepted
reader can pin that allowance; neither restart nor attempted budget withdrawal
removes its live mapping. Final release/death ends the charge. Moving custody does
not transfer funding. Reservations themselves persist with launch accounts and
return through existing loan ownership; reclaiming unused capacity is explicit.
Production sponsorship/adoption choices remain in OWNER_QUESTIONS.md.

[Host tests](tests.rs) cover nested conservation, atomic failure, an independent
sibling, pinned endpoints and accepted pages after creator death. The ordinary
[channels fixture](../../../../distribution/lab/channels/resources/mod.rs) additionally
subdivides real page/channel budgets twice, exhausts both descendants, checks bad
copy addresses and authority, cancels the subtree and recovers unused capacity.
Run `cargo test --locked` and `python tools/boot-harness-rs/run.py --ipc-test --release`.
