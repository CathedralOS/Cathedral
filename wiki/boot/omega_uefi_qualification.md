# Omega UEFI boot qualification

Cathedral's Omega kernel loaded its own segment table, checked the CPU's
selection, and restored the previous table after leaving UEFI. A freshly
compiled image passed all sixteen ordered QEMU/OVMF serial checks through
`GDT checked`. The earlier firmware-exit, retained-memory and mapping checks
also passed.

The kernel retains its roots and idles after this operation. Permanent segment
and task-state tables, installed interrupt handlers and stacks, repeated timer
delivery and an ordinary-code wake remain unfinished. These observations do
not establish the normal desktop or real-hardware support.

## Segment-table boot check, 2026-11-04 UTC

Ordinary package update checked all nine packages and published the lock in
**520.426 seconds**, with no source differences or new capability decisions.
The subsequent fresh build and boot passed in **392.162 seconds**, including
30 seconds of guest observation. Compiler timing was **361.970 seconds**;
these are host workflow durations, not guest boot times.

- Compiler source: `cee7ad87d0034e60fde2b54584bf56f4d12fcc9b`, submitted in
  [Omega PR #1793](https://github.com/CathedralOS/Omega/pull/1793).
  The compiler repair subsequently landed on Omega main at
  `841c1847f334e54278c4e83df8fba6b9c594baa9`, after focused integration checks
  and required CI passed. This boot used the earlier candidate above; its
  producer manifest retains that actual build identity.
- Compiler SHA-256:
  `50e0c2053c632504d4fcb35360f8ecc20ef5b5b2cdbd5a7ae3ae7cb5e226d7b2`,
  verified optimization level `1`.
- Cathedral base: `7f6c972b477ff37030c236dc2656a8f601d1590b`;
  executed working diff SHA-256:
  `82563bbc073b0f6e6585f28c1df85f3e578b856e0933abb44782d86ada0da595`.
  All captured source, profile, harness and lock inputs remained unchanged
  during execution. Subsequent documentation edits do not change the executed
  Omega source or harness; a fresh checkout still performs ordinary package
  update for its own source identity.
- Fresh image SHA-256:
  `be5e6a7f56c0a49fa4d67a7665f1efcf6aceb83f3fbbf2ad059f6a0b62e04f4b`.
- Serial SHA-256:
  `f2307463d8e8b587e04cad9591bf029b94044f3a071ec8c66bafa02904ab76b5`.
- QEMU `11.0.2`; bundled OVMF code SHA-256:
  `33090cc07675baa5190d9f1e84bf5176b33bcbfa9bacac522961150cdb6dbb2a`.
  Harness exit: `0`.

The command was:

```text
python3 tools/boot-harness/run.py \
  --compiler /Users/zcanann/Documents/projects/Omega/.agents/worktree/provider-demand-check/build/omega-timer-gdt-final \
  --omega-repo /Users/zcanann/Documents/projects/Omega/.agents/worktree/provider-demand-check \
  --build-dir build/timer-gdt-boot/final-reviewed/boot --timings --smoke
```

The new image booted from `run-jigu1xqh/esp/EFI/BOOT/BOOTX64.EFI`.
Its complete ordered output was:

```text
Hello from Omega
Owned 35 MiB
Boot map ready
Kernel entered
Memory split
Memory checked
Memory paired
Page ready
Tables ready
CPU checked
Root located
Root read
Tables retained
Mapping retired
Mapping checked
GDT checked
```

The last marker requires the load, readback, restoration and neighboring-byte
checks in [segments.omg](../../source/kernel/arch/isa/x86_64/interrupts/segments.omg).
The table stays borrowed until restoration;
the kernel's existing roots remain live across the operation. The admitted
closed interval excludes NMI, machine-check and debug-exception delivery.
No segment register is reloaded and no interrupt table is installed.

The compiler repair also executed an Omega helper eight times in a separate
CPU harness. Independent checks observed GDTR restoration, all ten written
bytes, immediate neighbors, registers, stack and flags. Corrupting an immediate
neighbor caused that same payload's test to fail. Cathedral's twenty host
harness tests reject missing, truncated and out-of-order success markers;
the source-layout check passed.

Raw records remain in this checkout's
`build/timer-gdt-boot/final-reviewed/result.json`, its adjacent `boot.log` and
serial output, and `build/timer-gdt-boot/final-update/`. The Omega worktree
retains `build/timer-gdt-final-producer.json` and the final native CPU records.
These are local evidence paths, not files promised in a fresh checkout.

Five preexisting timer and entry edits were present in the working tree but
are not imported or selected by this boot. Their bytes affect package identity,
so fresh checkouts perform ordinary package update. They do not alter the
executed GDT path; the retained source review records that dependency check.
This run keeps its original working-tree and compiler identities.

## Earlier memory-only boot with the published compiler

Ordinary package update checked nine packages, reused their accepted capabilities
and published the final lock in **461.5036 seconds**. A fresh-image smoke run then
passed all fifteen markers in **461.7412 seconds**, including its 30-second guest
observation. These are host workflow times, not guest boot times.

- Omega source: `6c99629c2becf0d0081982ae66cf1ae93a3b4a2e`, published through
  [PR #1641](https://github.com/CathedralOS/Omega/pull/1641) on main as
  `3ba944676de4644c64f17ca469177b1a4a935ce0`.
- Compiler SHA-256:
  `c72ba3d397f2e61c3951c86c5487aacb1066196a87c1d1e729a1833d5142318a`;
  verified optimization level `1`.
- Cathedral base: `cb902e65fb16cc2ef74407e0040641c78d121312`;
  executed patch SHA-256:
  `7c125856b1220fbc4c82d723eb30efbeb479ed0723081dbdc5764b11cfa592fb`.
  All 2,177 recorded Omega-source file hashes still match after execution.
  Later qualification prose is outside the nine captured package roots.
- `std` and `alloc` source pin:
  `c2cfe5f3decef0294ea80527f7a95264e4520ec9`.
- Fresh image SHA-256:
  `9151b7a137801262e6142a3706f095f8d03d7b29115040ac1f6f3c00fefb9760`;
  14,477,824 bytes, x86-64 PE32+ EFI application.
- Serial SHA-256:
  `f356eb7c46c19b4acf598aa0f11bd81856a83ddb4a91ff1042e9bf0c35a8c0d0`.
- QEMU 11.0.2 with its bundled OVMF firmware; harness exit `0`.

From the isolated Cathedral checkout, the command was:

```text
OMEGA_PHASE_PROFILE=1 /opt/homebrew/bin/python3.13 tools/boot-harness/run.py \
  --compiler /Users/zcanann/Documents/projects/Omega/.agents/worktree/provider-demand-check/build/omega-result-field-merge \
  --build-dir build/omega-uefi-current/boots --smoke --timeout 30
```

The harness created `build/omega-uefi-current/boots/run-1e1q4lql/`, placed that
invocation's new image at `esp/EFI/BOOT/BOOTX64.EFI`, and observed these ordered
messages in `serial.log`:

```text
Hello from Omega
Owned 35 MiB
Boot map ready
Kernel entered
Memory split
Memory checked
Memory paired
Page ready
Tables ready
CPU checked
Root located
Root read
Tables retained
Mapping retired
Mapping checked
```

Raw command, producer, source and artifact records are retained in the Omega
worktree's `build/cathedral-published-boot-evidence.json`,
`build/cathedral-final-update.json` and `build/cathedral-final-acceptance.json`.
These are local evidence paths, not files another checkout is expected to have.
The [harness guide](../../tools/boot-harness/README.md) gives the normal replay
procedure.

## Earlier corroborating run

An earlier fresh-image boot also passed all fifteen checks in **448.4566
seconds**. It used diagnostic-only compiler source
`197750a786931312dbdd6ef5e2eda2fd50839441`, based on the same published functional
source above. Its executable SHA-256 was
`14574f16ce6d7c25e5b86b17b7a5256c33ebbd4145acf9f0138fc1fdf6fc4cd6`,
with verified optimization level `1`. The executed Cathedral patch was
`50e9b1d886642ea75ec90b60bf8f1ac3ee2161decc4162e4127a88b7561838c9`;
subsequent source edits changed comments only. Its image and serial bytes are
identical to the published-compiler run.

That invocation created `build/omega-uefi-current/boots/run-u0ykep45/`.
The original record remains `build/review-resume-cathedral-boot.json` in the
Omega worktree. This historical observation is not relabeled as a run of a
later compiler.

## Fresh negative boot control

A separate newly compiled image reached the first fourteen ordered checks,
through `Mapping retired`, and never printed `Mapping checked`. The unchanged
smoke harness returned exit `1` in **450.7381 seconds**, including its 30-second
guest observation. This demonstrates that a failed readback check prevents the
final success message after the firmware handoff and mapping exercise.

The isolated source copy changed only the final mapped-readback expectation in
`source/kernel/boot/uefi/lab/memory/mapping/activation.omg`:

```text
last == 97 + next
```

became:

```text
last == 97 + next + next / 2
```

The harness and all other files in that frozen copy were unchanged. The source
file hashes before and after this mutation were respectively
`e4e0282034d410b418c960b1ac37dce423a369e7da1c173aed0e071e62ac0ca8` and
`4521af301140622837f72eeb03a2c7365210a146a3010aa7780abbf3ae9a80a5`.

- Compiler source: `397037555c6e9216d0e0269a2390558ca1ec8fbb`, published through
  [Omega PR #1660](https://github.com/CathedralOS/Omega/pull/1660) on main as
  `7b83e320f1d671f34f591812fdb7f00de34d0d97`.
- Compiler SHA-256:
  `45847e2148462980bdec891960bef8e995297bfb0d59e62f4ad6780d432a2301`;
  verified optimization level `1`.
- Fresh image SHA-256:
  `421f4ebd7c77c7fef1dccc74a51ca9a4b3e919bdee019af85d304c59025e01fd`.
- Serial SHA-256:
  `e56b55acf2cd1ed23d039bbb3c218d3be13f440deaf65e3c1f97398cfbee57c3`.
- Fresh output: the Omega worktree's
  `build/cathedral-negative-tree/build/omega-uefi-repaired-negative/run-8f5oi029/`.
  Command, source and result records are in
  `build/policy-replay-negative-boot.json` and its adjacent log.

An earlier attempt stopped before image creation because package review treated
changed temporary helper names as a code change. PR #1660 repaired that compiler
bug. The ordinary initial update and unchanged resume now produce identical
complete comparisons for all nine packages. All 246 requests match the prior
audited set except the declared format version; only normal decision words were
changed to accept them. Resume succeeded in **436.5375 seconds**. No comparison
identifier was edited or bypassed.

The positive and negative runs retain their different compiler identities. The
repair changes package-review descriptions; it does not change executable
selection, lowering, layout, ABI or native emission. Independent review of that
consumer path supports retaining the published `6c99629` positive observation
alongside the fresh `3970375` negative observation. It does not relabel the older
boot as a run of the newer compiler or assert identical image bytes between them.

## Remaining iteration cost

The earlier successful review resume with trace compiler `197750a` could not
retain its checked source graph: serialization exhausted the cumulative
sequence-element allowance at byte 324,896,512. The fresh negative compilation
with `3970375` also reported that limit, at byte 288,153,512, before successfully
emitting its image. This caused no acceptance bypass, but later invocations may
repeat expensive checking. The exact sequence responsible is not yet attributed;
no cache or trust limit was raised. This remains a measured workflow gap,
separate from the observed guest boot results.
