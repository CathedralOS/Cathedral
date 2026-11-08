# Omega UEFI boot qualification

Cathedral's Omega kernel now builds with reviewed Git dependencies, leaves UEFI
firmware and wakes ordinary kernel code eight times from timer interrupts. Two
fresh QEMU/OVMF boots passed: one ran undisturbed; the other checked eight actual
interrupt returns and the retained tables, stacks and context. This qualifies a
single-CPU kernel boot milestone. Scheduling, persistent kernel mappings, a
desktop, SMP and physical hardware remain unfinished.

## Dependency build and timer boot, 2026-11-08 UTC

The final dependency update checked ten packages and published the reviewed lock.
The review removed 226 obsolete hosted callable/supply rows and retained 20 policy
rows; it introduced no new grants. The new interrupt package uses checked Omega
implementations and the existing explicit hardware permissions. Cathedral's
executable boot source was preserved while general compiler defects were repaired.

The compiler repairs are published on Omega main through
[#2049](https://github.com/CathedralOS/Omega/pull/2049),
[#2113](https://github.com/CathedralOS/Omega/pull/2113) and
[#2117](https://github.com/CathedralOS/Omega/pull/2117). The final producer was
built from submitted source `b19e1c2aff703f55651e348f02c710af6d7e15ac`, published by
#2117 as `59eb7bb4562ea2bce7367a8250bb9b22eeff5538`. Its optimized, uninstrumented
CLI completed the ordinary native build in **661.117 seconds**, with unchanged
inputs and no resource stop. Sampled peak process-tree memory was **21.319 GB**.
Source checking accounted for 509.068 seconds and Terminal lowering for 57.592
seconds. These are host build measurements, not guest boot times.

The resulting 17,024,000-byte PE image equals the earlier R13 image byte for byte.
That comparison closes the earlier pending dependency-build check. The current
program and its conditions have their own identities:

| Input or result | SHA-256 or identity |
| --- | --- |
| Compiler executable | `b63cf6471a994cc2db6857fa1fdb65b0e4b16742fee0ca360a651ee81bd88322` |
| Published library Git revision | `64a843d84e992cad23222769065e18cdc107e2d6` |
| Customer source manifest | `e8721f3927d17aba217e5677299ae103a8f19bd8b32bfc1b8ce98c463ecac7e8` |
| Accepted dependency lock | `83d4a30d271bc7c488f9f22c925d8e501ae3d1c02f8c39bc74c717291f02d68b` |
| Current Terminal bytes | `af2e6a3b7e50c2abceaed3b8ef766a782e4c7dc7d5661a1b3f423fad384cdf5c` |
| Current semantic fingerprint | `33f7bed87740e00383f6ca368f9038d6fb138980ca2951097ebeca003c2767c3` |
| Reviewed environment description | `4fec94a4879b124d1551b6bd77f62fe8dc8c523de5aea4fea294bdd42f9e64bf` |
| Current bootstrap condition file | `52d6d9e8d6082213f417f802f199824268f03ed95e2590ab65adbf008a9f006a` |
| Current delivery condition file | `41300c8db7f26f9cde5ad211bf253f7e1fa933248367c2efbd24761a57153d54` |
| Emitted PE image | `c7ef3610fb7589ab5f55a5b47fb54f97cff091716303396a0f0598544aa60c56` |
| Successful ordinary compile result | `e84eca7888e88894692fd334f01d6a3d4ef14ef7e2d7b754594e8ff27bd7572b` |

Fresh whole-program inspection and canonical readback establish the selected
semantic entry 1, the actual route through the Build-selected checked registrar,
and installation 312 in body 66. Both encoded conditions bind that current
fingerprint, target and exact entry/installation; ordinary native compilation
accepted them. The current physical source requirement belongs to Cathedral's
UEFI package, not std's same-spelled declaration. Its nominal loaded-application
handle and live system-table reference retain their own current identities.

The **fresh undisturbed boot** passed in **12.566 seconds**, with all eighteen
ordered markers, `Owned 34 MiB` and exactly `........Timer wakes 8`. The guest
remained alive at the result. The separate **fresh observed boot** passed in
**12.785 seconds**. It authenticated its own load at `0x2737000`, extent
`0x1435000`, then checked eight entry/pre-IRET/restored sequences with no
differences in the checked restored state. All 33 gates and all 17 guard words
at each of 17 stops matched: 289 guard comparisons, plus two separate context
canaries. Counter and timestamp advanced from zero to eight; completed
acknowledgement debt was zero. Both owned VMs stopped cleanly afterward.

| Fresh runtime evidence | SHA-256 |
| --- | --- |
| Undisturbed result | `aed83904da2d26597bdf1477ef77564b0f8b24bd3cc50f29c3d3849e55f6db1a` |
| Observed result | `61560a3ed359e40c208700f880b789cd8b7400748914a0efcad7ca3c4f0e3f2e` |
| Serial output, both runs | `14793ddd7c84194c4f9a6b7d441b52734a4233d64bfee4200946f753ce16c9c6` |
| Root's complete runtime readback | `d821da7136236f1afb562e6c483d9e796ce7dcbc1059922c43a21caa4b431cb2` |

The receiving decision reuses the original exact-image code, layout, private
stack and retained-owner reviews. The native backend, realization and codecs are
unchanged, and the actual output bytes match. The frozen receiver and R6 observer
therefore keep their **original native artifact/Psi labels** as qualification
inputs while loading the newly emitted file. No new native metadata identity is
claimed, and no old runtime receipt is relabeled. The current compiler/source/
conditions are joined separately by the actual build and byte comparison; both
runtime occurrences above are new. The observed run's UUID is
`207b516b-6001-40c8-8a7b-993a74377d45`.

The complete image, compiler-private stack and suspended source owners remain
backed under the adopted loader/receiving contract through ExitBootServices.
The two boot roots stay owned. Incoming firmware stack access needs only the
return-address word before the private switch; the 128 KiB target value is a
private-capacity floor. The configured platform and precise environmental
conditions are the same as described in the historical R13 section below.
Neither condition nor debugger readability grants memory. Observed XMM values
are zero; no fatal interrupt was injected, and debt was not sampled inside the
handler. These results do not establish nonzero SIMD patterns, hardware or
universal future delivery.

Raw current evidence stays out of Git under Cathedral's ignored
`build/omega-timer-publication-r3/ordinary-qualification-r1/`: package review and
resume, `whole-program-inspection-r2`, `environment-reviewed-r19-root`,
`ordinary-native-r22-root`, `receiving-applicability-r23-root`,
`uefi-undisturbed-r25`, `uefi-observed-r27` and the root runtime readback. The
archived runtime bytes were rehashed against their original records. Later edits
to this qualification and navigation documents change no executed source body.
The [normal harness](../../tools/boot-harness/README.md) retains its malformed,
missing and out-of-order output controls; the earlier native/static failure
controls retain the original identities and limits stated below.

## Historical R13 timer qualification, 2026-11-07 UTC

The R13 image leaves firmware, retains the image/initial-storage roots and
suspended local owners, and installs all 33 entries: 32 fatal exception entries
and one returning PIC/PIT timer entry. Its undisturbed UEFI boot reports
`Owned 34 MiB`, all eighteen ordered markers through `Segments ready`, then
exactly `........Timer wakes 8`. The guest remains alive at that result.

A separate invocation authenticates its own firmware load before stopping at
the exact installed Work call. It observes all 33 gate targets, eight complete
entry/pre-IRET/restored frame sequences, context counter and timestamp advancing
from zero to eight, and all 17 authored table, task-state and stack guard words
at each of 17 stops (289 comparisons). The context's two separate surrounding
canaries also remain intact; they are not part of that 17-word roster.
The actual table, GDT, encoded TSS, root, context and stacks lie in the emitted
private stack partition. The debugger uses reviewed QEMU host pauses without guest debug
injection or guest state writes. Observed XMM values are zero; this does not
qualify a nonzero SIMD pattern. Pending acknowledgement debt is not sampled
inside the handler. No fatal delivery was injected.

The controlled receiver uses QEMU 11.0.2, machine `pc-q35-11.0`, CPU `qemu64-v1`,
TCG, one CPU, 128 MiB, pinned OVMF pflash, a fresh variable store and ordinary
`EFI/BOOT/BOOTX64.EFI` loading. The source's two explicit conditions exclude
asynchronous NMI/machine-check/physical failure until table publication, then
require the reviewed single-owner architectural delivery and debug conditions
while installed. They grant no memory. A separately adopted physical receiving
contract covers firmware input validity, ABI, complete loaded-image backing and
retention through ExitBootServices and the suspended installation. Source and
native owner/call/frame checks support that contract; debugger reads do not
grant permissions or prove future lifetime.

Exact retained identities:

- Compiler source: `843ab01ddb8adc3d5ecf70d236f57db065af3e4f`.

- Image SHA-256:
  `c7ef3610fb7589ab5f55a5b47fb54f97cff091716303396a0f0598544aa60c56`.
- Artifact identity:
  `0ec43d39d70deb766cf2cef1a7e0e1787daf7d351a7e8010068f3aee1743c1ae`.
- Captured Terminal SHA-256:
  `82002ffc567e2ae139a83f887e9c39fcea300a9ff2147a449be4d205159259fc`;
  semantic fingerprint
  `e2e6002cea59fdbdfdc232e4e44a39acf6172099a573da9c9458ebb91f6c001b`.
  Semantic entry 1, physical adapter 54 and installed Scope 312 belong to this
  capture; they are not names inferred from another image.
- Native/export driver SHA-256:
  `b4e46025586044f633ba81fc445c249f755fc8458d42db7b43c586073d118c7b`.
- OVMF code SHA-256:
  `33090cc07675baa5190d9f1e84bf5176b33bcbfa9bacac522961150cdb6dbb2a`.
- Undisturbed runtime result SHA-256:
  `e7291753406ea840cb0eba1a9cffe77b80b7af62024982662ee050d48e5f92b4`;
  stopped result:
  `65e7245ae1781f5fd3eaf53883a9d7f79e64069c11201b5b5dcd394d1d28233b`.
- Adopted receiving review SHA-256:
  `4f9a6864bf2133b17ee168b213af4bcebc8cf62f1d2f6f7663f6f8875f6512b1`.

Raw evidence remains in Omega's ignored
`build/native-main-integration/native-completion-r28-r13-original-parent/`:
`artifact`, `physical-receiver-r1`, `uefi-undisturbed-r1`, `uefi-observed-r1` and
`receiving-occurrence-review-r1`. The full source was checked and captured with
its frozen matching libraries before native completion. This result does not
claim that the unchanged ordinary Git dependency pins have already reproduced
that build. Root's final pin selection, review and fresh compile remain pending.

The first stopped attempt failed because the observation plan omitted the two
context-guard names. Its failed receipt remains separate; the retry added only
those labels and kept the frozen observer unchanged. Existing source and native
invalid controls remain separately pinned. The normal boot harness now requires
the eight-wake line as well as every earlier marker, but its serial oracle does
not independently perform the stopped observer's state/return checks.

The dated sections below preserve earlier milestones and their original source,
compiler, image and validation limits.

## Historical selector boot check, 2026-11-04 UTC

After `TSS checked`, the kernel reloads code and then data/stack selectors from
its retained GDT. The new marker requires CS 8, DS/ES/SS 16, FS/GS 0, TR 24,
the complete expected GDT and unchanged guard words. Its whole table/stack owner
remains borrowed alongside the existing memory roots. The marker does not reread
all 104 TSS bytes or the FS/GS bases; the separate compiler CPU witness checks
full-width base preservation.

A refusal aborts while retaining the backing. The two reloads are sequential;
a later refusal does not undo a completed code reload. Enabled CPU control-flow
protection (CET) is refused before the code reload changes selectors. The closed
single-CPU bootstrap conditions still exclude asynchronous/debug entry and
firmware calls until the complete exception floor is installed.

Ordinary update checked all nine packages in **805.080 seconds**. Independent
review found that all 246 capability decisions changed only the stored format
version, 26 to 27; the intentional selector source changes were also audited.
Normal resume published the matching lock in **602.365 seconds**. A fresh build
and boot then passed in **695.206 seconds**, including 30 seconds of guest
observation; the compiler reported **664.947 seconds**. These are host workflow
measurements, not guest boot times. Output reports `Owned 34 MiB` and all eighteen
messages in order.

- Optimized compiler SHA-256:
  `ca5b9db812cb1b4891e457ec1d473f767820572772597fdb1b2ef077802d2c3a`,
  optimization level `1`, with its frozen matching library.
- The selector operation is published through
  [Omega PR #1807](https://github.com/CathedralOS/Omega/pull/1807), main
  `6ace467e4f78bb55d5d3e6727539e4a47814ec0a`. The boot producer retains its
  actual earlier source snapshot; it is not relabeled as a build of that merge.
  The integrated compiler separately passed the same source/CPU controls and
  emitted byte-identical selector and task-register witness payloads.
- Cathedral base: `2c287634938b9f004d97932bbc9656a42516ffef` plus the recorded
  selector changes. Captured source, harness, profile, accepted lock, compiler
  and library inputs stayed unchanged during the successful run.
- Fresh image SHA-256:
  `5dbda23368e327df1552db1a986bd5ecea62ad18cec3950e589e50a91037f31a`.
- Serial SHA-256:
  `5300919c99eb9b3d495764d3c48e1a3f48abc80eb4034390703c976a083d71f7`.
- QEMU `11.0.2`; OVMF code SHA-256:
  `33090cc07675baa5190d9f1e84bf5176b33bcbfa9bacac522961150cdb6dbb2a`.
  Harness exit: `0`.

Local raw evidence is under `build/timer-selector-integration/`: the successful
`boot-v1-r2/result.json`, image and serial log; ordinary update/resume records;
and the independent source/package reviews. Five older timer/entry prototype
edits were captured but remain outside this boot's selected import/call closure.
The first update caught an unqualified namespace use, which was corrected.
The first fresh build ran out of disk space before an image reached the harness;
its log and recovered failure record remain separate from the successful retry.
Normal replay follows the [boot harness guide](../../tools/boot-harness/README.md).

The separate compiler CPU test completed eight calls and checked all selectors,
full-width FS/GS bases, kernel GS base, stack, registers, flags and neighbors.
Corrupting a neighboring byte made the same payload fail. QEMU's far-return
helper omits the code descriptor accessed-bit write; that witness allows only
that bit to remain unchanged. Cathedral seeds the accessed bits before loading
the table. No hardware accessed-bit write or timer arrival is claimed by these
selector tests.

## Earlier task-state boot check, 2026-11-04 UTC

The kernel builds the x86 task-state segment (TSS) with the existing typed
hardware definitions and explicit 104-byte codec. Its four stack addresses and
GDT descriptor remain inside one retained owner. The selected compiler operation
checks the actual GDT association and descriptor before executing LTR.
`TSS checked` requires selector 24, both words of the expected busy descriptor,
and unchanged GDT neighbors and outer guard words. The whole owner stays
borrowed in a nonreturning continuation with the existing memory roots.

The admitted single-CPU bootstrap interval excludes NMI, machine checks, debug
exceptions, firmware calls and other asynchronous entry until a complete
exception floor is installed. CLI alone does not establish that interval.
No code/data segment selector is reloaded and no interrupt handler is enabled.

Ordinary update checked all nine packages and published the matching lock in
**692.170 seconds**, with zero source differences. The fresh build and boot
passed in **560.160 seconds**, including 30 seconds of guest observation.
The compiler reported **529.878 seconds** total. These are host workflow
measurements, not guest boot times. Output reports `Owned 34 MiB` and every
previous marker in order, followed by `GDT checked` and `TSS checked`.

- Optimized compiler SHA-256:
  `6f441628ccacd7211c891590aefe4c5537a3db5c812bbbf71bcd41080f229045`,
  optimization level `1`, with its frozen matching library.
- The task-register source matches submitted `d429befa992e867bce7a12ebe0dd26e09a060648`,
  published through [Omega PR #1802](https://github.com/CathedralOS/Omega/pull/1802)
  on main at `64183c84cb09b9a31f39fdf3cff657e0c8761ae9`.
  The producer retains its actual earlier build snapshot, including the checked
  descriptor-helper repair from [PR #1801](https://github.com/CathedralOS/Omega/pull/1801).
- Cathedral base: `7d5b31fd67ccdc84c8c1e0e76649fd796a6ffe65` plus the recorded
  task-state boot changes. All captured executable source, harness, profile,
  accepted lock and compiler inputs remained unchanged during execution.
- Fresh image SHA-256:
  `7800fbcc6501c0b1b8ba6f4adcce306952aae01346b5aeca003be22c29f2e5d9`.
- Serial SHA-256:
  `4418c9dfe92b22ff7f3938887bb59936ac3685514d3445be2397d9cf63347a34`.
- QEMU `11.0.2`; OVMF code SHA-256:
  `33090cc07675baa5190d9f1e84bf5176b33bcbfa9bacac522961150cdb6dbb2a`.
  Harness exit: `0`.

The normal replay uses the [boot harness](../../tools/boot-harness/README.md)
with that compiler and `--timings --smoke`. Local raw evidence is retained in
`build/timer-task-state-integration/boot-v4-r3/`, including `result.json`,
`boot.log` and `boot/run-89k95nta/serial.log`; the ordinary update record is
`kernel-update-v4-r1.json` beside it. These local paths are not promised in a
fresh checkout. Documentation edits after execution change no executed body.

A separate compiler CPU test called the actual Omega operation eight times,
checking first activation, busy/wrong-table/wrong-task/invalid-selector refusals,
complete descriptor changes, registers, stack, flags and neighboring storage.
Corrupting a neighbor made the same compiled payload fail. Those observations
are retained under Omega `build/task-register-activation/cpu/`; they are ordinary
calls, not timer interrupts. The boot harness also rejects missing, partial and
misordered TSS markers. Earlier boot failures below retain their original scope.

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
