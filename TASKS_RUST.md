# Rust reference OS: tasks and handoff

## Active direction: Omega-owned desktop behavior

**Owner instruction, 2026-10-26:** focus on driving the Omega implementation
forward. The Rust reference is now supporting infrastructure and executable
evidence, not the default destination for further feature development. The open
outcomes below remain requirements; their order does not override this direction.
Continue Rust work where it enables an Omega migration, fixes a regression, or
supplies a missing mechanism needed by that migration.

The app catalogue, instance tickets, current selection, switch admission,
fallback choice, cancellation and scene restoration now execute in
`source/distribution/matrix/service/children/workspace/`; see the verification below. Rust retains
opaque catalogue/workflow state, exact request/task identity, asynchronous scene
transport and resource custody. The service-turn sequence now executes in
`source/distribution/matrix/service/`, through `service/host/` operations; the
Rust service entrance only drives that session. Request-family routing and owner
success consequences now execute in `source/distribution/matrix/service/requests/`;
`service/requests/requests.rs` supplies authenticated facts and executes the choice.
Owner child-control now descends into `requests/control/main.omg`; launch target/refusal
and role choices execute in `launch/admission.omg` and `launch/roles.omg`. The host
under `service/children/control/owner.rs` and `launch/request.rs` retains exact
capabilities, standing grants and pending replies. Realm discovery and saved
configuration/picker/background setup now execute sequences under
`source/distribution/matrix/service/children/startup/realm/`. The host entry points are
`service/children/startup/startup.rs` and `service/children/configuration/configuration.rs`.
Stock role resource offers, profile ceilings and preferred surface counts now
execute under `source/distribution/matrix/service/children/launch/resources/`. The host composes exact
image/scope launch plans, accounts for transport overhead and allocates resources.
Live-layout plans and stock service offers now execute in Omega as well, with
common typed interface matching under `source/platform/contracts/imports/`.
The bounded service-import record decoder now also runs in that Omega platform
package, before matching. The Omega scanner
in `source/platform/contracts/imports/image.omg` requests bounded spans from the
host's sealed mapping and owns ELF record discovery. Its Rust adapter descends
through `policy/imports/imports.rs`; the original Rust parser remains an oracle
and serves consumers not yet migrated. Local evidence includes 97 matrix tests,
full host tests, both matrix Clippy configurations, and ordinary F5/cold reboot.
The current desktop compiler is the qualified release producer `20dcd0409b66`,
available on Omega main through PR #339. It retains the publication repair from
PR #292 and supports Colors' typed app calls. Current evidence is under
[Colors cleanup](#colors-separates-app-behavior-from-native-packets); earlier
sections retain their historical producer. The supported checkout recipe is in
[the compiler build guide](tools/omega-ui/README.md#build); no local source patch
is needed. The default local compiler path resolves to that tested checkout.
Preserve duplicate/unknown record refusal, load-range overflow and truncation checks;
do not turn a malformed declaration into an absent legacy record.
Keep actual grants, mappings and resource custody in the host; do not turn the
native bridge into a permanent second distribution API.
The independent Omega matrix is a useful precedent, but its synchronous parent
refresh must not replace the desktop's yielding path.

Notes, Colors, Background and the chooser already execute their ordinary app behavior in
Omega. Matrix configuration, lifecycle, launcher/local recovery and ordinary
launch sequencing also do. This work does not claim a fully Omega matrix process
or a native Omega kernel. Recheck the recorded native-kernel proof-propagation
blocker against a newer compiler separately; it need not stall userspace work.

### Background now executes in Omega

The installed background's acquisition, storage-refusal probe, geometry-change
painting and lifecycle replies now execute in
[app/main.omg](source/distribution/applications/background/app/main.omg).
Its session and commands have named types; the temporary native packet/protocol
codec lives under `background/bridge/component/`. Rust retains only executable
startup, the existing startup-fault injection and the common Omega host. The old
Rust `backdrop.rs` implementation was removed.

The app still receives interaction only, reuses one surface, paints only after
a grant-epoch change and waits between work. A refused storage acquisition must
leave the interaction broker usable; unexpected storage authority fails closed.
Only canonical HEALTH/LOSS messages are accepted. Transport and drawing failures
end the task without replay; host operation 12 returns the app's error to its
entry. Background's stock private allowance is now 14 pages: the existing five
transport pages plus the common host's nine buffer pages. Reader, notification,
surface, storage and descendant authority remain unchanged.

Seven Windows host tests invoke the actual packaged Omega export. They are part
of the default `cargo test --locked` suite. The default suite, Background/host
Clippy (including the guest target and startup-fault feature), format and source
layout pass. The ordinary F5 desktop and cold reboot pass at
`build/boot-harness-rs/desktop/20261027T052235517408Z`; independent storage reopen
verifies the copied user, notes, color and ungranted root. This Background pass used
the then-qualified `e8bebc16d5883` producer. Background's component receipt is under
`build/omega-component/distribution/applications/background/bridge/native/`.
Four nested desktops also pass at
`build/boot-harness-rs/desktop/20261027T052513030543Z`, including protected close,
saved Notes/Colors, repeated subtree reclamation, isolation and cold reopen.
The picker, selected-user and background startup-failure fixtures pass at
`build/boot-harness-rs/nested-failures/20261027T052816308102Z`: bounded local
retries and repeated reclamation preserve the outer draft. Logs for this pass
are `build/omega-component/background-{host-suite,desktop,nested,startup-failures}.log`.

### Colors separates app behavior from native packets

Colors now follows Background's separation: [app/main.omg](source/distribution/applications/colors/app/main.omg)
advances typed session, input, storage and rendering work; the temporary
`colors/bridge/component/` owns native packet decoding and encoding. Storage owns
the six-byte `MC01` record codec. Rendering owns image contents and geometry;
the bridge only packs them for the existing host. No service grant, wire protocol,
resource ceiling or syscall changed. The installed app remains hosted by Rust.

All 14 compiled Colors tests pass, including the existing persistence, uncertain
write reconciliation, independent pointer edges, exact image bytes and bounded
input-drain checks. Added cases cover surface refusal with and without grant
replacement, malformed stored records, and pointer-slot retirement. Colors is now
included in the default host test suite. Full host tests, Colors host/guest Clippy
with all features, Rust format, source layout (83 Omega / 154 Rust edges), and the
five component-packaging tests pass with the release producer below.

The qualified desktop compiler is `20dcd0409b66de0f6d25d1a345e8da57d71bbb0a`,
release opt-level 3. The prior `e8bebc16d5883` producer cannot lower the typed
storage-step call in this cleanup; this is already supported by the newer
compiler, with no local compiler patch. F5's default now selects that release
after desktop and UI qualification. Producer identity and binary
hash: `build/omega-component/colors-typed-release-producer.json`.

The ordinary desktop and cold reboot pass with that producer at
`build/boot-harness-rs/desktop/20261028T005107350410Z`; the durable oracle checks
1,819 records. Shared UI host tests also pass. Logs are
`build/omega-component/colors-typed-release-{host-tests,clippy,desktop-retry,ui-tests}.log`.
The first desktop build stopped on an orphaned UI compiler lock left by a
terminated editor check. Its owner PID was verified absent before removing that
one lock and retrying; no source or compiler workaround was required.

Qualification caveat: the first UI guest run passed its interactive checks, then
cold boot hit the previously recorded ATA WRITE completion timeout before login
(`op=30`, LBA 24, status `d0`, 50 ticks). The storage provider stopped and sign-in
recovery appeared. Evidence is preserved at
`build/boot-harness-rs/omega-ui/20261028T005745141006Z` and
`build/omega-component/colors-typed-release-ui-guest.log`. This is not a passing
cold boot and the existing ATA issue remains unresolved. No deadline was raised.
The unchanged UI fixture then passes both boots at
`build/boot-harness-rs/omega-ui/20261028T010054239987Z`, including buttons, slider,
text editing, conditional rendering, clear/reset and close/reopen. Retry log:
`build/omega-component/colors-typed-release-ui-guest-retry.log`.

The first default-compiler four-level run reached every desktop, then reproduced
the previously recorded deep-input observation: Notes displayed `de` after the
harness sent `deep`, before Colors launched. No guest panic was logged. Preserve
`build/boot-harness-rs/desktop/20261028T010151806869Z` and
`build/omega-component/colors-typed-default-nested.log`. This matches the still
unresolved observation at `desktop/20261024T233351949583Z`; it is not evidence of
stable deep input or grounds to dismiss the loss as host load. The fixture and
its input pacing remain unchanged.

The unchanged four-level fixture then passes fresh boot and cold reopen at
`build/boot-harness-rs/desktop/20261028T010554059900Z`, using F5's default compiler
path without `OMEGA_BIN`. It covers protected close, saved Notes and Colors,
repeated subtree reclamation and isolation from the outer realm. Log:
`build/omega-component/colors-typed-default-nested-retry.log`. This passing replay
does not resolve the input-loss observation. UI packaging tests (11), source-layout
laws (8) and changed navigation links also pass. Colors uses 39 of the existing
240 image pages; the matrix uses 153. No native Omega kernel or ARM64
qualification is claimed for this app cleanup and desktop compiler upgrade.

## Language alignment: defer the blocked nested-machine migration

Owner clarification, 2026-10-26: replacing `state` with nested `machine` is a
language migration, not a reason to restate thousands of private contracts.
Each machine has its own checked contract, with justified private inference.
Preserve incoming control-flow facts, result typing, recursive reasoning and
constant-stack execution through the unified compiler path. Every incoming call
must establish inferred requirements; lexical nesting alone supplies no facts.
See [Omega's invocation contract](https://github.com/CathedralOS/Omega/blob/main/wiki/spec/language/state_contracts.md).

An audit at Cathedral `088082d4` found 4,829 tracked `state` declarations across
`source/`, including 244 in the chooser. Older `pub` and data-owned machine forms
also remain, especially in the translated libraries outside the desktop closure.
This is migration debt, not proof that every occurrence is independently broken.
Do not use a green desktop build as evidence of whole-tree language alignment.

A fresh optimized compiler from Omega `9b9bf65d51cbb` rejects a chooser change
that changes only those 244 declaration keywords. The policy component builds,
but app checking reports four mutable/shared argument conflicts, four missing
private arrival bounds (`i < 51` and `i < 6`), and one private result-bound failure
(`digit_count`'s `result <= 4`). These are the migration acceptance witnesses;
adding manual bounds or restructuring calls to evade them is not the agreed fix.
An intermediate experiment with explicit contracts reached a further disagreement
between retained and freshly derived mutation summaries; that experiment was
also discarded. No compiler checking or package-review gate was weakened.

The chooser source was restored exactly after that migration probe. F5 used
`e8bebc16d5883` for that restoration; the current producer is recorded above.
The owner reports parallel work on unification; Omega's
claim registry also records `NESTED-MACHINE-UNIFICATION`. Our duplicate compiler
claim was released without compiler edits. This pauses only this migration,
not other Omega OS work; no new owner semantic decision is needed.

Local evidence lives under `build/omega-component/language-migration/`:
`chooser-spelling-only.patch` and `chooser-current-rejection.log`. The producer's
revision, optimized profile and binary hash are in
`build/omega-component/language-current-producer.json`. To retry on a compiler
that contains the unification repair, apply the patch to the recorded source,
select that compiler with `OMEGA_BIN`, and run
`cargo test --locked -p cathedral-chooser --lib` in `source-rs/`, then the ordinary
`python tools/boot-harness-rs/desktop_test.py --trace-storage` flow. Require the
unchanged behavior and existing rejection tests to pass; a successful parse or
policy-only export is insufficient. Finish qualification before changing F5's
compiler. [Source guidance](source/README.md#language-compatibility) records the
ongoing distinction between the current specification and staged implementation.


Restoration verification: all 20 compiled chooser tests pass again through the
unchanged default compiler; log `build/omega-component/chooser-language-restored.log`.
Source layout passes (75 Omega / 151 Rust package edges), and the tracked chooser
source matches `088082d4` exactly. No fresh QEMU claim is made for this audit;
the desktop runtime sources and compiler selection were restored, not migrated.

## Active correction: Omega source quality and discoverability

The owner's current priority is correcting `source/` code and organization, not
adding more packet-driven policy ports. The stock matrix now follows the Rust
`service/children/` tree in one service package, with an explicit temporary native
`bridge/`. Close/loss policy uses named survey, answer and outcome types; service
turns use commands, observed facts and event kinds. Their native adapters own the
numeric packet encoding. Local startup and launcher recovery now use named
state and outcomes as well, with their encoding in `bridge/policy/`. Resource
admission now uses named capacities, optional declarations, role offers and
surface preferences; its codec is also confined to that bridge. Ordinary launch
now separates its typed operation state, guarded entrance and progress/cleanup
sequence; role selection returns a named launch plan. Admission also uses named
facts, targets and refusals. Import policy operates on common contract records;
its native codec, image scanner adapter and declaration adapter live together
under `bridge/policy/launch/`. Workspace now uses named catalogue entries,
selection outcomes and switch commands/completions. Its native state layout and
query tags live in `bridge/policy/workspace/`. Request routing now separates
authenticated caller facts, decoded message families and named operations;
owner effects use clearing/replay choices. Owner child-control descends through
`requests/control/` into syntax and per-operation checks. Its native envelope,
operation numbers and effect bits live in `bridge/policy/requests/`. Actual
eight-word IPC formats remain in the service's wire decoders. Realm setup now
uses named jobs, ready/waiting commands, completion observations and outcomes;
its native state and status encoding lives under `bridge/policy/realm/`.
Configuration now also uses named requests, geometry, rectangles and plans; its
native adapter lives under `bridge/policy/configuration/`. Saved-file and real IPC
codecs remain beside their service owner. Package/module imports retain this hierarchy.

The chooser now also uses ordinary typed calls for catalogue resolution,
geometry, input selection, launch/copy intents, replies and observations. Its
session no longer stages internal decisions through a 128-word scratch array.
`chooser/policy/` retains the Rust policy responsibility split; `chooser/app/`
owns the session, settings and rendering. Both temporary native entrances now
live under `chooser/bridge/`, with the Rust probe packet codec in `bridge/policy/`.
Start at [the chooser map](source/distribution/applications/chooser/README.md).

Chooser cleanup verification: full `cargo test --locked`, including 20 compiled
chooser tests and 97 matrix tests, passed on producer `e8bebc16d5883`.
Layout checks pass with 75 Omega / 151 Rust package edges; all eight layout laws,
five component-packaging tests and Rust formatting pass. Chooser Clippy passes
with and without `rust-probes`, with warnings denied. Normal F5 acceptance
covers login, three independent Notes instances, protected close, slot reuse,
Colors, saves and cold reboot; the durable oracle verifies 1,767 records.
Artifacts: `build/boot-harness-rs/desktop/20261027T034355939124Z`.
Logs: `build/omega-component/chooser-typed-{host-tests,desktop,clippy}.log`.
No service grants, image-page ceilings or deadlines were enlarged.

This is not a completed tree-wide cleanup. The stock matrix policy no longer
uses native packets as its domain model; retain its host regressions as further
mechanisms move into Omega. The Omega kernel now separates shared ownership
in `kernel/core/` from x86 mechanisms in `kernel/arch/isa/x86_64/` and PIC/PIT
policy in `kernel/arch/board/pc/`. UEFI now has separate physical-entry, firmware,
handoff and post-handoff memory-lab owners. The lab coordinator and firmware
ownership transaction still need smaller stage entrances, but first restore
full compiler qualification of their retained contracts. Older interrupt
prototypes also need qualification before new kernel features. Start the audit at
[source/README.md](source/README.md), with the ownership map and actual entrances.

The directory move requires reviewing the compiler's local source-location
replacement; it grants no new service authority. Migration/compiler qualification
above remains separate from the architecture correction. The local compiler
checkout for the current producer is
`C:/Projects/Omega/.agents/worktree/cathedral-boot-providers`;
`build/omega-ui/compiler` is a junction to it. The previous qualified checkout is
preserved at `cathedral-component-publication`, with its junction backed up as
`build/omega-ui/compiler-e8bebc16`. The older UI checkout, including its local
publication patch, is preserved at `cathedral-legacy-ui` beside those worktrees.

Verification of the reorganized tree, 2026-10-26: all 97 compiled matrix tests and
`cargo test --locked` across the host workspace pass. Rust formatting, source layout
(71 Omega / 151 Rust edges) and its eight dependency-law tests pass. Ordinary F5
flow passes with three Notes instances, protected close, Colors, saves and cold
reboot; the durable oracle verifies 1,689 records. Artifacts are under
`build/boot-harness-rs/desktop/20261026T231306307127Z`; logs are
`build/omega-component/matrix-structure-{tests,host-tests,desktop}.log`.
This run used producer `e8bebc16d5883`, not an arbitrary current-main compiler.
The normal session profile also builds through the default compiler path without
`OMEGA_BIN` (`matrix-default-build.log`).

Follow-up over `6dee9044`: startup recovery uses `Startup`, `ChildRole`,
`Context`, `PickerAction`, `PickerMode` and `RetirementEffect`. Launcher recovery
uses `Recovery`, `Phase` and a `Command::Admit` carrying its correlation ticket.
The domain entrances contain no native packet slots or numeric operation dispatch.
Their adapters preserve the existing Rust host interface; the host still owns
real tickets, images, authenticated connections and task custody.

Verification: the 97 compiled matrix tests pass with the supported compiler
selected through the default path. Three nested desktops, protected close, saved
state, repeated subtree reclamation and cold reopen pass. Picker, selected-user
and background fault injection all retain bounded local recovery and preserve the
outer draft. Asynchronous launcher recovery and owner revocation both pass with
input during the pending read and cold reopen; revocation permits no late
admission or fallback. Layout checks and navigation links pass. Logs are
`build/omega-component/matrix-recovery-{typed-tests,nested,startup-faults,async-recover,async-revoke}.log`.
Guest artifacts: `desktop/20261026T233346922557Z`,
`nested-failures/20261026T233732612252Z`, `desktop/20261026T233946964061Z` and
`desktop/20261026T234107883140Z` under `build/boot-harness-rs/`.

Resource-policy follow-up over `3db18014`: `launch/resources/main.omg` now owns
actual admission sequencing, replacing its former numeric dispatcher. It fits a
named `Profile` and `Declaration` to an offered `Capacity`; accepted allocations
carry an explicit surface request or retain the host offer. `recipes.omg` owns
stock roles and their offers, while `surfaces.omg` owns preference bounded by
standing authority and available backing. `bridge/policy/resources.omg` alone
encodes native operations, packet slots and errors. Rust allocation mechanisms,
resource ceilings and public service protocols are unchanged.

Verification: all 97 compiled matrix tests pass, including strict profile
refusals, absent declarations, owner ceilings and surface arithmetic boundaries.
The default F5 composition passes three nested desktops, protected close, saved
Notes/Colors, repeated subtree reclamation, isolation and cold reopen. Layout
(71 Omega / 151 Rust edges) and resource navigation links pass. Logs:
`build/omega-component/matrix-resource-{typed-tests,nested}.log`; guest artifacts:
`build/boot-harness-rs/desktop/20261026T235443314258Z`. The matrix image uses
122 of the existing 240 load pages. No limit was raised. The supported default
Omega compiler was used; no compiler patch or environment override was needed.

Launch-workflow follow-up over `30f96eb7`: `launch/main.omg` begins, issues,
validates completions and cancels work. `operation.omg` names the `Launch`,
`Plan`, `Command`, ready/waiting `Phase`, `Completion` and `Dispatch` values;
`progress.omg` exposes the successful sequence and reclaim-before-restore cleanup
order. Role selection also produces a named plan. `bridge/policy/launch/main.omg`
alone encodes workflow commands, the waiting offset, role flags and opaque state.
The Rust caller still owns the actual task, image, binding and authenticated
connection. The subsequent admission/import cleanup is recorded below.

Verification: all 97 compiled matrix tests pass with the supported default
compiler, including all launch-role/storage combinations, failures at every
ordinary stage, wrong/duplicate completions and cancellation. Ordinary F5 passes
three nested desktops, saved Notes/Colors, protected close, repeated reclamation
and cold reopen. Placement failure, pending-binding revocation and revocation
after an accepted scene-commit reply also pass with sibling input and cold reopen.
Logs: `build/omega-component/matrix-launch-{typed-tests,nested,place,binding-revoke,scene-late-revoke}.log`.
Guest artifacts under `build/boot-harness-rs/desktop/`: `20261027T001832876354Z`,
`20261027T002124768135Z`, `20261027T002230106497Z` and `20261027T002329268063Z`.
Layout (71 Omega / 151 Rust edges) and navigation links pass. The ordinary matrix
uses 125 of the existing 240 image pages; no limit or service deadline changed.

The first fresh-boot run (`20261027T000820312761Z`) passed its guest checks but
exceeded the harness's five-second host exit wait after QMP acknowledged quit;
the harness terminated it, so that attempt did not prove cold reopen. Capture
now allows 30 seconds for graceful emulator teardown and disk flushing, keeping
the failure/termination path and refusing cold reopen if exit still times out.
The successful runs above use that allowance. This changes test-host teardown,
not guest scheduling or service timeouts.

Admission/import follow-up over `f036da70`: launch admission now consumes named
`Request`/`Facts` and returns `Decision`/`Refusal`, retaining the selected target
when a later check refuses it. Busy/role/target/capacity/argument/authority order
is unchanged. Stock import policy owns only default declarations and offer
filtering over common platform contract records. The native image/declaration
adapters moved out of the service tree into `bridge/policy/launch/`, alongside
admission/import encoding and the actual `main.omg` dispatcher. There is no
same-named `launch.omg` beside that folder. The bridge explicitly imports the
platform contracts it uses; the protocol package has no reverse dependency.

Verification: all 97 compiled matrix tests pass, including occupied-slot subsets,
refusal precedence, import matching against the Rust oracle, record mutations
and ELF discovery boundaries. Ordinary F5 passes three nested desktops,
protected close, saved Notes/Colors, repeated subtree reclamation and cold reopen.
Undeclared and incompatible optional storage both remain absent for Notes while
Colors retains its independent binding, including cold reopen. Logs:
`build/omega-component/matrix-admission-{typed-tests,nested,undeclared,incompatible}.log`.
Artifacts under `build/boot-harness-rs/desktop/`: `20261027T003551074803Z`,
`20261027T003847008877Z`,
`20261027T003912241661Z`.
Layout (72 Omega / 151 Rust edges), all eight dependency-law tests and navigation
links pass. The ordinary matrix uses 126 of the existing 240 image pages. No
resource limit, guest deadline or runtime grant changed. The supported default
compiler was used without source patches or an environment override.

Workspace follow-up over `c6aa45b6`: `workspace/main.omg` guards begin, issue,
completion and cancellation; `operation.omg` names the state and messages, and
`selection.omg` owns activation/restoration sequencing. `catalogue.omg` exposes
named instance, position, entry and selection values. The native bridge owns
all numeric query/command tags, opaque state words and waiting offsets. The
existing Rust host still retains exact tasks, connections and scene custody.
Cancellation preserves the first refusal and drains outstanding scene work;
an accepted late commit requires restoration before the final reply. Failed
restoration remains fatal. Instance tickets never wrap or restart on slot reuse.

Verification: all 97 compiled matrix tests, layout (72 Omega / 151 Rust edges),
all eight dependency-law tests and navigation links pass. Three nested desktops
pass saved state, protected close and repeated subtree reclamation. All four
asynchronous switch fixtures (`complete`, `failure`, `stop`, `late-stop`) pass
routed input, repeated switches, protected drafts and reclamation. Every guest
run also passes cold reopen. Logs:
`build/omega-component/matrix-workspace-{typed-tests,nested,complete,failure,stop,late-stop}.log`.
Guest artifacts under `build/boot-harness-rs/desktop/`:
`20261027T005850605790Z`, `20261027T010150556142Z`,
`20261027T010254127627Z`, `20261027T010407883268Z`,
`20261027T010501410640Z`. The ordinary matrix uses 131 of its existing 240 image
pages. No resource limit, service deadline or grant changed.

Compiler finding during this cleanup: a nested bridge helper named `entry`
collided with the enclosing machine's implicit entry in the supported producer.
Catalogue publication/activation returned, but an entry query repeatedly jumped
to the same dispatcher. Renaming that helper to `read_entry` restores the intended
call; all 97 compiled matrix tests then pass. This is not a compiler repair.
The exact failing source, component, standalone probe and debugger/disassembly
evidence are preserved under `build/omega-component/workspace-entry-repro/`.
Keep this as a compiler-resolution follow-up, not an owner-design question.

Request-policy follow-up over `98546e76`: `requests/main.omg` now routes named
caller facts and message families to named operations. `children.omg` selects
child workflows, and `owner.omg` defines diagnostics, fallback and successful
clearing/replay consequences. `requests/control/main.omg` admits owner work,
preserves loss exclusion and selects forwarding or local validation; `local.omg`
owns the per-operation checks. Both real-wire decoders retain canonical padding,
scoped storage classification, attachment allowlists and the shared LOSS/imports
opcode precedence. Native 128-word envelopes, numeric operation results and
effect bits exist only in `bridge/policy/requests/`. Rust still owns original
requests, authenticated connections, tasks, grants and pending replies.

Verification: all 97 compiled matrix tests pass, including caller separation,
malformed packets, reserved arguments, owner failure atomicity, storage attachment
allowlists, control limits and forwarding/loss precedence. Layout (72 Omega /
151 Rust edges), eight dependency-law tests and navigation links pass. Three
nested desktops pass repeated subtree reclamation and confined persistence.
Producer acquisition and denial both pass ordinary desktop instances, protected
close and saved state; their durable host oracles verify 1,833 and 1,727 records.
Owner revocation after accepted launch-scene commit passes restoration, sibling
input, protected drafts and reclamation. All four guest runs pass cold reopen.
Logs: `build/omega-component/matrix-requests-{typed-tests,nested,producer,producer-denied,late-revoke}.log`.
Artifacts under `build/boot-harness-rs/desktop/`: `20261027T013924813796Z`,
`20261027T014156628351Z`, `20261027T014332561434Z`, `20261027T014443716396Z`.
The ordinary matrix uses 141 of its existing 240 image pages. No grants, resource
limits or guest deadlines changed. The supported producer is unchanged. The
native dispatchers explicitly import their callable entries: the producer's
qualified-only module references failed lowering despite each branch lowering
in isolation. Direct callable imports retain the intended composition.

Realm-setup follow-up over `ae61f9c8`: `startup/realm/main.omg` begins one named
job, issues a ready command once, and accepts only its outstanding completion.
The existing discovery/configuration/picker/background files own the subsequent
choices over `Setup`, `Context`, `Completion`, `Entries` and `Outcome` values.
`bridge/policy/realm/` alone owns native state words, command/job numbers, the
waiting offset, status encoding and discovery bitfields. Optional-background
failure remains separate from the final configuration outcome. Cleanup failure
replaces the preceding error; no failed cleanup reports success. This preserves
the host's existing resource custody and synchronous error paths.

Verification: all 97 compiled matrix tests pass, including setup ordering,
duplicate/wrong completions, every realm-entry subset, deferred unplaced startup,
missing authority, rollback and cleanup failure. Layout (72 Omega / 151 Rust
edges), eight dependency-law tests and navigation links pass. Three nested
desktops pass protected close, confined persistence, repeated reclamation and
cold reopen. Separate picker, selected-user and background fault fixtures all
pass bounded local recovery and repeated reclamation while preserving the outer
draft. Logs: `build/omega-component/matrix-realm-{typed-tests,nested,startup-faults}.log`.
Artifacts: `build/boot-harness-rs/desktop/20261027T015543543265Z` and
`build/boot-harness-rs/nested-failures/20261027T015819897726Z`.
The ordinary matrix uses 150 of its existing 240 image pages. No resource limit,
deadline or grant changed; the supported compiler is unchanged.


Configuration follow-up over `40a9338a`: `configuration/main.omg` now checks
named caller facts and requests before planning stock role placements.
`placement.omg` validates bounded geometry and returns named rectangles;
`wire.omg` decodes the actual layout IPC. The saved-file codec remains directly
available to the chooser. Native byte slots, geometry packets, status codes and
plan serialization live in `bridge/policy/configuration/`. The forwarding/packet
`live.omg` entrance is gone. Host authority revalidation and local world mutation
remain unchanged.

Verification: all 97 compiled matrix tests pass, including 100,000 geometry
comparisons, overflow boundaries, saved-format mutations and live-layout refusal
precedence. The settings guest test passes cancellation, saved/live edits with
running apps, preserved drafts, later launches using the new arrangement, cold
persistence and independent defaults for a second user. Artifacts:
`build/boot-harness-rs/settings/20261027T020748185520Z`; logs:
`build/omega-component/matrix-configuration-{typed-tests,settings}.log`.
Layout (72 Omega / 151 Rust edges), eight dependency-law tests and matrix
navigation links pass. The ordinary matrix uses 153 of the existing 240 image
pages; no resource limit, service deadline or grant changed. Verification used
the supported compiler through the default path without source patches.


Kernel structure follow-up over `556a95fe`: shared extent/geometry now live in
`core/memory/`, coalescing wake state in `core/timing/`, CPU mechanisms in
`arch/isa/x86_64/`, and PIC/PIT preparation/acknowledgement in
`arch/board/pc/timer/`. Core has no hardware package dependency. Boot selects
concrete architecture operations; no unused multi-ISA facade was added.
`memory/memory.omg` owns backed table preparation. The former 1,052-line numeric
validation file is now a complete-image entrance descending into image,
walk, entry and geometry owners. The x86 arrival ABI is separate from the PC
root provider and timer-vector selection. Active source inventories, probes and
navigation follow the moved definitions; older verification receipts remain
historical. The old architecture detail is preserved in `arch/FRONTIER.md`.

Verification with producer `e8bebc16d5883`: table-walk, table-clone and
paging-profile probes pass in both native and interpreted execution before and
after relocation. The complete four-page validation closure also passes checked
compilation through an isolated consumer. Layout (73 Omega / 151 Rust edges),
eight dependency-law tests, 19 boot-harness tests and changed navigation links
pass. A token comparison of all moved mechanism bodies confirms no algorithm
changes beyond module imports and the two validation helper exports needed by
the split. This pass does not change the Rust desktop kernel or its F5 profile.

Current Omega kernel compatibility limits, reproduced before and after the move:
the boot build cannot resolve the `ExtentRootProvider` operand in its build
selection; the root-geometry native probe fails lowering
`BootGeometry::exclude_image_suffix` with ?resident membership home has no exact
source projection?. These are not successful boot checks. Several older interrupt
sources still use retired `pub` syntax/contracts and are not qualified by the
numeric probes. Do not claim current live interrupt or full native-kernel support.
Logs are `build/omega-component/kernel-{structure,table-walk,table-clone,paging-profile,root-geometry}-{baseline,relocated}.log`
and `kernel-validation-check.log`. Continue architectural cleanup and separately
resolve source/compiler compatibility without weakening the resource contracts.


UEFI structure follow-up over `f725f202`: `boot/uefi/main.omg` is now the real
firmware-arrival validator. It enters `firmware/firmware.omg`, then the bounded
map/exit/grant transaction under `firmware/ownership/`. The copied boot record
lives in `handoff/`; backing assumptions in `memory/profile.omg`. The selected
semantic continuation is visibly a lab under `lab/memory/`, with lifecycle,
aligned-page and mapping owners. Mapping descends into firmware-table retention,
hardware custody, activation/retirement and destination checks. The actual root
entry path and build-selected `Kernel::enter` identity remain unchanged. The
contract dependency is now named `uefi_memory` so it cannot shadow local
`memory/profile.omg`.

Thirteen fixed COM1 marker bursts moved to `lab/memory/diagnostics/markers.omg`;
the lab retains its polling, timeouts and success/failure transitions. Expanding
those calls reproduces the original code tokens. This is not yet a small lab
coordinator: repeated UART waits and retained-state continuations still need
cleanup once full compiler checking can qualify that refactor. The older
firmware ownership transaction also retains descriptor-walk/report details.

The firmware-root test exposed a missing domain qualification from the preceding
core namespace move. Both firmware consumers now name
`core::memory::geometry::BootGeometry::Separated` explicitly; no arithmetic or
separation condition was relaxed. The complete map/root probe now passes native
and interpreted execution. The diagnostic leaves pass checked compilation in
an isolated freestanding consumer and were not executed on the host. Layout
(73 Omega / 151 Rust edges), eight dependency-law tests, 19 harness tests, local
import paths and navigation checks pass. Source-token comparison verifies the
relocated bodies after accounting for those domain names and marker calls.
Logs: `build/omega-component/uefi-{firmware-root-relocated,markers-check,structure-build}.log`.
Full boot still stops at the existing `ExtentRootProvider` build-selection
lookup; explicit qualification or an extra build import did not resolve it and
those experiments were discarded. No native-boot success is claimed. Diagnose
that source/compiler boundary next before further rewriting retained control
flow; the ordinary Rust-kernel F5 desktop is a separate baseline.

## Native Omega boot qualification after the structure pass

The desktop used producer `e8bebc16d5883` during this native-kernel investigation;
its current qualified compiler is recorded at the top of this handoff. Do not
switch F5's compiler merely to test the native-kernel experiment. The reorganized kernel
is not yet qualified as a complete native boot.

A fresh isolated compiler at `b8c81bbb8c1f` reproduced the `accept_boundary`
operand failure with no Cathedral dependencies. Build ran before Product
imports existed, but boundary acceptance tried to resolve its subject early;
completed Build settings also dropped root grants. [Omega PR #322](https://github.com/CathedralOS/Omega/pull/322)
repairs this through the existing source-issued request protocol and post-join
Product resolution. Candidate `6acfc27ebbbcd` is qualified locally and merged upstream as
`a72862f5fdbf`; PR and merge-queue integration checks passed. Four Build-effect integration tests and eight focused
corpus fixtures pass, including helper identity, untaken acceptance, fabricated
activation, forged protocol, missing subject and wrong declaration kind.
Scoped Clippy passes; pinned qualification has no invalidations.

With that candidate, Cathedral passes the earlier lookup and now stops while
acquiring its pinned `alloc` Git dependency: `source root exceeds identity entry
limit of 4096`. This does not establish native kernel type checking or boot.
Next: fix the allocator acquisition/selection boundary, then qualify the retained
boot contracts before further splitting the large firmware and memory-lab
coordinators. The separately recorded root-geometry lowering and retired
interrupt syntax limitations remain open. No grants or acquisition limits were
weakened, and no desktop compiler path changed.

Evidence: `build/omega-component/kernel-provider-{baseline-builtin,baseline-uefi,repair-uefi}.log`
and `kernel-provider-qualification.json`. The isolated compiler and full evidence
are retained at `C:/Projects/Omega/.agents/worktree/cathedral-boot-providers`.

Rechecked during the language audit with a fresh optimized Omega `9b9bf65d51cbb`
producer: an isolated archive of Cathedral `088082d4` still refuses the pinned
`alloc` acquisition with `source root exceeds identity entry limit of 4096`.
This happens before native-kernel checking or image emission. The main checkout
has an older pending package review, which was preserved; the isolated archive
avoids conflating that review with the source acquisition failure. Evidence:
`build/omega-component/kernel-current-language-isolated.log` and
`language-current-producer.json`; snapshot: `kernel-language-current/` beside them.
The optimized `target/debug/omega.exe` in the retained `cathedral-boot-providers`
worktree was built from `9b9bf65d51cbb`; later source work does not relabel that
producer. Its separate `target/release/omega.exe` now contains the qualified
`20dcd0409b66` desktop producer recorded above.
Prior PR #322 evidence remains historical and unchanged.
The allocator acquisition failure is separate from nested-machine unification.

The acquisition cause is now traced: `BuildSourceAcquisitions::acquire_source`
calls whole-root `acquire_git` before dispatching `PackageSelection::Named`.
`workspace_acquisition.rs` then derives every member from that complete capture.
This charges the repository against the selected library's 4,096-entry limit.

Omega [PR #339](https://github.com/CathedralOS/Omega/pull/339), source
`20dcd0409b66d`, merged as `79a661193e9625f3192d1f7524057868eb29d563`,
supplies the staged resolver prerequisite. It authenticates the
repository once, reads bounded root/member declarations across separate calls,
and materializes selected payloads under the existing package limits. Tests
cover an oversized unrelated tree, selected-content refusal, declaration-roster
refusals, a moving branch/shared cache, pin reuse and offline recorded recovery.
The Git resolver suite passes 52 tests with three existing Windows-specific
skips; strengthened session tests and scoped Clippy pass. Logs are in that
worktree's `build/allocator-{resolver-git-tests,session-tests,clippy}.log`.

**The kernel acquisition is still blocked; the application is not connected to
that resolver yet.** A prototype connection exposed the next invariant: Build
must execute once, preserving its complete input inventory and negative
observations. Turning declaration-only bytes into `CapturedLocalSource` and
rerunning the selected Build after materializing its Product violates that
contract. That prototype was discarded, without weakening source-join checks.
Continue at `compilation-inputs/src/source_imports/frontier.rs` and
`package_compilation.rs` (`validate_unbound_source_join`), together with the
application's `source_resolution/readiness.rs`. Establish compiler-owned custody
for staged discovery/Product source before wiring the session into acquisition.
The existing `executed_git_source` workspace witness requires five activations,
selected-return reuse, private-access refusal and no unselected Product work.
Then rerun the actual isolated Cathedral kernel. This needs no new owner choice;
do not raise limits, reintroduce static manifest parsing or claim a boot result.
The resolver claim was released after landing; this remaining compiler integration
is recorded work, not a live background build or a blocker for Omega app migration.

## Memory budgets and allocator delegation

**Open; ratified 2026-10-25.** Complete the
[memory-budget contract](wiki/spec/resources/memory_budgets.md) using the existing
`source-rs/kernel/core/user_tasks/resources/physical/` funding owner, memory owner
and platform grant/connection path. Existing conserved-frame and retained-reader
evidence is a starting point, not missing work to rewrite.

Acceptance: a matrix partitions one envelope between children; nested delegation
and concurrent allocation cannot duplicate funding, exhaustion leaves its sibling
usable, and failure/cancellation returns only discharged reservations. Demonstrate
both direct bounded allocation without per-request ancestor IPC and a substituted
provider whose client cannot bypass it with a raw syscall. Keep fine-grained
allocation local; no mandatory allocator process or instrumentation framework.

Exercise retained readers, queued grants, producer/subtree death and final release
without premature refund or account-reuse escape. Bound mapping/page-table,
handle/object and pending-grant costs separately; state whether evidence establishes
a backing ceiling or total-memory bound. Carry the path through the Omega-facing
boundary and actual guest execution. Do not freeze syscall spellings, offer-queue
layout or source signatures from illustrative discussion samples. The
[ordinary backing contract](wiki/spec/resources/memory_budgets.md#ordinary-backing-allocation)
is now ratified: demonstrate positive minimum-size requests returning qualified
Omega `Extent` values with actual usable lengths and satisfied alignment, zeroed
bytes throughout exposed rounding (including reused backing), and non-executable
ordinary access. Reject zero, overflowing/unsupported requests and insufficient
budget without partial grants or leaked reservations. Reuse the existing carrier
and provider evidence, not duplicate backing/size/alignment records. Carry this
through the Omega boundary and real guest behavior; existing page tests alone
do not establish the full byte-request contract.

## Shared-memory IPC and notification outcome

**Open; accepted architecture, not an owner blocker.** Carry the
[IPC contract](wiki/spec/communication/ipc.md) and
[notification contract](wiki/spec/execution/notifications.md) into ordinary service
use, replacing kernel-copied application message queues rather than adding a
permanent second transport. This is implementation work across the Rust reference
and its Omega-facing boundary; it is not an upstream-library translation task.

Start at `source-rs/kernel/core/ipc/ipc.rs`, the wait entrance
`source-rs/kernel/core/user_tasks/execution/waits/waits.rs`, shared backing in
`source-rs/kernel/core/user_tasks/memory/memory.rs`, and the platform runtime
`source-rs/platform/libraries/user-runtime/server/server.rs`. Preserve the active
reply-dispatch/launch work; its correlation and cancellation rules remain service
requirements, not a reason to freeze the copied-channel ABI.

Kernel notifications and a first reusable single-writer shared-memory/mailbox
slice execute on x86-64 and ARM64. Ordinary F5 now uses them for physical input
to the root matrix, root display commands/images, root storage and separate
parent-storage and pipelined interaction streams between stock matrices, Omega app
broker/interaction/storage, independent Split interaction/storage forwarding,
Background, the separate Omega UI presenter, Split's input producer and masked
capture viewer, brokered stock/Split lifecycle controls, and boot root/storage
provider supervision and boot recovery lifecycle control. Boot display configuration and input readiness use separate shared streams;
source-attenuation checks run on the driver-owned shared connection.
Brokered child input hints now use separate wait-only notifications.
The scoped storage fixture also
exercises both private-root provider connections. See the evidence below. Remaining acceptance:

- Continue the shared service migration through the remaining ordinary service
  clients, lifecycle/control links and explicitly copied
  fixtures. Independently
  selected control/data streams
  now multiplex; immutable attachments execute through ordinary display and
  storage, including nested stock matrices, Omega leaves and Split. Shared endpoint
  replies now retain custody through the Omega/Split service broker. Preserve
  mapping isolation, hostile-writer validation and live-reader lifetime; waking
  proves none of them.
- Demonstrate real cross-process ping-pong, including early reply/next-request,
  signal/register races, coalescing, closure/cancel/deadline races, revoked wait
  rights, denied foreign handles, stale incarnations and retained funding through
  death/transfer/final release. A successful wake is not message completion.
- Migrate the ordinary desktop's service path and Omega host bindings, preserve
  nested input/rendering, launch cancellation and storage behavior, and retire
  copied payload syscall consumers once migrated. Keep grant-installation checks
  and existing custody evidence; do not reinterpret old queue tests as new proof.
- Complete [self-coordination](wiki/spec/communication/ipc.md#scoped-participation-and-identity)
  through an Omega caller and real process registration/introduction. Exercise
  executable and scoped-path restrictions independently and together, endpoint
  selection, rejection of unrestricted endpoints and forged launch facts, and
  refusal of copied-name/restriction-downgrade attempts. The endpoint is a public
  rendezvous name, not a secret or third credential. Exercise competing first
  registrations, confirmed death and stale process
  incarnations. Timeout must not create another first participant. This does not
  require generic endpoint discovery, launch-time exclusion, a new launch-realm
  identity lever or transparent parent-provider routing; those are not ratified
  by this outcome. Existing transport tests do not close registration acceptance.

Done means the normal service path uses shared-memory data plus notifications,
not only a standalone lab. Pin focused host/race controls and real guest evidence
for that path and its invalid-authority controls. Shared-payload round trips now
exercise publication and backpressure. Input, display and storage at the root
provide ordinary service evidence, not a performance result or complete migration.

## Launch-established Matrix connection and service acquisition

**Open; owner decision ratified 2026-10-25.** Carry the
[bootstrap contract](wiki/spec/execution/components_and_matrices.md#scoped-application-bootstrap)
through funded launch-established backing/notification rights, a current-process
retrieval syscall and the ordinary service library. Include the accepted
[connection/resource-offer rules](wiki/spec/communication/ipc.md#matrix-access-and-connection-setup).
Start at `source-rs/platform/contracts/services/README.md`, the import
resolver, matrix admission and `source-rs/platform/libraries/user-runtime/services/`.
The current startup endpoint pair and `entry_with_services!` wrapper are evidence
of the old discovery mechanism, not completion or a compatibility requirement.

- Provision/init the Matrix connector at launch using existing parent authority and
  explicit funding. Lookup retrieves usable installed resources, not a bare endpoint
  or newly allocated connection. Exercise first-allocator acquisition without a
  child direct-allocation grant: parent/provider provisions that connector too.
- Preserve manifest/owner ceilings; lookup cannot select another process's connection.
  Absent authority returns unavailable. Resolve declared imports, not ambient string
  discovery; a matching friendly name cannot substitute a different provider.
- Repeated and concurrent lookup shares one logical connection with coordinated
  request IDs/custody. Demonstrate bounded allocations, no new authority, no reply
  theft and no revival after closure/revocation. Keep per-service acquisition rules
  distinct from broker lookup; do not reset an established protocol on reacquisition.
- Migrate ordinary Rust/Omega app bootstrap and subsystem library initialization
  without mandatory broker arguments to authored main or reserved application
  payload bytes. Remove the old discovery route as consumers migrate, rather than
  maintaining parallel startup/syscall paths. Retain explicit privileged boot grants.
- Exercise the normal desktop's rendering/storage acquisition, required/optional
  refusals, owner mediation, no-broker launch, connection failure and safe cleanup.
  Broker closure alone must not revoke independent service grants; actual provider
  failure and matrix subtree teardown still apply. Lazy provider activation is not
  implied by deferred acquisition and is not required for this outcome.
- Complete the versioned Matrix acquisition protocol and service-owned handoffs
  through exact recipient-bound backing and required notification offers. Check
  protocol, geometry and rights; cover partial
  acceptance, queue exhaustion, offer-before-wait, cancellation/acceptance races,
  stale/foreign offers and death. Never publish a half-installed connection or
  reclaim accepted backing on sender timeout. Include a one-shot immutable typed
  buffer handoff with no mandatory ring/header/reply resources, using the same grant
  mechanism. Validate contents or retain explicit provider/custody evidence; a cast
  or mutable authority cannot invent validity. Service layout and synchronization
  stay service-owned; wire fields, queue representation and syscall fusion remain
  engineering choices, not a universal transport ABI.

Pin focused invalid-authority and race controls plus actual guest observations.
No lookup syscall or arbitrary service catalog is claimed implemented by this
documentation change. ABI spelling/handle representation are engineering work.

## Independent installation and component-owned activation

**Open; owner boundary settled.** Implement the ordinary app update path from
[installation and activation](wiki/spec/execution/updates.md), reusing executable
admission, storage publication, scoped service connections and notifications.
Begin with a real installed app, not a kernel hot-swap framework.

- Install immutable v2 and publish its future-launch reference while v1 is
  running. Prove v1 remains usable and a new launch uses exact v2; retain old
  backing until its actual users release it. Failed candidate admission or
  reference publication must not select an incomplete image.
- Route an update request through the instance owner's explicitly authorized
  control connection. Notification submission/failure is the updater's boundary;
  no acknowledgement, a deferred app or an unresponsive app must not hold
  installation open. Reject signalling/control attempts without the right.
- Exercise one independent instance deferring and another saving/restarting under
  its owner. Readiness replies, if used locally, bind to the correct request and
  process incarnation. Compatible old/new service traffic and persistent state
  must remain usable; stopping a matrix still retires its subtree.
- Keep install/load/recovery orchestration in platform services and lifecycle
  owners. Kernel changes expose only the enforcement needed by this path. Preserve
  custody through notification failure, process death and eventual reclamation.

Done is an ordinary app update exercised through the distribution with the above
positive and rejection evidence. Global live-adoption metrics, mandatory module
migration, live kernel replacement and a universal distributed transaction are
not prerequisites. No runtime evidence is claimed by this docs ratification.

## Object-rooted watches and durable capture

**Open; architecture ratified, not an owner blocker.** Implement
[change watches](wiki/spec/storage/change_watches.md) through the ordinary storage
service, reusing the transaction engine and shared-memory/notification mechanisms.
Start at `source-rs/platform/services/storage/service.rs`,
`source-rs/platform/contracts/storage.rs` and the
[object library](source-rs/platform/libraries/database-objects/README.md).
Existing notifications and shared buffers are prerequisites available for reuse,
not evidence of watch delivery. No global kernel event log or Omega feature is
required.

Remaining acceptance:

- Exercise one real consumer watching an authorized object recursively, including
  parent/child watches, content and metadata edits, independent CoW copies,
  before/after reparenting, root movement invisibility and subtree arrival/removal.
  Pair consistent enumeration with a starting change position; neither hidden
  ancestry nor outside destinations may leak.
- Deliver self-contained committed records into funded shared buffers; demonstrate
  adversarial counters, exact backing custody, revocation and close/death races.
  Full live buffers stop capture with observable overflow, never silent loss.
  Mere observation authority cannot impose writer backpressure.
- Carry a durable subscription through mutation commit, crash before delivery,
  reconnect, duplicate delivery and acknowledgement. Prove recovery keeps events
  for committed mutations and excludes aborted ones. Disk retention and RAM
  delivery have separate funding; full rings do not lose durable events.
- Recover mandatory capture policy before covered writes, with no collector-start
  race. Exercise capacity exhaustion, authorized refusal/backpressure, retention
  beyond collector acknowledgement, policy changes and nonrecursive audit upkeep.
  A missing required policy must not silently disable capture.

Done means the ordinary service and consumer demonstrate live observation and
durable recovery under these controls, not only a log/queue lab. Summarization is
library work; historical file-byte retention and replica merging are not required
for this outcome. This docs ratification claims no runtime implementation.

## Explicit atomic storage batches

**Open; scope ratified, not a general isolation blocker.** Expose the
[atomic batch contract](wiki/spec/storage/atomic_batches.md) through the ordinary
storage service and client API. Reuse `source-rs/platform/libraries/database-objects/`
transactions, `source-rs/platform/services/storage/service.rs` and
`source-rs/platform/contracts/storage.rs`; internal atomic helpers do not establish
the public N-prerequisite/M-mutation path.

- Demonstrate a client replacing two files together after checking both revisions,
  plus checking an input-only file while writing a different output. A competing
  change to any prerequisite must refuse the entire batch with no partial writes.
- Carry authority, exact incarnation/revision identity and stable request/payload
  custody across the service boundary. Reject foreign objects, missing write
  authority, stale/reused identities and post-validation shared-buffer mutation.
- Admit funded, bounded work without holding transaction-wide locks during caller
  preparation. Capacity/oversize refusal must not split or partially apply a batch.
  Preserve existing storage and required change-capture policies.
- Exercise crash/restart before and after commit and lost replies. Reconcile the
  exact request's outcome; uncertainty must not become a false refusal or blind
  retry. State and fund reconciliation retention, and reject identity reuse with
  different contents. Pin focused invalid controls and real service evidence.

Done is a usable service batch with recovery and refusal evidence, not only a host
engine test. Query/predicate tracking, automatic read-set capture, CRDT merging and
cross-service distributed commit are not acceptance requirements. No runtime
implementation is claimed by this ratification.

## Reference deletion and scoped erasure

**Open; service contract ratified.** Carry the
[deletion contract](wiki/spec/storage/deletion_and_erasure.md) into the existing
storage protocol, provider and client API. Start at
`source-rs/platform/contracts/storage.rs`,
`source-rs/platform/services/storage/service.rs` and
`source-rs/platform/libraries/database-objects/`. Do not create a kernel erase
syscall or expose physical sharing as caller-owned bookkeeping.

- Exercise deletion of one independent CoW reference while another remains usable,
  with retained-reader custody and funded reclamation. Unauthorized references
  must reject, and shared content must not broaden authority.
- Expose the applicable ordinary-history policy and explicit no-history scope;
  prove covered recovery paths are removed without destroying independently held
  copies or mandatory retained state. Preserve required deletion audit events.
- Request cryptographic erasure through that same boundary. Providers lacking the
  required key machinery must refuse before mutation, never report ordinary
  deletion as cryptographic success. Carry lost-reply reconciliation and precise
  completion outcomes through the real service path.

The first usable slice is policy-aware deletion with honest unsupported
cryptographic-policy refusal and focused positive/invalid service evidence. Actual
cryptographic erasure remains unimplemented until a provider demonstrates covered
key/recovery-path destruction across restart, retained snapshots/backups/caches,
and independent copies. Track that remaining acceptance here; do not close it on
the refusal control or assume realm encryption already implements it. No encryption
implementation is claimed by this docs change.

## Drawing, images and resource availability

**Open; baseline ratified 2026-10-25.** Implement the
[drawing boundary](wiki/spec/human_surface/rendering.md#drawing-commands-and-completed-images)
and [resource contract](wiki/spec/human_surface/rendering_resources.md) in the
ordinary desktop path. Existing ASCII/rectangle/sealed-pixel requests are partial
evidence, not general glyph resources, remote hosting or GPU support.

- Carry bounded prepared drawing and completed images together through nested
  matrices, preserving clips, interception and resource authority without mandatory
  per-ancestor image materialization. Keep layout/shaping with app libraries;
  isolate file/resource preparation from final target writes. Reject malformed,
  over-budget and unauthorized inputs; drawing grants no text/capture access.
- Demonstrate one missing image alongside successful unrelated drawing. Skip the
  unavailable draw, remove its prior-frame contribution, and report exact
  submission/resource identities without a global last-error slot. Reacquire and
  resubmit successfully. Reject stale-generation substitutions; slow providers
  cannot indefinitely stall unrelated output.
- Use caller-funded shared command backing as well as images. Validate stable
  published batches; reject malformed lengths, unauthorized grants and mutation
  races. Demonstrate one-shot release/reuse and retained-scene replacement without
  releasing backing still referenced by CPU/device work. Derived renderer storage
  remains explicitly funded, not an automatic duplicate or kernel command store.
- Keep final-target writes in the renderer; independent apps receive no writable
  sub-page framebuffer slices. Internal row partitioning is not process isolation.
- Preserve backing through active CPU/device use, producer exit and attempted
  eviction/reuse. Charge actual retained storage and bounded metadata/work until
  release; do not replace caller-funded backing with a kernel image store. Index
  submissions by connection incarnation and retire on closure/death notification,
  not page faults. Cancel only safe queued work; drain or safely stop active work,
  preserve independent grants/copies, and reject stale connection reuse.
- Demonstrate a destination host uploading a remote resource into its own funded
  or explicitly sponsored local backing, forwarding commands through the ordinary
  interface and reusing unchanged resources. Remote names grant no local authority;
  resource replacement/disconnect must not violate active-use custody.

Use focused invalid controls and real desktop evidence; do not relabel existing
lab evidence as completion. GPU import, synchronization, presentation efficiency,
wire encoding and codecs remain separate work. No zero-copy/latency claim follows
from this architecture. Shared-cache sponsorship and stronger residency promises
remain scoped in OWNER_QUESTIONS.md and do not block the baseline.

## Shared-buffer audio playback

**Open; shared playback ratified, exclusive mode deferred.** Implement the
[playback contract](wiki/spec/human_surface/audio.md) through a platform audio
service, an audio driver and an ordinary consumer. Reuse shared-buffer custody in
`source-rs/platform/libraries/user-runtime/memory/shared/` and notification/connection
mechanisms; keep new service protocol and driver responsibilities in their platform
owners, not the kernel. Generic shared-memory tests are not playback evidence.

- Play two independently funded client streams through one final mixer and a real
  device-facing path; preserve independent stream controls and reject unauthorized
  playback, mappings and control of another stream.
- Carry playback through nested policy-only Matrices without per-ancestor sample
  copies or submix buffers, while enforcing ancestor mute/routing and revocation.
  Materialize processing only for actual signal transformations, not hierarchy.
- Define and test bounded frame publication/reuse, malformed positions/lengths,
  slow or dead producers, concurrent mutation and closure. A stalled source cannot
  indefinitely stall healthy output. Retain funding/backing through last consumer
  or device access; prove teardown does not permit stale DMA or mapping reuse.
- Observe actual output and record format, device period, buffering, work and
  underruns. Compare policy-only nesting with genuine processing depth. Do not
  claim pro-audio latency from fewer copies or host-only mixer tests.

Done is ordinary shared playback with real output and invalid-authority/custody
controls, not a ring-buffer lab. Exclusive output, raw device handoff, microphone
capture and a general DSP graph are not prerequisites. No runtime implementation
or real-time performance result is claimed by this docs ratification.

## Cursor presentation without upstream disclosure

**Open; presentation and information boundaries settled.** Complete the existing
[cursor presentation path](wiki/spec/human_surface/rendering.md#cursor-presentations)
under the [input provenance rule](wiki/spec/human_surface/input.md#upstream-provenance-is-not-input-metadata).
Start at `source-rs/platform/libraries/matrix/world/input/cursor/cursor.rs`,
the scoped channel directory and the existing bounded surface/cursor path.

- Carry a leaf-selected shared asset/hotspot through nested owner policy to final
  rendering, with hide and restoration on hover/capture-target loss or death.
  Updating position must not require the app to redraw every motion or every
  ancestor to copy pixels. Measure latency rather than infer it from the topology.
- Exercise equivalent supplied input from different upstream producer types.
  Ordinary events, directory data and cursor interfaces must expose no automatic
  real/synthetic/AI/remote flag or provenance query. Local channel identity and
  control interpretation still work. Descriptive labels are not origin credentials.
- Preserve the outer recovery gesture and owner-side UI authority without exposing
  private enforcement provenance to children. Pin denied observation, stale asset
  custody and teardown controls through the ordinary service path.

Done is scoped asset/hide behavior plus interface-level non-disclosure evidence,
not perfect timing indistinguishability or mandatory agent badges. Existing
default/hidden cursor evidence remains valid for its scope, not the full asset API.

## Current implementation evidence

Implementation update 2026-10-26: **Omega decodes service declarations before
stock admission**, over `5b3030b4`.

`source/platform/contracts/imports/main.omg` sequences header, entry and padding
validation through `record.omg`, returning typed declaration metadata. Exact
record lengths, known versions, reserved header bits, counts, required booleans,
identity/version syntax, duplicate identities and unused zero padding are checked
before matching. Explicit resource/profile demands remain metadata, not grants.
The stock adapter publishes decoded fields only after successful validation.

The Rust ELF locator exposes the bounded raw record; ordinary stock admission
uses the Omega decoder instead of `Declaration::decode`. The old decoder remains
for other hosts and conformance testing. ELF header/range access, record placement,
unknown-version and duplicate-record checks remain Rust-owned in this slice.

Four compiled-Omega tests bring the matrix total to 92. They compare all four
record forms, every entry count, single-bit mutations throughout the record,
truncation/extension, resource extremes, and ELF boundary failures against the
Rust oracle. A malformed later entry refuses before an earlier unavailable
required service can reach matching. Full workspace host tests, default and
all-feature x86-64 matrix Clippy, formatting and source layout (83 Omega packages,
151 Rust edges) pass. Logs are `build/omega-component/declaration-*.log`.

Ordinary F5 and cold reboot pass with the 1,681-record storage oracle
(`desktop/20261026T205128536334Z`). Four-level nesting passes protected close,
saved Notes/Omega Colors, repeated subtree reclamation, isolation and cold reopen
(`desktop/20261026T205254614950Z`). Incompatible optional Notes storage stays absent
on fresh/cold boots while dirty-close and Colors' independent service binding work
(`desktop/20261026T205538691307Z`). Paths are beneath `build/boot-harness-rs/`.

Implementation update 2026-10-26: **Omega plans live layout and matches service
declarations**, over `0681b83e`.

`configuration/live.omg` preserves authority-before-busy-before-syntax refusal,
checked geometry, stock role rectangles and the existing-app standing mask.
The host still revalidates exact layout authority, applies through `World`, then
remembers placement, flushes and replays. Revoked views and key masks are not
restored by planning a new arrangement.

`launch/imports.omg` owns stock offer filtering and the explicit absent-record
legacy declaration. The typed common matcher is platform-owned in
`source/platform/contracts/imports/main.omg`: required/optional interfaces,
versions, duplicates and unknown identities. It has no distribution dependency
or native packet API. The Rust host retains sealed-image mapping/unmapping,
record decoding, resource metadata and actual grant installation. No manifest
format, supported service or kernel changes.

Seven compiled-Omega tests bring the matrix total to 88: common matching against
the Rust oracle over version/identity/offer combinations, duplicate and excess
records, stock versus positional offers, absent declarations, authority refusal
order, standing placement selection and geometry extremes. Full workspace host
tests, default/all-feature x86-64 matrix Clippy, formatting and source layout
(83 Omega packages, 151 Rust dependency edges) pass. Logs are
`build/omega-component/binding-*.log`.

Guest evidence: ordinary F5 and cold reboot pass with the independent 1,673-record
storage oracle (`desktop/20261026T204101554870Z`). Settings passes cancel,
live/saved layout, running apps, subsequent launches and independent user defaults
across three boots (`settings/20261026T204222440728Z`). Two forced render
invalidations preserve Notes content and input
(`settings/20261026T204331811744Z`). Undeclared Notes storage stays absent across
cold reboot while dirty-close and Colors' independent binding work
(`desktop/20261026T204444126684Z`); incompatible optional storage also stays absent
through cold reboot (`desktop/20261026T204509795069Z`). Artifact paths are beneath
`build/boot-harness-rs/`.

Implementation update 2026-10-26: **Omega chooses stock resource offers and
profile ceilings**, over `80507146`.

`resources/main.omg` dispatches role offers, program resource selection and
surface preference. `recipes.omg` supplies launcher, picker, background,
workspace app and selected-user offers. `allocation.omg` preserves strict
explicit-profile refusal, bounded unprofiled defaults, owner ceilings and the
spare-surface preference for descendant hosts. A background still receives only
interaction; the picker still reserves its surfaces before lending to a user.

The existing host admission path retains import decoding, one-shot funding
consumption, checked transport accounting, disjoint surface allocation, executable
reservations and rollback. No kernel, budget allowance or manifest contract
changes. Legacy catalogue launch, import binding and live layout retain Rust
behavior; this is not a fully Omega matrix executable.

Nested verification exposed an Omega-host startup gap: unlike the Rust app
paths, `omega-app/host/host.rs` postponed owner-control setup until the first
event wait, after storage and rendering. One four-level run refused subtree
close because a launcher's LOSS could not be submitted
(`desktop/20261026T201541540882Z`). The diagnostic run recorded a shared-control
opening timeout while deep Notes loaded and then failed to accept typing
(`desktop/20261026T202053243457Z`). The host now completes the existing independent
owner-control handshake before service acquisition or the first app step, using
the same 300-tick deadline as the Rust app paths. Control setup/submission
refusals now retain their error codes in storage traces. No longer deadline,
automatic replay or treatment of missing LOSS as consent was introduced.

Six compiled-Omega tests bring the matrix total to 81, covering all role offers,
picker backing, every profile ceiling, absent declarations, default bounds and
surface extremes. The full host suite, default/all-feature x86-64 matrix and
Omega-app Clippy, layout (81 Omega and 151 Rust package edges), format and diff
checks pass. Guest verification after the handshake fix passes:

- Ordinary F5, multiple app instances, capacity refusal, saved Notes/Colors and
  cold reboot, including the 1,669-record durable oracle:
  `desktop/20261026T202750408062Z`.
- Four nested desktops, protected close, repeated subtree reclamation, isolation
  and cold reopen: `desktop/20261026T202417516448Z`.
- Optional input-producer acquisition/custody and the desktop workflow across
  cold reboot: `desktop/20261026T202645298923Z`.
- Independent nested Split layout, focus, resize, funded refusal, dirty-close,
  repeated reclamation and cold persistence: `desktop/20261026T202851312374Z`.
- Headless matrix suite: identical worlds, local-only views, attenuated input
  and recovery, `matrix-test/`.

Artifact paths above are relative to `build/boot-harness-rs/`; logs are
`build/omega-component/resources-*.log`.

Implementation update 2026-10-26: **Omega sequences realm discovery and setup**,
over `dd0b0d2e`.

`realm/main.omg` selects discovery, configuration, picker or background setup.
The owning sequence decides the next operation and accepts only its outstanding
completion. Discovery waits for placement, gives Matrix precedence over the
Login/Users pair, and distinguishes missing realm authority from inspection
failure. Configuration preserves launcher/storage refusal cleanup, policy clearing
before placement rollback, image release ordering and nonfatal background retry.
Picker retry arming precedes the read; background placement failure stops its
admitted task. Host mechanism errors keep their existing propagation behavior.

The former configuration leaf is now `service/children/configuration/`, with a
small entrance driving Omega and named resource owners for workspace/background,
live layout, immutable reads and saved bindings. `startup/startup.rs` similarly
drives discovery; `startup/picker.rs` executes picker resource operations. Resource
recipes and profile admission remain Rust policy. This does not make startup
asynchronous or complete the Omega matrix port.

Eight compiled-Omega tests bring the matrix total to 75: setup order, duplicate
completion, all realm-entry combinations, unplaced startup, failure propagation,
rollback and optional background refusal. The full host suite and matrix
default/all-feature x86-64 Clippy, source layout (79 Omega and 151 Rust package
edges), formatting and diff checks pass. Guest verification passes:

- Ordinary F5 desktop and cold reboot, with the 1,657-record durable oracle:
  `build/boot-harness-rs/desktop/20261026T200141117548Z`.
- Four nested desktops, recursive close/reclamation and cold reopen:
  `build/boot-harness-rs/desktop/20261026T200241878414Z`.
- Asynchronous launcher recovery with live sibling input and cold reopen:
  `build/boot-harness-rs/desktop/20261026T200509958328Z`.
- Nested picker, selected-user and background startup faults: bounded retries,
  repeated reclamation and preservation of the outer draft on disposable copies:
  `build/boot-harness-rs/nested-failures/20261026T200622233417Z`.
- Configured launcher recovery and startup failure, source fixture unchanged:
  `build/boot-harness-rs/launcher-recovery/20261026T200723249663Z`.

Logs are `build/omega-component/realm-*.log`. No kernel or stack allowance changed.

Implementation update 2026-10-26: **Omega chooses owner child-control operations
and launch admission/roles**, over `24ea8113`.

`requests/control/main.omg` owns canonical control shapes, loss exclusion, bounded
forwarded hops, pending-request exclusion and forwarding deadlines. It checks
image/import/page/surface ceilings from host facts before selecting local work.
The former owner dispatcher leaves the child record entrance; its host operations
live with control custody under `service/children/control/owner.rs`. Actual image
grants, exact connection incarnations, pending replies and local mutations remain
host-owned. LOSS and LAUNCH_IMPORTS retain their existing shared opcode and exact
shape distinction; this is no wire-format change.

`launch/admission.omg` chooses stock app positions or the owner-driven pair,
preserving refusal precedence and the standing-policy/storage-binding requirements.
`launch/roles.omg` classifies picker selection, child placement and desktop
activation, including recognition of the root owner's Login/Users pair. Rust
supplies exact chrome authentication and resolves names in its own realm, retains
the selected grants, then starts the existing ticketed workflow.

Twelve new compiled-Omega tests bring the matrix total to 67. They cover owner
authentication, canonical/malformed controls, pending hop isolation, shared-opcode
precedence, limits, every occupied app subset, refusal order, withheld imports and
launch roles/picker recognition. The full host workspace, default/all-feature
matrix x86-64 Clippy, layout and format checks pass. The owner-driven matrix and
matrix-desktop fixtures pass, including recursive input, clipping, recovery and
cold persistence. Ordinary F5/cold reboot passes with the 1,657-record oracle.
Four nested desktops, launch-read `revoke`, storage-binding `stop` and scene
`late-stop-target` pass on both boots, preserving sibling input, protected drafts
and subtree reclamation. This establishes the exercised functional behavior, not
a latency guarantee. Logs are `build/omega-component/control-*.log`.

Implementation update 2026-10-26: **Omega routes stock matrix requests**, over
`da1ecec9`.

`source/distribution/matrix/service/requests/main.omg` owns request-family selection from
authenticated owner/source/child/storage/producer facts, canonical app packet
shapes, separated-producer enforcement and the storage attachment allowlist.
`owner.omg` selects diagnostics, owner fallback handlers and successful update
consequences. Rust retains exact requests/incarnations, validation of actual
resources, pending replies, attachment custody and publication order. Unknown
callers refuse; packet words cannot select a caller class. This is the ordinary
F5 path, not a second isolated matrix or a new service protocol.

Eight compiled-Omega tests bring the matrix total to 55. They check connection
separation, malformed packets, reserved fields, app operation bindings, owner
success/failure consequences, separate producer enforcement and every storage
attachment opcode with scope bits. The first full rebuild caught a name collision
from adding matrix constants to the app reference module; a separate `matrix.omg`
import keeps operation names out of app host namespaces. The full host workspace,
default/all-feature x86-64 matrix Clippy, layout and format checks pass. Ordinary
F5 and cold reboot pass with the 1,649-record durable oracle. Four nested desktops,
repeated isolated producer acquisition/closure, existing-app selection `late-stop`
and launch-scene `late-stop-target` all pass on both boots. The late cases preserve
sibling input, protected drafts and subtree reclamation. This is functional
regression evidence, not a latency guarantee. Logs are
`build/omega-component/requests-*.log`.

Implementation update 2026-10-26: **Omega sequences the stock matrix service turn**,
over `cbb8e464`.

`source/distribution/matrix/service/main.omg` owns upstream progress order,
admission gates, child workflow/reclamation ordering, launch/recovery priority,
waiting, loss/cancellation handling, deferral and retained-event category dispatch.
Its named completion transitions show the actual sequence. Omega asks for only
the observations needed by each gate; ordinary progress does not scan deferred
events. One issued command completes once; a wake without an event restarts
progress rather than manufacturing a request.

The former large loop in `service/service.rs` is replaced by initialization and
command/execute/complete. `service/host/` owns connections, exact requests, child
handles, pending reads, backing and waits. Detailed request operations, role
choices and local world mutations still execute in Rust; this is service
orchestration in Omega, not a fully Omega matrix or native kernel. The private
host command vocabulary is not a new standard service protocol.

The first guest run caught stack duplication from returning the large host state
by value: root matrix faulted below its guarded 32 KiB stack before reaching the
loop. Server and child records now live in process-owned arenas initialized once,
as the parent/world/deferred records already do. No stack allowance, kernel or
authority changes. The failed boot is retained under
`build/boot-harness-rs/desktop/20261026T185708394471Z`.

All 47 matrix host tests pass, including eight new compiled-Omega session tests
for ordering, progress restart, wrong/duplicate completion, event disposition,
ready-request/input fairness and bounded gates across all 1,024 fact combinations.
The full host workspace, default/all-feature matrix x86-64 Clippy, layout and
format checks pass. Final ordinary F5 and cold reboot pass, including the
1,647-record durable oracle. Four nested desktops with interaction delay,
asynchronous launcher recovery, delayed launch reads and storage binding, and
late-stop launch/selection restoration all pass, including cold reopen. The
targeted cases verify sibling input, protected drafts and subtree reclamation;
they do not establish a latency guarantee. Logs are
`build/omega-component/session-*.log`.

Implementation update 2026-10-26: **Omega owns the stock app catalogue and selection
state**, over `b40c71d3`.

Workspace `main.omg` now dispatches to `catalogue.omg` and `selection.omg`.
The catalogue owns its three app positions, non-reused instance tickets, local
program-object identifiers, active instance, switch admission and first-survivor
fallback after retirement. Rust's parallel `running`, `active` and `next_ticket`
fields are removed. It retains one opaque eight-word catalogue and translates
queries/commands through the existing private native export. Publication follows
successful scene commit; retirement follows the kernel's exact task outcome.
Task/endpoint identities, chrome authentication, scene/input mutation and resource
reclamation stay host-owned. No new shared service protocol or authority is added.

Six new compiled-Omega tests cover independent instances of the same program,
stale tickets after slot reuse, busy/current/missing precedence, every surviving
app subset, invalid positions/duplicate publication and ticket exhaustion without
wrap or reset. All 39 matrix tests and the full host workspace suite pass; matrix
all-feature x86-64 Clippy, layout and format checks pass. The ordinary F5 desktop
and cold reboot pass with the 1,639-record durable oracle. All four selection
cases (`complete`, `failure`, `stop`, `late-stop`) pass, as does launch-scene
`late-stop-target`: sibling input remains usable, exact-instance rollback restores
the draft, and final subtree reclamation succeeds. Each guest case includes cold
reopen. See `build/omega-component/catalogue-*.log`.

An initial host hang was an authored transition named `entry` selecting the
machine's implicit entry state rather than the intended catalogue reader. Naming
the operation `read_entry` resolves it. Temporary tracing was removed; the
compiled catalogue tests exercise the actual operation. No compiler blocker or
Rust fallback was introduced.

Implementation update 2026-10-26: **Omega owns existing-app workspace switching**,
over `484f17a7`.

`source/distribution/matrix/service/children/workspace/main.omg` sequences Begin/Wait/Activate or
Restore/WaitRestore before Finish. Commands complete only with their exact
operation ticket and command; duplicate/wrong completions cannot advance work.
Cancellation keeps the first refusal and drains the submitted scene. If commit
already succeeded, Omega requires restoration before the reply. Restoration is
not cancellable and failure is fatal rather than false successful cleanup.

The normal F5 `SELECT_APP` path dispatches these commands through
`distribution/matrix/service/children/workspace/selection.rs`. Rust still checks
authenticated chrome and exact task/connection identities before commit and after
the reply, owns scene IPC and image custody, applies local visibility/input changes,
and refuses to deliver an old result to replacement chrome. Request admission and
synchronous lifecycle fallback activation remain Rust; this is not yet an
Omega-owned service loop or native Omega kernel.

All 33 matrix host tests pass, including six new compiled-Omega switch tests for
wrong/duplicate completion, refusal, cancellation, late commit and failed restore.
All four `desktop_test.py --selection-async` modes (`complete`, `failure`, `stop`,
`late-stop`) pass initial boot and cold reopen, routed input during held scenes,
repeated switches, protected Notes drafts and exact reclamation. The ordinary F5
desktop and cold reboot also pass with multiple Notes instances, switching,
capacity refusal, protected close, saved Notes/Colors and the 1,621-record durable
oracle. Full host workspace tests, matrix all-feature x86-64 Clippy, layout and format checks pass.
Logs are `build/omega-component/workspace-*.log`.

Implementation update 2026-10-26: **ordered pointer batches resolve development pressure input loss**, over `d000196d`.

The pressure failure was the kernel's 64-byte raw input queue overflowing, not
controller parity/timeout loss. Temporary kernel phase/wait measurements showed
no single long stall explaining the backlog. Source timing measured 23 report
calls taking 471 ms elapsed versus 31 ms of source CPU: waiting per report could
not keep up under the hostile workload. Diagnostic panic/timing changes were
removed; the kernel and raw queue are unchanged.

The positional physical source now drains at most 32 available raw bytes per
owner-control turn and sends up to four ordered pointer reports per shared call.
It flushes on a full batch, empty device queue, end of turn or before keyboard/
session-control delivery. It never waits to accumulate reports. The compositor
authenticates the source connection and validates the entire batch before any
effects, then routes each report and redraws old/final cursor bounds once.
Press/release/reset transitions are retained; this does not substitute latest
state for queued events. No enlarged queue, increased grant, scheduler priority
or relaxed timing assertion is involved. Ordinary F5 uses its existing separate
matrix input path; no general input ABI is frozen by this lab encoding.

Host tests cover malformed batches (including an invalid later report), signed
motion, capacity and ordered short clicks between app reads. The startup guest
probe denies a non-source batch and checks pointer state is unchanged. Two
development `--session-test --startup-probes --pressure` runs pass all 120 drag
reports, full scanout, memory/object/backpressure assertions and compositor,
host, source and hostile-peer recovery. Final cursor catch-up was 16/218 ms;
these observations establish neither real-time deadlines nor general performance.
Release pressure with startup probes also passes all 120 reports (31 ms final
catch-up). Development pointer tests pass capture, Escape, synthetic scope,
app/source replacement and unread-reply flood recovery.
The normal F5 desktop and cold reboot pass, including the 1,631-record durable
storage oracle. This checks regression only; batching does not change its input path.
Host workspace tests, compositor/input/display all-feature Clippy, layout and
format checks pass. Logs are `build/omega-component/input-batch-*.log`; diagnostic
logs are `input-pressure-*.log` in the same directory.

**Remaining Rust transport work (deferred behind Omega migration):** audit runtime wake/upstream adapters and remaining copied service
consumers before retiring copied application payload support. Preserve typed
grant/lifetime and hostile-authority probes while removing data-path fallbacks.
Concrete remaining entrances: `user-runtime/services/wake.rs`,
`user-runtime/matrix/{matrix.rs,admission/connections.rs,upstream/upstream.rs}`
and `user-runtime/storage.rs`. The positional matrix-composition host still calls
the copied upstream helper; stock parent loading still accepts the old five-endpoint
environment. The separate physical-key service under `platform/services/input/keyboard/`
also retains copied startup/control/data traffic. These are audited consumers,
not evidence that the already-shared F5 path regressed.

Implementation update 2026-10-26: **positional compositor, pointer and session traffic uses shared IPC**, over `723d5b70`.

The lab supervisor, app control/data, pressure peer and physical pointer source
now own funded shared connections. App data uses a four-call window; normal app
loops retain one WAIT, while hostile clients deliberately fill the window without
reading replies. The compositor retains replies without blocking unrelated
clients. Scene snapshots still use producer-authenticated sealed page offers.
The isolated wait-array probe now transfers an empty typed notification grant;
it preserves copy bounds, complete authority validation and non-consuming
readiness without copied application payloads. Timer-only sleeps use empty waits.

Session configuration retains the exact owner request until the outstanding
input WAIT completes. Provider death between poll and mixed wait retires only
the data connection; owner control remains available to reconnect the surviving
app. Replacement releases old mailbox/notification custody before admitting the
new connection. The named lab [transport owner](source-rs/distribution/lab/compositor/transport.rs)
keeps exchange/deadline handling out of `main.rs`.

Pressure exposed stale fixture funding: its old 16-page grant no longer matched
the current 256-page work object and 24-object ceiling. The grant now funds that
1-MiB object plus two mailbox writers (258 private pages), two reader pages and
two notifications. The object probe allocates 22 small objects beside the two
writers. It still requires both object/page exhaustion and unused-reply
backpressure, then cancellation/fault with live backing and successful replacement.
No kernel ceiling or latency assertion was relaxed.

**Performance failure resolved by ordered batches above:** the initial development session with both
`--startup-probes --pressure` lost raw input during the 120-report drag and failed
the existing one-second/lost-motion assertion. This reproduced in four runs,
including diagnostic attempts at less frequent control polling and a four-call
source pipeline. Neither resolved it; those attempts were removed. Development
without pressure passes all 120 reports (125 ms final catch-up in this run), and
release pressure with startup probes passes (16 ms). These are workload
observations, not input deadlines. That checkpoint recorded the failure and kept
its check intact. The newly funded full-size work object makes this a stronger
workload than the old 64-KiB documentation described; the diagnosis and fix are
recorded above.

Host tests and all-feature Clippy for compositor/input/display pass, as do layout
and format checks. Release drawing, pointer, ordinary session and pressure
session with/without startup probes pass; development pointer and session with
startup probes pass. The ordinary F5 desktop test passes through multiple Notes
instances, Colors, save/close controls and cold reboot; its host oracle verifies
all 1,631 durable records. Logs are `build/omega-component/compositor-*.log`; failed
development pressure serial logs are retained beside them. See the updated
[lab guide](source-rs/distribution/lab/compositor/README.md),
[session guide](source-rs/distribution/lab/compositor/SESSION.md) and
[pressure guide](source-rs/distribution/lab/compositor/PRESSURE.md).

**Previous next (pressure failure resolved above):** resolve the development pressure input failure above, then audit runtime
wake/upstream adapters and remaining copied service consumers before retiring
copied application payload support. This fixture migration adds no F5 UI behavior.

Implementation update 2026-10-26: **watchdog recovery uses shared IPC; clock denial has dedicated process coverage**, over `0022d3a0`.

The watchdog owner, client, observer and eight service incarnations use funded
shared connections. Each service echoes its round, prepares and consumes a hang
request without replying. Four then block and four spin without syscalls. The
owner retains its client's closure request while checking both the past-deadline
and original 12-tick child wait; it requires independent observer progress before
cancellation. Only then may terminal service closure complete the retained owner
request. Stolen/stale authority, duplicate cancellation, eight collected outcomes,
four blocked/four runnable cancellations, final spin preemption and exact
heap/frame reclamation remain checked. Guest checks all sixteen task timeouts
directly; aggregate timeout counts may additionally contain shared maintenance.

Shared runtime setup and maintenance currently require clock access. The old
clock-denial assertions move to explicit initial and supervised process roles
with no clock grants, rather than disappearing. Those roles also check CPU reads,
timed task waits, timer-only/mixed readiness and timed notification waits are
denied. Denied waits must not consume an already signalled notification: an
untimed wait still succeeds and releases the charge. Existing keyboard clockless
coverage remains unchanged. No clockless shared connection support is claimed.

The runner requires shared-watchdog and clockless-probe completion markers.
Host tests, supervision/smoke Clippy, layout/format checks and release/development
smoke pass, including 66/99 display admission failure boundaries respectively.
Logs are `build/omega-component/supervision-watchdog-*.log`.
See the [supervision proof guide](source-rs/distribution/lab/supervision/README.md).
The remaining raw sends/receives in this lab are intentional stale-authority and
failed-admission endpoint-custody probes. No runtime/kernel mechanism or normal
F5/ARM64 composition changed.

**Previous next (positional migration completed above):** migrate positional compositor traffic and audit runtime wake/upstream
adapters before retiring copied application payload support. Preserve actual
pressure/backpressure, retained-reader and cancellation checks; the already-shared
ordinary F5 route does not prove these separate consumers have been migrated.

Implementation update 2026-10-26: **sibling-service and link-graph proofs use shared IPC**, over `2f7e3bec`.

The sibling owner retains independent shared clients, leaves one exact reply
outstanding across the other child's cancellation/replacement, then collects it
without replay. Sixteen alternating restarts preserve both streams' later echoes.
Failed admission still leaves the healthy sibling usable. Raw keyboard readiness
uses a notification gate; the keyboard child retains both clock-denied and
clock-granted variants, forbidden controller-command checks and exact blocking/
timeout assertions. Only the shared echo roles gain clock grants.

The four-child link graph now uses shared owner control, shared provider client
slots and three application-to-provider clients. All sixteen app/provider
replacements retain unrelated clock/link identities and reject retired names.
The existing first-command barrier still separates app admission from connecting
to providers that the owner has not finished launching. Each connection retires
after collection and before replacement; the composition explicitly funds its
writer, reader and notification counts. At final owner return the clients remain
live for kernel parent-death teardown, rather than being closed early by Rust
drop. All expected child cancellation outcomes and exact heap/frame reclamation
remain checked. The runner requires both shared migration completion markers.

Host tests, supervision/smoke Clippy, layout/format checks and release/development
smoke pass, including 66/99 display admission failure boundaries respectively.
Logs are in `build/omega-component/supervision-graph-*.log`.
See the updated [supervision proof guide](source-rs/distribution/lab/supervision/README.md).
No runtime/kernel mechanism or normal F5/ARM64 composition changed.

**Previous next (watchdog portion completed above):** migrate the watchdog recovery exchange while preserving absent-clock
authority coverage. Shared `Client`/`Server` currently use clocks for setup and
maintenance; do not silently add clocks and discard denial checks. Then migrate
positional compositor traffic and audit runtime wake/upstream adapters. Typed
grant handoffs and intentional raw invalid-authority/custody probes remain
separate from the copied application payload consumers being retired.

Implementation update 2026-10-26: **restart payloads use shared IPC; lifecycle and CPU gates use notifications**, over `53e476f2`.

The supervision restart owner and live client retain a shared coordination
stream; the client establishes a fresh shared connection for each of 32 faulting
service incarnations. All 128 echoes are checked byte-for-byte. The owner still
publishes its command before blocking on child outcome, then collects the exact
pending reply. Invalid wait destinations, stolen launch/child authority, hidden
grants before acceptance, duplicate acceptance, stale names and terminal peer
closure remain asserted. Each faulted connection must return the client's page,
reader, channel and notification usage to its pre-connection baseline before the
next round. The smoke runner requires the shared-incarnation completion marker.

Payload-free ready/start/done coordination in CPU, parent-exit, failed-admission
completion and watchdog completion/idle fixtures now uses one-shot notifications.
The sender publishes the signal before exporting the wait-only grant; queued
custody survives immediate sender exit. No shared page or new clock grant is
needed. Explicit transport budgets cover these gates; workers only receive wait
rights. Existing CPU ceiling, restart-debt, subtree fairness, all-idle deadline,
parent-death cancellation and exact heap/frame reclamation assertions remain.
The failed-admission endpoint self-loop remains an intentional custody probe,
not an ordinary application transport consumer.

Follow the [supervision proof guide](source-rs/distribution/lab/supervision/README.md)
for the userspace roles and boot composition. Host tests, supervision and smoke
Clippy, layout/format checks and release/development smoke pass, including 65/98
display admission failure boundaries respectively. Logs are under
`build/omega-component/supervision-shared-*.log`.
This is x86 fixture work; no F5 or ARM64 behavior change is claimed.

**Previous next (sibling/graph portion completed above):** migrate the remaining watchdog recovery exchange, sibling-service and
link-graph fixtures. Preserve denied-clock/device checks and replies held across
other children's cancellation. Then migrate positional compositor clients and
audit runtime wake/upstream adapters before removing copied payload support.
Typed grant setup and deliberate invalid-authority probes remain distinct.

Implementation update 2026-10-26: **direct display and its coordination use shared IPC**, over `6cfa53a0`.

The immediate display provider selects shared owner control and any supplied
client link. The distribution display fixture retains one shared provider client
per incarnation plus separate supervisor/client and supervisor/observer streams.
Observer completion is an explicit request/reply; display mapping probes use a
funded shared owner/peer completion stream too. Only intentional stale raw-handle
send/receive probes remain in this lab. The drawing library now accepts the
existing retained connection; unused copied drawing-session helpers are removed.
Follow the [display proof guide](source-rs/distribution/lab/display/README.md) across
the executable boundaries.

The first provider still faults on its ninth application request, after malformed
packet/geometry refusals and pattern requests. Setup traffic does not count as an
application request. Its replacement answers INFO and remains connected until
supervisor cancellation; the client then checks shared peer closure. Both rounds
retain the original 200-tick completion deadline and independent observer progress.
The old aggregate wait-timeout/copied-send assertions no longer describe this
transport: shared clients have maintenance waits while retaining their exact call.
Actual child outcomes, deadline checks, old-name denial and both progress replies
remain directly asserted. No provider deadline was relaxed.

The smoke graph explicitly funds two mailbox writer/reader/notification units at
the supervisor and client, and one at the observer/provider. Mapping probes fund
their completion stream independently of the tested device grant. Final owner
retirement checks zero private/reader/channel charges and notification resources.
The final-reply sender deliberately yields before dropping its reader, and the
supervisor bounds its wait for actual reclamation rather than assuming immediate
refund on reply receipt. This preserves accepted-reader lifetime.
The host still checks exact heap/frame reclamation, spawn/reap/cancel counts,
ungranted mapping faults, NX/guards and every device-admission frame boundary.
The runner requires shared setup, both provider incarnations and retirement markers.

Verification is in `build/omega-component/display-shared-*.log`: host tests,
display lab Clippy, all-feature provider Clippy, smoke Clippy and release smoke
pass. Release and dev smoke pass the final-reply/reader-release boundary and
exercise 65 and 98 display admission boundaries respectively. Normal F5/cold persistence passes in
`desktop/20261026T165021924598Z`, including exact verification of 1,631 host records.
The direct `recovery-lab`
hooks now reply through the selected server transport and are compile-checked;
this smoke composition selects `lab`, not those optional hooks. No ARM64 runtime
change is claimed; this fixture is x86-only.

**Previous next (restart and gate portion completed above):** migrate the supervision fixtures and positional compositor clients,
including their actual pressure/backpressure and lifetime tests. The latter still
use copied owner, input-WAIT and presentation traffic; do not confuse them with
the already-shared normal F5/root/nested-matrix route. Then audit positional
runtime wake/upstream adapters before retiring copied payload support. Keep typed
grant setup and deliberate invalid-authority probes distinct from application work.


Implementation update 2026-10-26: **direct object-store proofs use retained shared connections**, over `c3d2fce6`.

Both `distribution/lab/objects` clients now retain shared-memory connections to
their separately scoped provider roots. Inline reads, malformed scope fields and
immutable publication/bulk attachments use the same connection owner. The
supervisor retains shared HEALTH/CLOSE control for the provider's entire lifetime;
there are no raw application-payload send/receive calls left in this lab. Typed
setup and attachment transfers remain kernel-mediated capability operations.

The two profiles explicitly fund mailbox writers/readers and notifications. Four
retention data-reader pages remain available beside the mailbox: two retained
bulk replies charge five readers, a third is refused with NO_MEMORY, and dropping
the two replies leaves exactly one. Final provider collection/control retirement
must leave the supervisor with zero writer/reader/channel charges and zero
notification objects/rights. The host requires shared-root and shared-control
markers on every boot, alongside the original persistence/refusal checks.

Verification also exposed older dev/startup problems, now repaired:

- The provider's image/tables already charge 300 frames. The initial 67-page
  private grant derived 355 physical frames, leaving only 55 for a 56-page heap.
  Both fixture grants are now 83 pages: the original heap/output capacity, three
  mailbox writers and 16 pages of extra startup funding. No kernel limit changed.
- The dev provider had grown to 262 load pages against the 240-page cap. Dev-only
  size optimization of the engine/content packages, contracts and allocator,
  plus one runtime codegen unit, restores 238 pages. Runtime assertions remain
  enabled; existing engine/provider assertion settings and release settings are
  unchanged.
- An inline notification wait array inflated nested IPC/session constructors
  until the dev boot stack faulted. The wait arena is now fallibly preallocated
  on the heap for all configured slots, including dormant children. Notification
  operations never grow it. Measured x86 dev frames for grant composition,
  preparation and IPC construction fell from 33,752/14,568/17,192 bytes to
  23,496/8,712/8,408. This is local evidence, not a global stack-use bound.
  The existing stack capacities/guards and startup failure rollback remain.
- The mapping rollback smoke fixture used `page as u8 + 1` after its coverage
  grew to 256 pages. Dev overflow checks correctly rejected page 255; the fixture
  now explicitly wraps its byte pattern on both write and verification. Mapping
  checks still cover every page and every alias slot, including guards.

The protocol harness now fails immediately on `CATHEDRAL_RS_FAULT`; a halted fake
guest regression verifies it reports the fault instead of waiting for a timeout.
Verification (`build/omega-component/objects-shared-*.log`):

- Host tests, base/retention lab Clippy, x86/ARM64 smoke Clippy, layout, format,
  whitespace and the harness fault regression pass.
- Publication/refusal/recovery passes two cold boots in release and dev; durable
  pins/collection/bulk reader pressure passes three in each. Exact host records
  are 24/32 for publication and 34/24/24 for retention. All runs preserve neighbor
  guards, report no heap allocation failures and reclaim final owner transport.
  The [lab guide](source-rs/distribution/lab/objects/README.md) records artifact IDs.
- Full x86 dev smoke passes, including 102 startup heap refusals with exact
  reclamation, every page-allocation boundary and the repaired mapping fixture.
  Full ARM64 dev portability passes, including 134 admission rollback boundaries.
- Full release IPC passes, including shared transport races, recursive admission
  and reclamation. Normal F5 and cold persistence pass in
  `desktop/20261026T163518576499Z`, with exact verification of 1,631 host records.
  Ordinary release service behavior and storage representation are unchanged.

**Previous next (direct display completed above):** migrate the direct display/supervision application consumers, preserving
fault/restart, stale-handle and peer-closure evidence. Their bootstrap coordination
still sends copied payloads; the restart fixture also deliberately probes stale
raw handles, so distinguish those probes from normal service traffic. Audit the
runtime's positional wake/upstream adapters before removing copied payload ABI
support. Launch-established Matrix connectors remain a separate open outcome.


Implementation update 2026-10-26: **stock matrices and headless leaves have shared data/control only**, over `339f086c`.

Headless roots now offer the same brokered, separate interaction/storage shape as
the desktop. The leaf establishes shared owner control before acquiring services,
asserts a notification input wake, and retains its interaction and storage clients.
Stored-code launches declare and receive that bootstrap too. Publication still
constructs sealed backing locally; malformed scope/deadline fields still reach the
real storage boundary unchanged. No leaf receives a world lookup or hardware grant.

The stock matrix's copied parent interaction, combined parent storage and copied
child-control variants are removed, along with the composition flag and combined
stream blocking/retirement branches. Interaction always retains its two correlated
lanes; storage failures retire only storage. Child control keeps at most one late
request, represented by a boolean rather than an eight-reply FIFO count.

The former eight-unanswered-LOSS fixture now holds one shared request for 1,000
ticks. Nine owner surveys cannot enqueue another request; the leaf observes no new
control work while holding it. Its late clean reply is drained before later dirty
observations. Existing malformed LOSS, delayed replies, missing control, child
exit, sibling close, scope revocation and stored-code recovery checks remain.
`Server::establish_control` owns the existing setup-only sequence used by both
`OwnerControl` and this adversarial server; it rejects copied application requests
during setup without consuming valid application work.

Funding is explicit in the two headless profiles: leaves reserve three transport
writers/notifications plus a publication page, and four readers; a nested matrix
funds three upstream and three downstream connections. Storage forwarding also
needs its attachment readers. Dynamic leaves now receive four writer/reader pages
and three notification allowances. The storage root reserves 345 readers, including
the existing loader policy's 256-reader forwarding headroom. This is a capability
ceiling, not eager allocation. Kernel limits are unchanged. Initial failed runs
identified insufficient attachment-reader funding, then the loader reserve; no
refusal was bypassed or timeout relaxed.

Verification (`build/omega-component/fixture-shared-*.log`):

- Host tests, headless/storage/all-feature Clippy, layout, format and whitespace
  checks pass.
- Headless input/storage suites pass, including exact shared leaf/parent and
  held-LOSS markers, teardown accounting, raw malformed requests, independent
  roots, stored-code launch, sibling-close races and durable recovery.
- Unpressured matrix desktop and cold boot pass (27.56 s to scene readiness;
  nested key-to-scanout observed at 219 ms). CPU-capped desktop and cold boot also
  pass (115.50 s to readiness; observed key-to-scanout 1,093 ms). These are measured
  test observations, not latency guarantees.
- Normal F5 and cold persistence pass in `desktop/20261026T160310790021Z`, including
  independent Notes instances and exact verification of all 1,631 host records.
- Full x86-64 IPC and ARM64 portability suites pass. No kernel implementation or
  ceiling changes; the runtime setup helper extraction uses the existing protocol.


**Previous next (object-store subset completed above):** audit direct mechanism consumers before retiring copied application
payloads from the runtime/ABI. Start with `distribution/lab/display`, `objects`,
`supervision`, and positional wake/upstream adapters in `user-runtime`; distinguish
actual payload consumers from typed setup/grant transfers and intentional hostile
transport probes. Preserve their refusal, lifetime and accounting evidence. The
launch-established Matrix connector is still an open outcome, not implied by the
current broker-pair migration.


Implementation update 2026-10-26: **headless fixture owners retain shared control**, over `e493d887`.

The common `distribution/desktop/fixture.rs` now owns a shared `Client`; all
scenario/storage commands borrow it mutably, preserving request correlation and
the existing 300-tick proof deadline. Idle observation does not consume raw replies.
The storage fixture's initial and recovered providers also retain shared control,
validate exact `ready` replies and answer health after the recovered data checks.
Recovery collects the old tasks and retires both old clients before replacement,
asserting zero owner writer/reader/channel charges and notification objects/rights.
Recovered clients remain alive for the observation loop: the first migrated run
exposed premature provider-control drop on helper return, which signalled owner
departure even though its recovery assertions had passed.

Profiles explicitly fund each new control stream's writer, reader and notification.
The input owner has one of each, the storage owner two; root/provider allowances
add one each. No kernel limits, runtime or ordinary desktop behavior changed.

Verification (`build/omega-component/fixture-control-*.log`): host tests,
headless/storage/all-feature desktop Clippy, layout, format and whitespace checks
pass. Both `--matrix-test --release` and `--matrix-storage-test --release` pass
with the exact shared-control and teardown-accounting markers. Artifacts are in
`build/boot-harness-rs/matrix-test/` and `matrix-storage-test/`. Existing locality,
revocation, malformed/late LOSS, producer capacity, sibling launch/close and durable
recovery checks remain enabled. F5 and ARM64 were not rerun: these implementation
changes are confined to non-desktop fixtures and their two test profiles.

**Previous next (completed above):** migrate the combined nested matrix data and `desktop/leaf.rs` relay to
separate shared interaction/storage and shared child control, then remove the
copied parent branches. Preserve the stale/late LOSS, malformed reply, missing
control and sibling-close tests. The old eight-unanswered-reply fixture encodes
copied FIFO behavior; replace it with explicit retained-request/backpressure
coverage, rather than silently dropping that scenario or pretending the shared
single-outstanding control client queues eight requests. The newly ratified
launch-established Matrix connector remains a separate open outcome above.


Implementation update 2026-10-25: **storage-stall input injection retains shared source custody**, over `bcf8aa41`.

The desktop lab now takes its untouched source pair into one retained `Client`.
That connection rejects SPAWN, READ_PIXEL and BIND, then forwards pointer/key
injections throughout blocked-storage, provider replacement and sibling-death
checks. The original deadline and proof that a sibling consumes input/repaints
before storage recovery remain in place. There is no copied source-request helper.

Admission records the untouched pair as optional custody: failed driver spawn
retains it, successful spawn transfers it, and the lab can instead consume it.
After the source fixture, an attempted driver admission must return DENIED without
creating a task. The lab then collects its disposable tree and rebuilds from disk
with a fresh source for physical input. Established mailbox/request state never
moves across a process boundary as bare endpoint aliases. This costs one extra
root/display/storage admission in the destructive lab only; ordinary F5 and cold
restoration take no additional rebuild. Existing page/channel grants suffice.

Verification (`build/omega-component/source-shared-*.log`): the
unpressured and CPU-capped matrix desktop and cold boots pass, including
consumed-handoff denial, all storage-stall/lifetime proofs, driver-source loss and
persistent colors. Under pressure, the scene became ready after 114.31 s and a
nested key reached verified scanout in 1,359 ms; these are observations, not a
latency guarantee. All ten input bootstrap cases pass in
`bootstrap/20261026T050051019782Z`: death, stall, malformed reply, refusal and bad
readiness, each with one-shot recovery and repeated terminal failure. Their
process/frame/page/channel/notification rollback checks pass, and the source disk
remains unchanged. The final unpressured matrix run became ready after 26.78 s,
with nested key-to-scanout observed at 203 ms; its cold boot also passes. Unmodified
F5 and cold persistence pass in `desktop/20261026T050321897166Z`, including exact
verification of all 1,633 host records. Host tests, lab/all-feature desktop Clippy
and layout/format checks pass. No kernel/runtime implementation change; ARM64 was
not rerun.

The first pressure run timed out on the recovery fixture's initial save after
rebuilding the tree. That fixture assumed a warm app from the preceding probes;
geometry admission alone did not establish readiness. It now waits for both
expected initial app frames before issuing the save. The capped rerun passes
without increasing the storage deadline or changing resource grants. The original
failure is retained in `source-shared-pressure-startup-failure.log` and its serial
log alongside the passing rerun.

**Previous next (owner controls completed above):** remove the remaining combined matrix stream by migrating the headless
`distribution/desktop/fixture.rs`, scenario/storage suites and leaf relay to
separate shared connections. Then simplify `matrix/service/parent`'s copied
interaction/storage branches. Retain locality, revocation and malformed-input
checks; a passing ordinary desktop does not cover those fixture contracts.


Implementation update 2026-10-25: **Rust chooser probes use retained shared services**, over `496542c5`.

The Rust picker/launcher fixture now declares the same three service connections
as the ordinary Omega chooser and receives `Services` at its entry. Its named
session establishes owner control first, then shared interaction, notification
input wake and shared storage. Realm listing/copying, settings persistence,
management, task selection, layout and immutable images retain those clients.
No chooser-session data path creates raw copied requests or receives replies from
an alias of an established connection. The normal installed chooser stays Omega.

The desktop harness's explicit `--rust-chooser` variant exercises both picker and
user launcher on a fresh disposable composition and checks the selected startup
markers. Its runtime marker follows an assertion that the wake is a notification.
Replacement-launcher and management tests also require the shared chooser marker.
This is a transport migration, not a new locator, authority policy or UI design.

Verification (`build/omega-component/chooser-shared-*.log`):

- Fresh/cold Rust chooser desktop passes in `desktop/20261026T044019871853Z`,
  including user creation, picker retirement, independent Notes instances, slot
  reuse and exact verification of all 1,595 persisted host records.
- All seven launcher recovery modes pass in
  `launcher-recovery/20261026T044103617641Z`: recovery/cold persistence, clean
  exit, startup fault, owner stop/revocation and fallback clean/fault. Replacement
  retains settings/layout authority; pending app drafts survive repeated crashes
  and explicit retries. The source disk remains unchanged.
- Management recovery passes in `management/20261026T044310096528Z`, including
  hung-app termination, stale/ungranted/revoked authority, replacement and cold
  persistence; its source disk remains unchanged.
- All five root startup cases pass in `startup/20261026T044421327714Z`, including
  picker fault after shared setup, bounded retry and preserved disk contents.
- Unmodified F5 and cold persistence pass in `desktop/20261026T044500943125Z`,
  with exact verification of all 1,633 host records.
- Host tests, Rust-probe/all-feature chooser Clippy, layout/format and whitespace
  checks pass. No kernel/runtime change; ARM64 was not rerun.

**Previous next (source injection completed above):** migrate explicitly combined/copied fixtures and the pre-driver source
injection before retiring copied application payloads. Source stall tests currently
inject through an untouched pair before handing it to the driver; upgrading that
pair and passing only bare endpoints would discard retained protocol ownership.


Implementation update 2026-10-25: **Rust Notes probes retain shared data connections**, over `4e3b8190`.

The disposable Rust Notes session establishes owner control first, then retains
shared broker/interaction and optional storage clients. Immutable image publication,
cursor preferences and scoped input-directory reads use the same interaction
owner. Management-denial checks exercise both real connections before moving the
storage client into `Scope` without resetting correlation. Installed Notes remains
Omega-authored. The normal probe funds three shared streams; its optional producer
variant funds four and acquires no authority beyond its explicit manifest/offer.

Producer parts now pass through the broker's existing typed grant/correlation
owner (`acquire_endpoint_part`). This low-level operation transfers one part,
does not populate a complete-service cache, and must not be mixed with complete
acquisition of that offer. The partial fixture intentionally retains just the send
half and attempts an early copied message before the reply half has transferred.
Complete acquisition selects a shared producer connection, checks wrong-service,
forged-group and stale-grant rejection, then closes it while ordinary app services
remain usable. Stock admission reserves the additional owner-side transport cost.

Guest testing exposed and fixed two issues:

- Shared producer teardown had already closed its endpoints; the old copied-path
  cleanup treated `BAD_HANDLE` on the second close as a fatal matrix error.
  Disconnect and child collection now handle retired incarnations like storage.
- At three live Notes instances the stock matrix held 62 of 64 notification rights.
  Setup needed three free rights despite ample pages, channel funding and frames.
  The separate metadata ceiling is 96, with explicit setup headroom; page/object
  funding is unchanged. The capacity lab's initial channel grant is 96 so its
  metadata-exhaustion test can still reach that independent ceiling after lending
  peer funding. Refused shared setup now reports these resource dimensions.

Verification (`build/omega-component/notes-shared-*.log`):

- Complete, partial and denied producer acquisition pass fresh/cold desktop flows:
  `desktop/20261026T042643664711Z`, `20261026T042758959915Z` and
  `20261026T042843073348Z`. Each checks three live Notes instances, independent
  drafts, protected close, slot reuse and exact persisted state.
- Receiver hold/replacement and directory identity checks pass fresh/cold in
  `desktop/20261026T042949389393Z`; hidden input, deferred painting and restore
  pass in `desktop/20261026T043028038068Z`.
- Delayed lifecycle replies, live editing, stale clean-answer draining, fresh
  dirty-close refusal and clean close pass fresh/cold in
  `desktop/20261026T043231425731Z`.
- Full IPC and ARM64 release guests pass. IPC keeps twenty shared streams live,
  reaches the 96-right metadata ceiling independently of object funding, refuses
  excess creation without partial grants, continues exchanging and refunds custody.
  The two-free-slot case also refuses three-right creation without partial grants.
- Management recovery passes in `management/20261026T043346262144Z`, including
  both service-denial connections, hung-app termination, revoked/stale authority,
  launcher replacement and cold persistence; the source disk stays unchanged.
- Unmodified F5 and cold reboot pass in `desktop/20261026T043521892304Z`, with
  exact verification of all 1,633 host records.
- Host tests, Notes (Rust probe/producer/all-feature), matrix and channel-lab Clippy,
  layout and format checks pass. No new owner decision or latency guarantee is claimed.

**Previous next (chooser completed above):** migrate Rust chooser probe data, then the combined-stream fixtures and
pre-driver source-probe handoff before retiring copied transport. The source pair
cannot be upgraded and handed off as bare endpoints. Authorized broker lookup
remains a separate outcome; these changes preserve the supplied relationship.

Implementation update 2026-10-25: **modern catalogue launches acquire services and notification wakes through the broker**, over `8586acb4`.

Stock hosts with separate storage now use brokered admission for supplied launch
and static-catalogue RUN requests. They offer the same interaction/storage rights
as the old five-endpoint bundle; local view/storage grants are still installed
separately. Nested lab matrices and Colors/Omega Notes acquire those connections
and a wait-only notification through the existing authenticated service offer.
This reuses ordinary desktop acquisition and cleanup, not a new locator or payload
transport. Combined-stream mechanism fixtures and stored legacy probe images
still select their positional paths. Stored-image manifest admission is unchanged.

The boot-bundled recovery program now receives `Services`, establishes independent
owner control first, then acquires shared interaction and a notification wake.
POLL, retry requests, screen commands and immutable card images all use that same
retained interaction owner. The startup owner gives it 66 private pages, two
readers and two notification channels, retaining the old two-card capacity.
Launcher fallback admission offers only INTERACTION and two channels within the
retained launcher budget. It verifies storage acquisition is unavailable and that
its interaction connection cannot query management, layout or storage authority.
Boot-catalogue admission retains its previous unbound storage offer; recovery does
not acquire it. No realm access is added by discovery.

Colors requires a notification wake and checks actual kernel refusals when it
tries to signal or close that wait-only right. Physical input and nested painting
then use the same right. Admission/rollback, child death and subtree replacement
now exercise retained broker offers instead of copied wake queues in the integrated
labs. The notification replaces the previous wake queue at the same object charge;
Colors/Notes lab budgets and top-level F5 grants are unchanged.

Verification:

- F5 and cold persistence pass in `desktop/20261026T040640555071Z`, including
  the shared recovery marker and the exact 1,631-record host oracle.
- The destructive desktop and cold reboot pass in `matrix-desktop-test/`, with
  notification wakes in both Colors instances and brokered startup in the nested
  matrix. Existing image, input, blocked-save and subtree recovery proofs remain.
- Notes recovery and cold persistence pass in
  `matrix-database/20261026T040809922279Z`, including exact text/history/quota
  (20 host records each boot) and the nested broker path.
- The pressure desktop and cold boot pass in `pressure-matrix-desktop-test/`,
  including wait-only authority refusals and actual input afterward. The sampled
  nested key reached scanout in 1,312 ms under the deliberate CPU cap; ordinary
  unpressured input sampled 218 ms. These are observations, not latency guarantees.
- Launcher retry, clean fallback exit and fallback fault pass in
  `launcher-recovery/20261026T041309880041Z`; the source fixture stays unchanged.
- All six provider-startup cases pass in `provider-startup/20261026T041423449561Z`,
  including malformed setup/readiness, stalls, repeated crashes and invalid media.
  Recovery remains visible and explicit retries/cleanup preserve the original disk.
- Database recovery and cold persistence pass in
  `matrix-database/20261026T041618018960Z`, with exact revisions [4, 2] and
  all 18 host records on both boots. Three Colors instances acquire notification
  hints, including the independently interactive peer.
- Host tests; recovery/matrix, positional and all-feature desktop Clippy;
  layout/format and whitespace checks pass. No kernel/runtime implementation changes;
  ARM64 was not rerun. Logs: `build/omega-component/catalogue-broker-*.log`.

**Previous next (Notes completed above):** migrate the remaining Rust Notes/chooser probe data adapters, then the
combined-stream fixtures and source-probe handoff before retiring copied transport.
The pre-driver source still cannot be upgraded and handed off as bare endpoints.
On-demand authorized broker lookup remains a separate outcome; this change still
uses the existing explicit startup relationship.

Implementation update 2026-10-25: **Rust Colors lab rendering and storage use retained shared clients**, over `50ac8625`.

The retained Rust Colors probe now upgrades its supplied interaction connection
through `Services` and owns a separate shared storage `Client`. Surface commands,
compact transactions, immutable images and close-button backing use the same
interaction owner. Helpers borrow it mutably; no raw endpoint alias sends or
receives payloads behind the shared connection. Owner control remains independent.

The storage client first exercises denied VIEW and POINTER_SOURCE requests.
`Scope::from_client` then adopts that exact idle client, preserving request IDs
and custody through binding discovery and subsequent object operations. It rejects
an in-flight client rather than relabelling its pending work. `Scope::connect`
uses the same constructor. The app still reconciles uncertain writes explicitly;
no timeout causes a copied fallback or automatic mutation replay. Interaction
uses the existing retained-reply service API; storage keeps its existing budget.

The Colors leaf is funded at 5 private pages / 3 readers / 3 notification channels,
its volatile peer at 4/3/3, its nested matrix at 19/19/25 and its root at 45/37/49.
Both ends of each added interaction/storage stream are funded, beside retained
image and publication backing. Notes and top-level F5 grants are unchanged; there
are no kernel changes. The installed F5 Colors remains the Omega app; this is the
adversarial Rust lab consumer, not a second installed implementation.

Verification:

- The destructive matrix desktop and cold reboot pass in `matrix-desktop-test/`,
  including staged/rejected presentation, forwarded sealed images, subtree death,
  blocked persistence and exact saved colors. Cold boot observes two Colors instances with
  shared interaction and storage; the first boot includes the replacement probe instances.
- Database recovery and cold persistence pass in
  `matrix-database/20261026T035745668627Z`, with exact revisions [4, 2] and all
  18 host records on both boots. Three shared Colors instances include the
  independently interactive peer; both wrong-service refusals execute on shared IPC.
- The pressure desktop and cold reboot pass in `pressure-matrix-desktop-test/`,
  including late-reply reconciliation, resource/subtree reclamation and physical
  input. The sampled nested key reached verified scanout in 813 ms under the
  deliberate CPU cap; this does not establish ordinary-desktop performance.
- F5 and cold persistence pass in `desktop/20261026T040007175059Z`, including
  the exact 1,631-record host oracle. The installed Omega app uses the same
  `Scope::connect` constructor as before; its retained client ownership is preserved.
- Host tests, positional and all-feature desktop Clippy, layout/format and
  whitespace checks pass. ARM64 was not rerun. Logs: `build/omega-component/colors-shared-*.log`.

**Previous next (recovery data and modern catalogue wakes completed above):** migrate remaining Rust Notes/chooser/recovery data adapters and positional
wake delivery, then retire their copied consumers. Pre-driver source probes still
share an untouched endpoint pair with the later physical driver: upgrading that
pair and passing only raw handles would lose the retained connection state. That
handoff needs an explicit ownership-preserving migration. Direct mechanism fixtures
and intentional malformed-setup tests remain distinct from ordinary application
payload transport. On-demand authorized broker lookup remains separate accepted work.

Implementation update 2026-10-25: **positional desktop children use shared lifecycle control**, over `40d0a130`.

Stock admission now selects shared control for separate-storage compositions as
well as brokered children. The integrated Colors/Notes labs and their nested
matrices therefore use the existing yielding control client, including exact
late-reply custody, instead of the copied FIFO adapter. Older single-connection
mechanism fixtures still select that adapter. This includes children without their
own storage grant: control selection follows the modern host, not that child's
storage entitlement. Ordinary brokered apps already used shared control.

The Rust Colors probe, optional Rust chooser probe and boot-bundled recovery
screen use `OwnerControl`, as Notes already does. The Colors save probe acknowledges its current shared request before
blocking in storage; subsequent liveness observes completion. Neither probe reads
or writes raw control payloads behind the retained server.

The first pressure run exposed startup ordering: Colors could finish its initial
paint after the owner's shared setup had expired, leaving a live app with unusable
control. `OwnerControl::establish` now permits an explicitly selected shared leaf
to complete setup before costly startup. Colors calls it before service requests;
the Rust chooser before loading its realm, and recovery before its service work. It advances only setup, checks readiness
before reading the established stream, rejects unexpected copied payloads and
retains no application request. Both peers keep their existing 300-tick deadline.
Use it only where the owner initiates setup independently of app readiness.
The failed pressure serial is preserved in
`build/omega-component/lab-child-control-pressure-startup-failure-serial.log`.

Funding includes both ends, not just the app's writer: each leaf gets one extra
private page, reader and notification; each nested lab matrix also funds its own
control server and two child control clients. Colors uses root 33/25/37, inner
11/11/17 and leaf 3/1/1; Notes uses root 153/153/45, inner 59/51/21 and leaf 43/7/3
(private/readers/channels). The volatile sibling receives 2/1/1. The F5 boot-recovery child receives
65/1/1, preserving its old two-card backing; a launcher fallback receives one
notification channel from the retained launcher budget. Top-level F5 grants and
kernel limits are unchanged. Early setup can complete for short-lived probe children
before they are collected, so first-boot child setup counts are a lower bound;
functional control/recovery checks still run, and the cold desktop has exactly
three established child links.

The first F5 check exposed the boot recovery screen's remaining raw receive loop:
it tried to decode OPEN as an application packet. Recovery now retains shared
request custody, including while awaiting collection after an acknowledged retry.
Startup/launcher failure paths are included in the final checks below.

Verification:

- F5 and cold persistence pass in `desktop/20261026T034219955389Z`, including
  the exact 1,629-record host oracle and no recovery-screen panic.
- Launcher recovery, fallback clean exit and fallback fault pass in
  `launcher-recovery/20261026T034346951434Z`; the source fixture stays unchanged.
- The final pressure desktop and cold boot pass in
  `pressure-matrix-desktop-test/`, including all destructive guest controls,
  sibling activity during blocked persistence and exact saved colors. The sampled
  nested key reached verified scanout in 922 ms under the deliberate CPU cap;
  this is not an ordinary-desktop performance claim.
- Notes recovery passes in `matrix-database/20261026T033105028048Z`, including
  physical input, preserved drafts and both exact 20-record host checks.
- Database recovery and cold persistence pass in
  `matrix-database/20261026T034810334748Z`, with exact revisions [4, 2] and
  18 host records on both boots. Its startup-only peer now establishes shared
  control before painting, so all four child setups are observed even on cold boot.
- All six provider-startup cases pass in `provider-startup/20261026T034503741769Z`:
  crash-once, bad-ready-once, bad-setup-once, stall-once, repeat-crash and bad-volume.
  Visible recovery, explicit attempts, cleanup and unchanged rejected media remain checked.
- The final unpressured destructive desktop and cold reboot pass in
  `matrix-desktop-test/`, including setup counts, actual app controls and exact
  saved colors. The pressure failure above was corrected before these final runs.
- Host tests; all-feature desktop and recovery Clippy; positional desktop,
  matrix and Rust-chooser Clippy; format/layout and whitespace checks pass.
  The optional Rust chooser was compiled, not newly exercised in QEMU. Kernel
  code is unchanged; ARM64 was not rerun. Logs: `build/omega-component/lab-child-control-*.log`.

**Previous next (Colors data completed above):** migrate remaining positional app data/wake adapters and pre-driver
source probes. Direct storage/input/display mechanism labs also retain copied
consumers. Preserve malformed-setup and kernel-mediated grant tests. On-demand
authorized broker lookup remains a separate accepted outcome.

Implementation update 2026-10-25: **positional provider supervision and fault gates use shared control**, over `e4c40757`.

`desktop/session/persistence/provider/` now has only shared Opening/Ready states.
Ordinary F5 still yields during startup; positional admission polls the same owner
and parks on its current setup or reply interests until the existing readiness
deadline. HEALTH, fixture commands and later watchdog replies use one correlated
client. The copied-control accessor and its raw-pair Drop path are removed.
Retirement still collects the exact child before releasing control custody.

`platform/services/storage/gate.rs` no longer sends unsolicited blocked payloads
or reads the bootstrap endpoint behind Server. Arming, observing state and release
are explicit owner requests; `desktop/lab/gate.rs` is their fixture client. At the
selected before-execution or post-commit boundary, Gate retains the in-flight data
request, pauses both data slots, and dispatches only owner control through Server.
State queries report blocked; release replies before unpausing and resuming the
original store request. HEALTH deliberately receives no reply, preserving the
watchdog's nonresponsive-provider tests. No data operation executes during this
pause and no application reply bypasses shared correlation. The redundant quiet
arm opcode is removed; all arms use the same non-unsolicited protocol.

Positional owners now have four writer pages, three readers and eight channel/
notification allowances because storage starts before root/display admission.
Each positional storage provider receives 131 private pages, 67 reader pages and
three notifications, preserving prior data/heap funding beside the new control
stream. Ordinary F5 funding, kernel limits and runtime implementation are unchanged.
The fault/trace storage image reaches the existing 240-load-page admission cap
(ordinary F5 is 238); further service growth needs code-size review, not an
unexamined ceiling increase.

Verification:

- The destructive desktop and cold reboot pass in `matrix-desktop-test/`, with
  five shared provider incarnations across recovery and one on cold boot.
- F5 and cold persistence pass in `desktop/20261026T031225293141Z`, including the
  exact 1,621-record host oracle.
- Database recovery passes in `matrix-database/20261026T031318519714Z` with
  revisions [4, 2] and 18 exact host records; four provider incarnations on the
  first boot and one after cold restart.
- Notes recovery passes in `matrix-database/20261026T031344763612Z`, with both
  exact 20-record checks, two provider incarnations during failure/recovery and
  one on cold boot.
- All six live-provider cases pass in `provider-live/20261026T031423594096Z`:
  precommit, postcommit, exhausted retries, stalled replacement readiness,
  nested and deep. Draft protection, explicit recovery, retained-root resource
  baselines and cold persistence remain checked.
- All six startup cases pass in `provider-startup/20261026T031942737098Z`,
  including malformed transport versus malformed HEALTH, stalls, repeated
  crashes and invalid volume. Failed attempts release control resources before
  the explicit retry; failed-volume/repeated-crash media stay unchanged.
- The pressure desktop and cold reboot pass in `pressure-matrix-desktop-test/`,
  including all five shared provider incarnations, sibling interaction while
  storage is blocked, loss/rebuild and the established-focus precondition.
- Host tests, all-feature desktop/storage Clippy, positional desktop Clippy,
  format/layout and changed Markdown links pass. No kernel/runtime implementation
  changes; ARM64 was not rerun. Logs: `build/omega-component/lab-provider-*.log`.

**Previous next (child control completed above):** migrate positional app transport/wake adapters and pre-driver source
probes. Other direct storage/input/display mechanism labs also retain copied
consumers. Raw malformed-setup tests and kernel-mediated capability handoffs
remain intentional; they are not ordinary application payload fallback. The
on-demand authorized-broker lookup outcome remains separate accepted work.

Implementation update 2026-10-25: **positional desktops share F5's root/display transport owners**, over `9b762621`.

The copied root-control and display-configuration adapters are removed from
`desktop/session/`. Both ordinary F5 and positional desktop labs open separate
shared display configuration, hand untouched presentation endpoints to the root,
check root HEALTH, and retain one `RootControl` for the session. Static lab labels
use the narrow boot-display configuration port. Every lab operation borrows the
retained root client; no raw-pair alias receives or publishes its payloads.

Idle/timeout probes now wait on that client's lifetime endpoint, leaving shared
reply correlation and notification draining with its owner. Display death uses
a real shared request, expects peer closure and collects the failed root before
rebuilding the tree. The old copied source-probe helper is explicitly located in
`desktop/lab/source.rs`: it only touches the pre-driver source pair. Migrating
that producer must preserve the later physical driver's ownership; upgrading it
and passing bare endpoints would lose transport state. Storage control/fault gates
are also still copied, as are positional app adapters/wakes.

Each positional owner has three writers, two readers and seven channel/notification
allowances, funding root control, input readiness and possibly still-retiring
display configuration beside four transferred channels. Display receives one
additional writer, reader and notification for configuration. Each root receives
one additional writer/reader/notification, preserving its old child/image capacity:
Colors root 25/17/29; Notes root 145/145/37. F5 budgets and kernel limits are unchanged.
The database-recovery profile was below its admission constants and relied on
borrowing the difference from the initial owner; explicit owner budgets exposed
that mismatch as BUSY before root admission. Its profile now matches the actual
25/17/29 admission limits.

The pressure fixture exposed a precondition race: it injected a focus click and
killed the source before the nested app necessarily consumed that click.
Cancellation may discard undelivered input. The test now observes both click
events through the app's existing pointer counter before source death, so its
later key checks preservation of an established selection rather than scheduling
luck. No production input semantics or timeout increase is used for that check.

Verification:

- F5 and cold persistence pass in `desktop/20261026T025520349313Z`, including the
  exact 1,621-record host oracle.
- The database-recovery workload and cold reboot pass in
  `matrix-database/20261026T025723711955Z`, preserving the two objects' exact
  revisions [4, 2] and all 18 host records. The prior profile-mismatch failure is
  `matrix-database/20261026T025611587096Z`.
- Notes recovery passes physical input, draft preservation, lost-reply recovery
  and both exact 20-record checks in `matrix-database/20261026T025750601187Z`.
- The pressure matrix desktop passes nested clipping, storage isolation, subtree
  recovery, source/display death and cold persistence in
  `pressure-matrix-desktop-test/`. Final assertions observe three shared root/display
  incarnations across recovery, one on cold boot, and delivered focus before
  source loss. The original failed focus run is preserved in
  `build/omega-component/lab-control-pressure-focus-failure.log`.
- The final unpressured destructive desktop and cold rerun pass in
  `matrix-desktop-test/`, with the same exact transport incarnation checks and
  delivered-focus precondition.
- Host tests, all-feature and positional desktop Clippy, format/layout and changed
  Markdown links pass. Kernel and runtime implementations are unchanged; ARM64
  was not rerun. Logs: `build/omega-component/lab-control-*.log`.

**Previous next (provider portion completed above):** migrate copied provider-control/fault-gate consumers, then the remaining
positional app transport/wake adapters and pre-driver source probes. Preserve
hostile/setup tests that deliberately use malformed raw bytes. Kernel-mediated
capability installation remains necessary even when ordinary payload queues are
retired. The on-demand authorized-broker lookup outcome remains separate work.

Implementation update 2026-10-25: **input readiness and source validation use shared IPC**, over `41967919`.

`desktop/session/admission/input.rs` passes untouched source endpoints to the
driver and requests input HEALTH on an independent shared bootstrap connection.
Setup and the exact idle reply share the original 300-tick readiness deadline.
The driver checks denied SPAWN, READ_PIXEL and BIND on its own shared source,
initializes hardware and publishes its initial pointer before serving HEALTH.
No upgraded connection is handed off as bare endpoints.

`platform/services/input/readiness.rs` owns the short-lived startup server. It
flushes replies until owner closure, bounded to 300 ticks, then releases its
shared custody before physical delivery resumes on the independent source.
The steady input loop has no added readiness polling. Server Drop already owns
shared endpoint cleanup; the initial F5 run caught a duplicate-close error that
terminated the driver after HEALTH. Cleanup now tolerates endpoints already
retired by that owner, and closes only any leftover raw startup handles without
intervening allocations.

The input grant funds two writer pages, two reader pages and two notifications.
F5's owner funds four writers, two readers and eight channel/notification
allowances: four transferred channels, root/storage, and bounded outstanding
remote custody from display and input startup. No kernel ceiling changed.
All four positional matrix-input profiles use the same readiness protocol and
explicitly fund their owner and driver's startup stream. Keyboard-only and
direct compositor-input labs retain their existing interfaces.

The bootstrap suite now distinguishes malformed input setup from bad HEALTH
after successful setup, bringing it to 30 cases. Ordinary desktop assertions
require source authority validation, shared readiness and startup retirement.

Verification:

- Normal F5 and cold persistence pass in `desktop/20261026T024007080593Z`,
  including physical input and the exact 1,621-record host oracle.
- All 30 bootstrap cases pass in `bootstrap/20261026T024103314716Z`: malformed
  setup versus bad HEALTH after setup, bounded retry, terminal restart screens,
  unchanged failed-boot storage and complete resource reclamation. The final
  source/readiness/retirement marker assertions also pass across all 30 logs.
- Four recursive desktops pass physical input, protected close, repeated subtree
  reclamation, confined Notes/Colors persistence and cold reopen in
  `desktop/20261026T024705062741Z`.
- The positional Notes recovery lab passes physical input, draft preservation,
  persistence and both exact 20-record host checks in
  `matrix-database/20261026T024844399136Z`.
- Provider bad-setup and bad-HEALTH recovery pass in
  `provider-startup/20261026T024923346492Z`, preserving the retained-root charge
  baseline after display and input startup custody has retired.
- Host tests, input/desktop all-feature Clippy, positional desktop Clippy and
  format/layout checks pass. Kernel and runtime implementations are unchanged;
  ARM64 was not rerun. Logs: `build/omega-component/bootstrap-input-*.log`.

**Previous next (root/display portion completed above):** inventory explicitly copied lab/service adapters and migrate their
application payloads before retiring copied-payload syscall consumers. The
ordinary desktop's bootstrap is now shared, but that is not proof that all
fixture consumers have migrated. The on-demand authorized-broker lookup outcome
remains separate accepted work. No latency improvement is claimed.

Implementation update 2026-10-25: **bootstrap display configuration uses an independent shared stream**, over `dcbf3f11`.

Ordinary display INFO, background, cursor enable and terminal restart text now use
shared IPC. `desktop/session/admission/display.rs` opens the provider's bootstrap
port and retains that Client in the admission Attempt. The separate presentation
pair is handed to the root unchanged; that root opens its own shared stream and
answers HEALTH before the Attempt releases configuration. Setup plus first INFO
share the original 300-tick readiness bound. Positional labs keep the old copied
adapter and static labels; shell labels are supplied by its programs.

The display compositor installs bootstrap as an explicit client slot, separately
from its root control stream. No payload supplies the role and no request identity
is rewritten for replies. `compositor/configuration.rs` owns the narrow INFO,
background, cursor-enable and text dispatch shared with root control. Bootstrap
cannot publish matrix scenes, inject input or grant clients authority. Closing it
retires that stream without stopping the live root presentation connection.
No new public wire command, kernel mechanism or global limit.

Display receives two writer pages, two notification allowances and 514 reader
pages (the old 512-page image headroom plus both possible stream readers). The
boot owner receives three writer pages, two reader pages and seven allowances.
Dropping configuration releases its local reader/rights, but remote readers and
signal rights may retain its writer/notification charge until display processes
closure. Fund that overlap with root/storage explicitly; do not assume local
Drop synchronously returns remote debt. After retirement, storage recovery still
expects the same retained-root baseline as before.

Malformed display setup and malformed INFO after valid setup are separate fault
cases; rejected fixtures exit on peer closure. The bootstrap suite now has 28
cases, and probe builds exercise denied scene/input operations on the new port.
Restart rendering still requires neither a root matrix nor input/storage tasks.

Verification:

- Normal F5 and cold persistence pass with the final funding in
  `desktop/20261026T022420036886Z`: user selection, three independent Notes,
  protected close/slot reuse, Colors and the exact 1,621-record host oracle.
  The test requires one shared bootstrap-display connection per normal boot.
  The earlier implementation pass is `desktop/20261026T021119241722Z`.
- All 28 bootstrap cases pass in `bootstrap/20261026T021824384477Z`, including
  malformed shared setup versus bad INFO/HEALTH after setup, successful explicit
  retries, exhausted restart screens and unchanged storage on failure. Every
  rollback compares the complete guest budget baseline and reports zero reader,
  endpoint, notification-object and notification-right residue. Probe builds
  refuse matrix-scene and physical-input operations on bootstrap display control.
- Four recursive desktops pass dirty-close protection, repeated subtree
  reclamation, confined saved Notes/Colors and cold reopen in
  `desktop/20261026T022505680047Z`.
- Provider bad-setup and bad-HEALTH recovery pass in
  `provider-startup/20261026T022647045072Z`, including the unchanged retained-root
  charge baseline after the earlier display stream has retired.
- `--compositor-test --release` passes isolation/recovery and all 786,432 scanout
  pixels with the standalone copied composition. The positional Notes recovery
  lab passes physical input, drafts, persistence and both exact 20-record host
  checks in `matrix-database/20261026T022729925365Z`.
- Host `cargo test --locked`, desktop/display all-feature Clippy, copied desktop
  lab Clippy, final format/layout checks, local Markdown links and whitespace
  checks pass. Kernel and runtime implementations are unchanged; ARM64 was not
  rerun here. Logs: `build/omega-component/bootstrap-display-*.log`.

**Previous next (completed above):** migrate input-driver readiness and the source-attenuation checks in
`desktop/session/admission/input.rs` and `platform/services/input/matrix.rs`.
The source pair currently receives copied denial probes before being moved to the
driver, which opens shared transport. Keep one real transport owner per lifetime;
do not repeat the old endpoint-only handoff after upgrading it. Then inventory
explicit copied labs and retire those consumers deliberately. The on-demand
broker outcome remains separate accepted work. No latency improvement claimed.

Implementation update 2026-10-25: **boot desktop-to-root control uses one shared owner**, over `e6318026`.

The ordinary F5 bootstrap opens shared root control and validates HEALTH before
committing the Session. `desktop/session/control.rs` owns the retained client,
request deadlines, plain-reply decoding and the idle lifetime wait. Shell/login,
recovery probes and storage regrant borrow that owner mutably; no request
reconstructs a client from aliased raw endpoints. The old request helper has left
`main.rs`. Positional desktop labs explicitly retain the copied adapter.

Root setup may park during pre-user admission: its display task is already
running and it needs no service from this boot owner. Setup plus initial HEALTH
share the original 300-tick bound. Later request deadlines and two-attempt
bootstrap policy are unchanged. A failed transport is never reused or replayed;
ordinary live-root failure keeps the existing terminal owner policy. The idle
loop has no outstanding request, so it waits on the connection's lifetime/grant
endpoint and deadline, not notification hints interpreted as unsolicited replies.
It still periodically checks user state and storage; this is not event-driven
replacement of that policy polling.

Setup takes endpoint custody on success and failure; the raw partial-admission
inventory never also owns those endpoints. An admitted Session collects its child
tasks before dropping the root client and returning reservations. Bootstrap
rollback now compares notification objects and local rights as well as the old
page/frame/channel/handle snapshot. Fault fixtures distinguish malformed setup
from bad HEALTH after successful setup. The rejected setup fixture exits on
closure without a secondary panic.

F5's initial owner now receives two writer pages, two reader pages and six
channel/notification allowances: four transferred display/input channels plus
root control and storage control. No kernel or root-subtree ceiling increase.
Storage retirement tests now require the retained root's exact baseline
(one writer, one reader, five channel/object charges, one notification object and
three notification rights); replacing storage must not discard or leak root
control.

Verification:

- Normal F5 passes independent Notes drafts, protected close, slot reuse, Colors
  and cold persistence in `desktop/20261026T014518039473Z`. The host verifies
  exactly 1,621 records; the guest trace must show exactly one ready shared root
  connection per normal boot.
- All 26 bootstrap cases pass across `bootstrap/20261026T014625829566Z` and
  `bootstrap/20261026T015033216355Z`: transient/persistent death, stall, malformed
  reply and admission refusal for display/input/root, plus two bad-HEALTH-after-
  shared-setup cases. Retried boots reach a working app; exhausted boots leave
  disk contents unchanged. The final run requires explicit zero reader/endpoint/
  notification-object/right residue as well as the guest's complete baseline
  comparison, and proves the bad-HEALTH fixture received an application request.
  The initial run exposed an input fault fixture panicking on expected owner
  closure after its malformed readiness response; that fixture now exits cleanly.
- Four recursive stock desktops pass repeated subtree reclamation, protected
  close, saved Notes/Omega Colors and cold reopen in
  `desktop/20261026T015336864313Z`.
- Provider bad-setup and bad-HEALTH recovery pass in
  `provider-startup/20261026T015517225430Z`. Live restart exhaustion/manual retry
  and deep recovery pass both boots in `provider-live/20261026T015544327670Z`,
  including exact retained-root resource baselines after each storage retirement.
- The headless `--matrix-test --release` passes; copied desktop lab features
  pass Clippy. Host `cargo test --locked`, all-feature desktop/stock/input Clippy,
  formatting, source-layout, local Markdown links and whitespace checks pass.
  Kernel and runtime implementations are unchanged; ARM64 was not rerun here.

- The copied positional `matrix_database_test.py --release --notes-recovery`
  regression passes both boots in `matrix-database/20261026T015810711801Z`,
  including physical input, drafts, recovery and exact 20-record text/history/quota
  verification. Logs for this update are `build/omega-component/root-control-*.log`.

**Previous next (display completed above):** finish the remaining ordinary bootstrap exchanges: initial display INFO,
background/labels and terminal restart rendering, input-driver readiness, and
source-attenuation checks before endpoint handoff. Start at
`desktop/session/admission/display.rs`, `labels.rs` and `input.rs`. Display/source
pairs are handed to new owners that open shared streams: changing their early
traffic requires an explicit connection-lifetime handoff, not a second client
on an already-upgraded pair. Keep copied fault-lab migration separate and retain
the ordinary F5 acceptance checks. This update does not remove copied payload
syscalls or claim a latency improvement. The on-demand broker outcome above
remains separate accepted work.

Implementation update 2026-10-25: **ordinary storage-provider supervision uses shared IPC**, over `43d167e1`.

The F5 owner now retains one yielding shared control connection per provider
incarnation. `desktop/session/persistence/provider/provider.rs` owns admission,
readiness and collection; sibling `control.rs` owns opening and request transport.
Opening progresses while the recovery UI and root services remain usable. Only a
correlated HEALTH reply completes readiness; accepting the transport alone does
not. Later watchdog probes use that same retained client. On failure, collect the
exact task and drop its control custody before replacement and explicit data
regrant. Existing startup deadlines, automatic restart limits and manual retry
policy are unchanged.

Storage's Server accepts shared control beside its two independent root clients.
F5 explicitly funds one private writer page and one reader page in the desktop
owner and raises its channel/notification allowance from four to five. Those four
existing display/input-source channels remain charged to their original sponsor
following endpoint handoff; the first integration attempt exposed that full
reservation (`desktop/20261026T010509422794Z`, diagnostic
`desktop/20261026T010825600060Z`). The provider has three notification allowances
instead of two and 67 reader pages instead of 66, preserving the previous 64-page
attachment headroom beside three stream readers. Heap sizing reserves the third
possible mailbox writer. No kernel/global limit increase.

Copied positional fault labs explicitly obtain `copied_control()` for their
unsolicited blocked/release protocol; they are not a production transport fallback.
F5's quiet fault arming uses the shared control client. Bad transport ACK and bad
HEALTH after successful setup are separate startup fixtures. A rejected setup
fixture exits on peer closure without opening the disk or introducing a secondary
panic. `storage-spawn-failed`, `storage-control-failed` and
`storage-control-retired` distinguish admission, allocation and final local
custody. Recovery tests check writer/reader/notification refunds on every attempt,
including repeated replacement and explicit retry after automatic exhaustion.

The deep provider exercise exposed a separate Split startup race: its first Notes
frame could appear before shared child control finished opening. A Deeper click
then treated a not-yet-submitted LOSS as refusal. Split now keeps an unasked set,
continues servicing setup and submits LOSS once ready, within the original survey
deadline. An expired unsent query refuses that action without poisoning later
queries as if an old reply were outstanding. Published requests retain the
existing uncertainty rule. No longer deadline or automatic destructive close.

Verification:

- Host `cargo test --locked`, userspace Clippy, desktop/storage/Split all-feature
  Clippy and the copied desktop lab feature set pass. Format and source-layout
  checks pass. No kernel or runtime implementation changes in this update;
  the preceding ARM64 evidence remains the baseline, not a fresh run here.
- Ordinary F5 and cold persistence pass in `desktop/20261026T012336865261Z`,
  including the exact 1,621-record host oracle and shared-control assertion.
- Startup crash, stall, repeated crash and bad-volume cases pass in
  `provider-startup/20261026T011251583496Z`; distinct bad-HEALTH-after-shared-setup
  and malformed-setup cases pass in `provider-startup/20261026T012310184558Z`.
  Failed attempts return owner writer/reader pages and notifications, leaving
  exactly the four pre-existing source/display channel charges.
- Live precommit, postcommit, restart exhaustion/manual retry, readiness stall and
  nested recovery pass with cold boots in `provider-live/20261026T011432186347Z`.
  That run's deep case found the Split race above; after the correction, deep
  recovery and cold persistence pass in `provider-live/20261026T012230571175Z`.
  These tests assert resource refunds and one new shared control per successful
  replacement. The rerun did not force the pending-opening branch, so it is
  recovery/regression evidence, not a deterministic race fixture.
- `desktop_test.py --deep --trace-storage` passes recursive Split admission,
  dirty-close protection, resource reclamation and cold reopen in
  `desktop/20261026T012518826338Z`.

- The copied positional `matrix_database_test.py --release --notes-recovery`
  fixture passes both boots with exact 20-record text/history/quota verification
  in `matrix-database/20261026T012622345818Z`. Its storage executable is now
  239/240 admitted load pages; preserve this headroom in follow-up work.

**Previous next (completed above):** migrate boot desktop-to-root control, starting at
`desktop/session/admission/root.rs`, the root connection in `Session`, and
`desktop/main.rs::call`. That path currently copies `Pair` freely; introduce
one retained root client owner before adapting its callers, not a new client
per call. Preserve interactive bootstrap rollback and bounded root/input/display
failure policy. Remaining explicit copied fixtures and direct
bootstrap display configuration are separate consumers. Provider supervision is
now shared; this is not a complete copied-IPC removal or a latency claim.

Implementation update 2026-10-25: **owner-control failure is isolated from live matrix services**, over `ee673f8a`.

Stock and independent Split explicitly select `Server::isolate_control_failures`.
The runtime's named `server/control.rs` owns local control retirement: rejected or
expired setup, closed peers and failed plain replies release that connection and
remove its wait interests. Independent admitted clients, upstream waits and already
accepted work continue. A closed owner link is neither task death nor permission
to close a child. Actual owner task death still tears down the kernel-owned subtree.
No kernel, quota, timeout or public wire-protocol change.

The opt-in requires a clock grant. Unselected services retain their old error
policy, including clockless copied-control reception. Deadline-only waits work
when the retired owner was the final interest. Invalid caller-owned interests and
oversized replies still report errors. Typed replies retain their existing custody
rules; a failed unexposed endpoint move remains the sender's. An explicit control
replacement starts a new incarnation, and abandoned plain replies cannot reach it.
`control_connected` means locally admitted and not retired, including setup; it is
not a shared-readiness or process-liveness test.

The guest proof starts at
`distribution/lab/channels/connection/owner_failure/owner_failure.rs`. It checks
nine failure paths with an independent shared client: copied closure, unsupported
pipeline setup, malformed first/second grants, expiry after an accepted writer,
established shared closure, a retained request across closure/replacement, and
shared/copied reply publication detecting failure while owner reads are paused.
It also checks source endpoint custody, stale replies, deadline-only waits,
invalid caller interests and exact logical/physical/notification refunds.

Ordinary desktop tests use a disposable `control-failure-probe` stock build to cut
the first launched app's established control only. Production F5 does not enable
it. `python tools/boot-harness-rs/control_failure_test.py --mode split` or
`--mode matrix` proves descendant editing and durable saves after the cut, and
refusal of an owner close whose loss state is unknown. Split additionally closes
itself through its independent interaction connection. The new trace
`matrix-close-unavailable` identifies the live-child/failed-link refusal before a
LOSS can be sent.

Verification:

- Split fault mode passes fresh and cold (`desktop/20261026T005526899510Z`):
  owner close refuses, Notes edits and saves after the cut, and Split's own
  clean close collects the subtree through its independent connection.
- Nested stock fault mode passes fresh and cold
  (`desktop/20261026T005603344150Z`): its user matrix and Notes/Colors stay
  usable, saved text reopens, and input still works after the outer owner
  refuses an uncertain close.
- Normal untraced F5 passes three Notes, protected close, slot reuse, Colors
  and exact cold restoration (`desktop/20261026T005635380963Z`); its independent
  host oracle verifies 1,621 records.
- Four recursive stock desktops pass protected close, repeated subtree
  reclamation, persistence and cold reopen (`desktop/20261026T005838134723Z`).
- Full IPC passes all nine owner-failure scenarios plus the existing default
  control-error, capacity, attachment, notification, admission and subtree
  reclamation controls. Exact budget refunds pass at every scenario boundary.
- ARM64 portability passes; it does not run the stock desktop fault workflow.
  Host tests, userspace/UEFI checks, all-feature stock/Split Clippy, source layout,
  format, changed-document local links and whitespace checks pass.
- Logs: `build/omega-component/owner-isolation-*.log`. Early fault-harness runs
  were corrected to consume the setup writer ACK, target app slot one (slot zero
  is chrome), and observe immediate failed-control close refusal separately from
  a completed LOSS refusal. These were test assumptions, not added OS allowances.

**Previous next (provider portion completed above):** migrate ordinary boot desktop control and provider supervision from
copied packets, preserving startup/recovery and actual child-outcome handling.
Start at `distribution/desktop/session/admission/admission.rs`, `root.rs`,
`session/persistence/provider/provider.rs` and `desktop/main.rs::call`. Explicitly positional
fixtures remain copied. This receiver fix does not claim complete shared service
migration or a latency improvement. No owner decision is blocking this slice.

Implementation update 2026-10-25: **stock brokered lifecycle/control uses shared IPC**, over `a0adfa79`.

The [stock control entrance](source-rs/distribution/matrix/service/children/control/control.rs)
keeps owner requests, LOSS surveys, late replies and teardown sequencing above its
[transport owner](source-rs/distribution/matrix/service/children/control/link.rs).
Brokered admissions open a shared control stream while continuing to serve child
startup/acquisition. One first request can wait unpublished during opening;
expiry drops that queued request rather than publishing it later. Published
expiry retains the exact connection request and refuses further shared probes
until its late answer drains. A stale clean answer cannot close a now-dirty app.
The explicit copied fixture adapter retains its eight-unanswered FIFO limit.
Neither failed setup nor a broken stream establishes process death or permission
to discard work; kernel outcomes still govern that decision.

Stock accepts shared owner control at every hosted level. Background and the
separate Omega UI presenter now use the common OwnerControl receiver too. The
service wait combines child setup/transfer deadlines with upstream work instead
of discarding them while storage is busy. Close replies remain held while
teardown cannot use the parent stream, with bounded grace on resumption. Child
control custody is dropped before its admission reservation is returned. Authorized
stop now cancels/waits for the exact child before removing service bindings and
waiting on upstream erasure; retained visual readers are released afterward.

Every brokered admission reserves its owner-side writer, reader and notification
before lending the child grant. Chrome/Notes/Colors declarations include control;
Background receives two notification allowances and five private / two reader
pages, and chooser reader funding is four pages (three streams plus one listing).
F5's boot picker has the same explicit extra notification allowance.

Four recursive desktops exposed the total object budget separately from the
per-task metadata limits prepared previously. Stock keeps its original 40-object
initial desktop declaration and 24-object workspace grant, growing explicitly
as children arrive. The global channel/notification pool is 256 (was 192), the
per-account ceiling 252 (was 188), F5's funded root 224 (was 180), and its user
allowance/descendant growth ceiling 208 (was 168). Kernel and host composition
checks agree. No process-count, deadline or kernel-heap increase. Exhaustion and
withdrawal host tests fill the entire larger arena across independent accounts;
these larger finite scans are not a latency improvement.

Failures found during integration:

- The first ordinary UI pass did not prove every control opened: the readiness
  gate caught missing boot-picker notification funding
  (`desktop/20261026T001434470832Z`). After fixing it, its three existing reader
  pages were all occupied by streams, so a committed user copy could not refresh
  its listing (`desktop/20261026T001806367360Z`). The fourth reader fixes this.
- Raising each matrix's starting grant over-reserved recursive capacity
  (`desktop/20261026T002022371837Z`). Keeping small starting grants still exposed
  a real 21-object shortfall beyond the outer user's old 168 allowance
  (`desktop/20261026T002314032473Z`). The explicit aggregate funding above covers
  this; the host validator was updated alongside the kernel ceilings.
- Closing the UI presenter exposed a preexisting teardown race: service bindings
  disappeared before cancellation, and upstream cleanup could park the owner
  long enough for the still-running UI to panic on its closed interaction stream
  (`desktop/20261026T003254303126Z`). Stopping the child before withdrawing those
  bindings removes the half-torn-down execution window.

Verification:

- Normal untraced F5 passes user creation, three independent Notes, protected
  close, slot reuse, Colors and exact cold restoration, including the host
  oracle's 1,619 records (`desktop/20261026T003520824784Z`). The harness now
  requires all four initial shared controls: picker, user matrix, chrome and
  Background.
- The new `python tools/boot-harness-rs/control_lifecycle_test.py` passes both
  boots (`desktop/20261026T003445126582Z`): an original clean LOSS is delayed
  beyond expiry, input edits the still-live app, a second close cannot replace
  the outstanding request, the old clean reply drains, a fresh dirty LOSS
  refuses, saving permits clean close, and the separate UI presenter launches
  and closes without a teardown panic.
- Four recursive desktops pass repeated reclamation, protected close,
  persistence and cold reopen (`desktop/20261026T003607333624Z`). Deep Split
  passes both boots (`desktop/20261026T003750366663Z`), including shared stock
  control into Split. Delayed upstream interaction/storage passes three nested
  levels and close resumption (`desktop/20261026T003836715219Z`).
- Matrix and matrix-storage guest suites pass the retained copied adapters.
  Full IPC passes connection correlation, setup/partial-custody failures,
  twenty-stream capacity and final logical/physical refunds.
- ARM64 portability passes with the enlarged global arena; it does not exercise
  the stock desktop control workflow. Host tests include full-arena exhaustion
  and withdrawal, and composition-boundary refusal checks pass. Userspace/UEFI
  checks, all-feature stock/Notes/Background/UI Clippy, format, layout and
  whitespace checks pass.
- Notes recovery passes both boots with exact text/history/quota (20 records),
  physical input, draft/reload and focus isolation
  (`matrix-database/20261026T004111561648Z`).
- Logs: `build/omega-component/stock-control-*.log`. Final reruns after the
  teardown-order fix pass; the failed integration artifacts above remain useful
  evidence of the missing funding and old ordering.



**Previous next (resolved by the update above):** isolate inbound owner-control failure in multi-client matrix
receivers. The outgoing stock controller does not infer death or replay on
expiry, and leaf OwnerControl preserves independent input after failure. However,
stock/Split use the generic Server, whose owner-control read/setup/reply failure
still propagates out of the service; actual matrix exit then tears down its tree.
This retained receiver policy is not evidence of matrix survival after control
failure. Add explicit control retirement and a guest proof that independent
upstream/downstream service remains usable, without suppressing unrelated server
errors or treating owner-link retirement as a task outcome.

Root desktop control and provider supervision still use copied packets,
as do explicitly positional fixtures. Start at
[boot admission](source-rs/distribution/desktop/session/admission/admission.rs),
[its root handoff](source-rs/distribution/desktop/session/admission/root.rs), and
[storage supervision](source-rs/distribution/desktop/session/persistence/provider/provider.rs).
Migrate that owner path with exact request custody and failure tests, then inventory remaining copied payload
consumers before retiring their syscalls. This completes brokered stock controls,
not the entire shared-service migration. The ordinary desktop remains the proving
path; there is no new owner-level blocker.

Implementation update 2026-10-25: **funded connection capacity for stock control migration**, over `1434566e`.

The stock control audit found three independent metadata ceilings. Its eleven
service streams plus incoming owner control and five child controls need seventeen
writers and about 62 notification rights including wake hints. The kernel now
bounds each task at 24 private objects (was 16), 64 notification handles (was 48)
and 48 wait interests (was 32). Page and channel/notification object funding is
still explicit and independently enforced. These larger finite tables and scans
are capacity headroom, not a latency improvement. The task-count and dynamic IPC
endpoint limits are unchanged.

Runtime wait arrays retain single-page alignment for the enlarged ABI. Server
client dispatch remains 31 clients plus control because its pause mask is u32;
extra wait interests do not silently introduce unsupported client bits. The
reservation probe accepts sixteen shared notification slots beside 32 endpoint
slots, then refuses the next before taking custody.

The [capacity guest](source-rs/distribution/lab/channels/connection/capacity.rs)
keeps twenty shared streams live with sixty notification rights on each peer.
It fills spare metadata, proves failed creation leaves the budget unchanged,
continues exchanging messages, exercises readiness in slot 47 and rejects an
invalid final handle despite a ready first entry. Both wait entrypoints are
covered; failed validation does not consume readiness. Final stream retirement
returns exact enclosing logical and physical resources. Close authority remains
retained by each production stream.

The channels fixture explicitly funds 64 private pages, 64 reader pages and 80
channel/notification objects. The first expanded run exposed insufficient fixture
page funding in the private-object exhaustion probe; increasing that explicit
fixture grant fixed it. Construction moves peer endpoint pairs immediately to
avoid exhausting unrelated endpoint metadata. The copied-channel quota test
retains only wait custody for extra notification objects so it reaches object
quota with handle metadata still available.

Verification:

- Full x86 IPC suite passes, including twenty streams, invalid trailing authority,
  metadata refusal, continued exchange and final refunds.
- Normal untraced F5 passes creation, independent apps, protected close, slot reuse
  and cold restoration (`desktop/20261026T000405405635Z`).
- Four nested desktops pass protected close, repeated subtree reclamation,
  persistence and cold reopening (`desktop/20261026T000509966287Z`).
- ARM64 portability passes with a new SVC proof of slot 47 readiness and invalid
  final authority; the harness requires its marker. It does not run the x86
  twenty-stream fixture.
- Host tests, userspace checks, all-feature channels/stock/portability Clippy,
  source layout, formatting and whitespace checks pass.
- Logs: `build/omega-component/control-capacity-*.log`. The initial underfunded
  guest failed as described above; the subsequent complete suite passes.

**Next:** stock lifecycle/control is still copied. Integrate yielding control
setup and queue the first owner request while setup is incomplete; preserve
LOSS/HEALTH correlation, late replies, storage pauses and uncertainty without
inferring process death. Explicitly fund each receiver and migrate Background
and the separate UI receiver. The ceilings above make room for that work; they
do not themselves migrate the desktop or prove its eventual full composition.
Root provider supervision and copied fixtures also remain.

Implementation update 2026-10-25: **Split lifecycle links use shared control**, over
`d4a780a0`.

The installed Split now opens separate shared control links to its two Notes
children and to its own nested copy. Its
[control owner](source-rs/distribution/applications/split/session/children/control.rs)
advances setup while the ordinary loop serves child acquisition, input and storage.
LOSS requests use the connection's exact request identity. Timeout keeps loss
unknown until that child exits; neither timeout nor failed setup kills a child,
replays the question, or treats a late answer as a new observation. The stock
matrix's link into the outer Split remains copied pending stock migration.

[OwnerControl](source-rs/platform/libraries/user-runtime/server/owner.rs) opens
only the process's owner port, without discovering service clients. Both the
Omega host and Rust Notes fault fixtures use it. It retains one authenticated
request until the app answers, rejects a second receive while pending and rejects
unsolicited/double replies. Invalid attachment custody is released and refused.
Closed or malformed owner links release their local resources while independent
input remains usable. The guest opening proof exercises the real owner port,
upstream progress, pending/double-reply refusal, closure, malformed setup and a
subsequent independent wake, with exact enclosing resource refunds.

Funding is explicit: Split declares 43 channel/notification allowances (was 38),
each Notes receives 42 private pages, seven reader pages and three notification
allowances, and nested Split receives 122/74 pages and 25 channel/notification
allowances. Shared controls retain the same separate authority as the old owner
port. No kernel ABI, wire version or metadata ceiling increases. Control clients
are dropped before admission cleanup; only BAD_HANDLE for an already-closed,
never-reused endpoint name is harmless.

The untraced desktop exposed a separate installation-headroom issue: the failed
image had 2,095,660 retained bytes against the lab's 2,097,152-byte root allowance,
leaving only 1,492 bytes for a new user's hierarchy. NEW USER correctly refused
instead of partially publishing. The traced build fit, so a traced pass alone was
insufficient evidence. The lab root allowance is now 4 MiB; physical volume capacity
remains independently enforced. This also doubles the collection scan ceiling and
moves its half-quota trigger; it is not a production storage policy or a bounded
latency improvement. `inspect-hierarchy` now prints charged bytes and the limit.
Failed artifacts are `desktop/20261025T234422029760Z` and
`desktop/20261025T234736697008Z`; the quota inspection is recorded in
`build/omega-component/split-control-quota.log`.

Verification:

- Untraced normal F5 passes user creation, three Notes instances, protected close,
  slot reuse, Colors and exact cold restoration (`desktop/20261025T234910734214Z`).
- Untraced deep Split passes two Notes, dirty-close protection, resize, repeated
  reclamation, same-code nesting, bounded deeper refusal and cold restoration
  (`desktop/20261025T235005742718Z`). The harness requires shared-control readiness
  for all six initial leaf admissions and all sixteen leaf/nested admissions
  over its complete deep workflow, on each boot.
- Masked capture passes hidden editing, sibling input, recipient byte inspection,
  restore and cold reopen with the Rust Notes fixture
  (`desktop/20261025T235050664466Z`).
- Full IPC suite passes the owner-receiver and connection cases. Notes recovery
  passes both boots, exact text/history/quota and capacity refusal
  (`matrix-database/20261025T235251467261Z`); provider size remains 239/240 pages.
- Host tests, sixteen host volume tests, userspace checks, all-feature
  Split/Omega-host/Notes/channels Clippy, layout, format and whitespace checks pass.
  ARM64 portability passes; it does not run the Split or owner-receiver fixture.
- Logs: `build/omega-component/split-control-*.log`. The two pre-fix untraced
  runs above failed user creation; subsequent untraced runs pass.

**Next:** stock lifecycle/control still needs its opening/ready/retired owner
states, explicit backing/notification funding, and migration of Background and
other control receivers. Preserve stock's late LOSS/HEALTH handling and distinguish
an unhealthy link from process death. Its fully populated user matrix already
uses 44/48 notification handles; Split's successful bounded composition does not
prove that stock's extra streams fit. Root provider supervision and explicitly
copied fixtures also remain. The broader shared-service migration is open.

Implementation update 2026-10-25: **client connection setup yields to upstream
work**, over `067750c8`.

[Connecting](source-rs/platform/libraries/user-runtime/connection/opening.rs)
owns partial setup and advances one incoming handoff per poll. Its readiness
interest and absolute deadline can join the caller's existing event loop.
Completion transfers the connection once; errors immediately release local
custody, even if the caller retains the failed connector. Published backing keeps
its peer/queued-grant lifetime: abandoning setup cannot recall exposed memory.
The ordinary synchronous `connect` now drives this same implementation. No wire
version, kernel ABI, authority rule or resource ceiling changes.

The [opening guest](source-rs/distribution/lab/channels/connection/opening.rs)
exercises the actual owner control port: a child requests an upstream service
before accepting control setup, then requests upstream work during eight control
calls. The parent keeps answering while opening and polling the control link.
This would deadlock with blocking setup in that parent loop. Fault cases cover
every partial phase with abandonment, malformed replies, expiry and peer closure,
plus a wait right offered in place of signal authority. Accepted and still-queued
backing survive local abandonment; all enclosing logical/physical/notification
budgets return exactly after final release.

Verification:

- Full IPC guest suite passes, including the new opening/custody cases.
- Normal F5 passes independent apps, protected close/slot reuse, persistence and
  cold reboot (`desktop/20261025T233319449334Z`).
- Three nested desktops with delayed interaction pass upstream progress, repeated
  subtree reclamation, protected close, persistence and cold reopen
  (`desktop/20261025T233414338101Z`).
- Notes recovery passes both boots with exact text/history/quota
  (`matrix-database/20261025T233614827602Z`); the recovery provider remains
  239/240 load pages.
- ARM64 portability suite passes; it does not run the new x86 control-opening
  fixture. Host tests, userspace checks, all-feature channels/stock-matrix Clippy,
  layout, formatting, whitespace and changed-document local links pass.
- Logs: `build/omega-component/opening-*.log`. No failing guest run in this slice.

**Next:** ordinary lifecycle/control messages still use copied queues. Integrate
opening/ready/retired states into stock and independent Split owners and their
leaf control receivers. Preserve exact request correlation, delayed LOSS/HEALTH
answers, cancellation and actual process outcomes; a slow reply is not a dead
process. Fund each end's writer/reader page and notification metadata explicitly:
the populated stock user matrix already uses 44 of 48 notification handles, so
enabling every extra stream without a capacity plan would fail. Keep control
authority separate from the supplied service provider. This slice supplies and
proves the setup prerequisite; it does not complete that lifecycle migration.

Implementation update 2026-10-25: **installed stock matrices acquire their startup
services**, over `54254685`.

The ordinary user/workspace matrix now starts with the same scoped broker shape
as its apps. [Parent initialization](source-rs/distribution/matrix/service/parent/parent.rs)
selects `ServiceBindings<2>`, acquires its notification wait right and storage pair,
then moves the existing broker pipeline into its input/presentation dispatcher.
No second handshake or request-ID reset occurs. The default `Services` alias keeps
ordinary apps at width one; callers explicitly select a larger negotiated window.
`into_shared_transport` consumes the acquisition owner, preserving its transport
and leaving previously returned service rights with the caller.

The normal stock image declares `CTHIMP02` broker startup without changing its
owner-selected resource share. The `desktop-host` image declares `CTHIMP05`: the
same required resource profile as before, now explicitly brokered. The retired
positional `CTHIMP04` profile is rejected; there is no legacy image conversion or
silent reinterpretation. All installed matrices rebuild with the selected entry
protocol. Host import tests check the new selection, unchanged resource/service
ceilings, truncated profiles, and refusal of the old profile.

Each matrix's input hint is now independent of RPC reply hints, with only wait
authority in the child. A fully populated user matrix adds one local wait right
to the prior 43-handle count (44 of 48); object/page allowances and kernel limits
are unchanged. Root hardware/display adapters keep their explicit boot wiring.
Three/five-endpoint copied-wake bundles remain only for explicitly wired fixtures,
not the installed user or Nested programs.

Verification:

- Default F5 passes independent Notes, switching, protected close/slot reuse,
  Colors persistence and cold reboot (`desktop/20261025T231805476392Z`).
- Four nested desktops pass repeated teardown, protected close, saved state and
  cold reopen (`desktop/20261025T231959803234Z`). The desktop harness requires the
  matrix broker/wake marker; the nested harness requires one for each workspace
  and user matrix, not just the outer user.
- The three-level delayed-interaction fixture passes on fresh and cold boots:
  input progresses while presentation is held, late replies retain their owners,
  and recursive close/state restoration remain usable
  (`desktop/20261025T232136121656Z`).
- `--matrix-test` and `--matrix-storage-test` pass their explicit positional
  compositions, recursive attenuation, revocation, replacement and stored-program
  launch checks. Notes recovery passes both boots with exact text/history/quota
  (`matrix-database/20261025T232413795127Z`). The recovery provider remains 239/240
  load pages; the ordinary stock matrix is 88/240.
- Host tests, userspace checks, stock matrix all-feature Clippy, source layout,
  formatting, whitespace and local links in changed documents pass. No failed
  guest run in this slice. This is x86-64 desktop service evidence; the ARM64
  portability suite was not rerun for this platform-only change.
- Logs: `build/omega-component/matrix-broker-*.log`.

**Next:** lifecycle/control messages remain copied. Their shared setup must yield
while newly admitted children acquire upstream services; blocking the parent in
that handshake would deadlock. Preserve separate control authority, correlated
LOSS/HEALTH replies, pending custody and cancellation through replacement/death.
Account for control connections' backing and notification metadata before enabling
all child slots. Explicit copied fixtures also remain to migrate. The broader
shared-service outcome is still open; no latency improvement is claimed here.

Implementation update 2026-10-25: **brokered input wakes use notifications**, over
`b8a943ca`.

The [admission coordinator](source-rs/platform/libraries/user-runtime/matrix/admission/admission.rs)
replaces each brokered child's wake queue with one kernel notification. The matrix
keeps signal/control rights; [Offer](source-rs/platform/libraries/user-runtime/services/offer.rs)
retains a wait right until acquisition and releases its source copy after receiver
custody commits. The child receives only wait authority. Failed creation/spawn,
interrupted acquisition and child collection include notification cleanup.
Bootstrap version 2 distinguishes this typed handoff from version 1's endpoint
reply; individual service versions remain 1. Old bootstrap requests are refused,
not silently interpreted as notification requests.

[Wake](source-rs/platform/libraries/user-runtime/services/wake.rs) exposes mixed
readiness and consume-before-poll behavior. The notification is independent of the
shared connection's reply bell, so unrelated RPC completion cannot consume an input
hint. Omega hosts, Background, Split, the separate UI presenter and Rust probe
loops use it; existing control-first dispatch remains control-first. Coalescing
applies only to readiness, not to the input event queue. No new kernel ABI or
Omega application operation is required.

Object allowances do not grow: one notification replaces one queue. Notification
metadata is separately bounded at 48 local handles. A fully acquired stock matrix
with eleven shared connections and five brokered children uses 33 connection
handles plus ten signal/control handles (43); unacquired wait rights are temporary
additional custody. A fully populated Split has thirty connection handles, eight
child signal/control handles and its own wait right (39). This does not reserve
unlimited room for future control connections; that migration must account for its
own metadata and backings.

Verification:

- Independent matrix guest passes repeat acquisition, interrupted handoffs,
  replacement, subtree retirement and exact resource refunds. Peers assert denied
  signal/close calls and exactly one wait handle despite cached acquisition. New
  metadata-exhaustion coverage leaves object funding spare, fails wake creation
  after channel allocation sixteen times, and verifies exact object/handle refunds
  before continuing admissions (`matrix-composition/20261025T230612852253Z`).
- Normal F5 passes three independent Notes, capacity refusal, close/slot reuse,
  saved Colors and cold reboot (`desktop/20261025T230615695014Z`). Its harness now
  requires the ordinary Omega host's notification-wake marker. Host tests,
  userspace checks and all-feature Clippy for
  changed app/proof packages pass. ARM64's existing portability suite passes;
  it exercises the generic notification transport, not this desktop service path.
- Four nested desktops pass repeated teardown, protected close, confined persistence
  and cold reopen (`desktop/20261025T230721235986Z`). Independent participants pass
  input/cursor routing, resize revocation and repeated grants on both boots
  (`desktop/20261025T230951707705Z`). Masked capture passes recipient byte checks,
  hidden input, restoration and cold reopen (`desktop/20261025T231055177851Z`).
- Notes recovery passes both boots with exact saved text/history/quota
  (`matrix-database/20261025T231206411751Z`); its provider remains 239/240 load pages.
  The Omega UI guest passes controls, editing, close/reopen and cold boot
  (`omega-ui/20261025T231252818214Z`). The independent Omega matrix's upstream proof
  passes nested attenuation, revocation and subtree reclamation
  (`omega-matrix-upstream/20261025T231427471109Z`).
- Layout, formatting, whitespace and local links in changed documents pass.
  Logs: `build/omega-component/wake-*.log`. No failed guest run in this slice.

**Next:** positional stock-matrix wakes still use the explicit copied adapter;
lifecycle/control links and copied fixtures also remain. Migrate these without
adding a permanent alternative transport. Stock interaction already owns a
two-call pipeline: broker startup must preserve its request sequence and negotiated
window rather than constructing a new client on the same pair after acquisition.
Control setup must yield while the child
acquires/calls upstream services, not block inside admission. Preserve correlated
LOSS/HEALTH replies, cancellation and exact custody across early exit. The overall
shared-service outcome remains open; this slice is correctness evidence, not a
desktop latency improvement claim.

Implementation update 2026-10-25: **correlated notification-right replies**, over
`cbbacfa8`.

The next input-wake migration needs a separate wait right, not reuse of the
connection's reply notification: consuming a reply hint must not consume the
app's only pending input hint. The [shared connection](source-rs/platform/libraries/user-runtime/connection/connection.rs)
now supports that handoff. Its experimental v4 frame/setup names add a notification
attachment; all callers rebuild together, without old-format negotiation. Kernel
grant mediation still copies only the eight-byte request ID. The new generic
`NOTIFY_TRY_RECEIVE` (87) uses existing checked notification installation without
parking, preserving incompatible messages and validating output before acceptance.

`Client::call_retained_notification` returns an owning NotificationGrant guard;
unclaimed/rejected grants release their local right. Plain, memory and endpoint
calls reject notification replies, and Server rejects notification requests.
`Server::reply_notification` delegates the supplied authority while retaining the
provider's source right. Err means no delegation published; Ok after exposed
failure means receiver escrow committed, not successful app delivery. No replay
or borrowed-grant queue is introduced. Endpoint and notification replies share
incarnation/pending-ID validation but retain their different custody semantics.

Verification:

- The full IPC guest exercises usable wait-only replies, a distinct hint surviving
  unrelated reply traffic, coalescing, repeated unclaimed guard cleanup, wrong
  reply API kinds, wrong IDs/kinds, orphan-half expiry, frame-first arrival,
  stale/duplicate replies and receiver notification-metadata exhaustion followed
  by successful reuse. Final logical, physical and notification-handle refunds
  are exact. The failure-after-publication test keeps the accepted grant usable
  after transport retirement while the provider still owns its source right.
- Nonblocking typed receive checks empty/wrong-authority calls, invalid output and
  reserved fields, and incompatible queued grants without consumption. ARM64
  executes the syscall through SVC, uses the accepted wait-only right and checks
  refunds. Its exact message-count assertion now includes the new introduction;
  the runner requires both endpoint and notification custody markers.
- Normal F5 passes three Notes, slot reuse and cold saved-state restoration
  (`desktop/20261025T225146749198Z`). Notes recovery passes both boots with drafts
  and post-commit reconciliation. Its provider remains 239/240 load pages.
- Host tests, userspace checks, all-feature channel-lab Clippy, source layout and
  formatting pass. Logs: `build/omega-component/notification-grants-*.log`.

**Next:** use this handoff in Services/Offer and brokered child admission, replacing
copied wake queues with an owner-held signal/control and child-held wait right.
Migrate Omega host, Background, Split, UI and Rust probe waits together, preserving
consume-before-poll and control fairness. Review notification-handle capacity and
refunds even where an old queue's object allowance can fund its replacement.
Positional stock-matrix startup and explicit copied fixtures also need migration;
ordinary desktop wakes/control still use copied channels at this checkpoint.
Do not report the overall shared-service outcome complete. Lifecycle control
migration remains after these independent hints; its lazy setup must yield while
children make upstream service calls, rather than deadlocking inside admission.

Implementation update 2026-10-25: **shared input-producer and masked-capture connections**,
over `8d67be75`.

The installed [AUTO participant](source-rs/distribution/applications/input-demo/participant.rs)
and [capture viewer](source-rs/distribution/applications/capture-demo/preview.rs)
now select shared Services before acquiring their separate service connections.
Each keeps one broker owner and one acquired-service Client. The producer's
storage-acquisition attempt is denied on its production endpoint, then valid input
continues on that same connection. Capture retains correlated immutable reply
custody and forwards the sealed result for presentation; its interaction endpoint
still cannot capture, and the capture endpoint cannot acquire storage.

[Split's entrance](source-rs/distribution/applications/split/session/session.rs)
selects all eight child slots before admission. Its producer gets two writer and
reader pages plus two notifications in addition to its previous private grant;
the capture viewer gets two mailbox writers/readers and notifications beside its
32-page capture reader grant. Split declares 38 channel/notification allowances:
18 for two Notes, two upstream notifications, and nine per auxiliary child. With
both auxiliaries it retains ten connections/thirty notification handles, below
the existing 48-handle ceiling. The stock owner's child-matrix ceiling is now
168 private/144 reader pages, retaining masked output and mailbox backing beside
child loans. Ordinary leaves and the bounded nested Split loan are unchanged.
No kernel capacity or Omega ABI/source changes were needed.

The first capture run exposed a real routing bug: dispatch replaced the broker's
transport slot with the viewer's world slot, then replied using that changed slot.
Copied sends tolerated this; shared correlation did not. Dispatch now restores
the broker slot before either plain or image replies. Shared Server removal also
retires producer/capture endpoint names before admission cleanup; cleanup tolerates
only BAD_HANDLE for these exact, never-reused names, as for interaction/storage.

Verification:

- Ordinary participant routing, repeated stop/start, resize withdrawal and cold
  saved-state restoration pass. Producer hold, disconnect and pressure variants
  pass actual receiver cancellation, unaffected input, fresh identities, repeated
  grants and cold reopen. Fault fixtures require the shared/denied-storage marker.
- Masked capture passes byte inspection, zeroed padding, single delivery, hidden
  source behavior and simultaneous AUTO input. Ordinary delivery, source death,
  recipient death and expiry each pass three grants, sibling survival, subtree
  reuse and cold reopen. Capture fixtures require shared-connection markers.
- Same-code nested Split passes dirty-close propagation, capacity refusal,
  repeated collection and cold persistence. Normal F5 passes three simultaneous
  Notes, slot reuse and exact saved-state restoration.
- Host policy tests, userspace/kernel checks, changed-app all-feature Clippy,
  ARM64 portability, source layout and formatting pass.

Logs: `build/omega-component/producer-capture-*.log`; each guest log names its
`build/boot-harness-rs/desktop/` artifact directory. These are bounded single-CPU
correctness/lifecycle proofs, not a latency or throughput result.

**Next:** replace the ordinary copied lifecycle/control links and input wake
hints, then migrate explicitly copied fixtures and retire unused copied-payload
consumers. Preserve loss/health correlation, cancellation and authority across
notification coalescing; retain both endpoint funding and pending reply custody.
The stock producer-acquisition fault fixture still deliberately uses copied
traffic; the installed Split participant and capture viewer no longer do.

Implementation update 2026-10-25: **shared Background and Omega UI presentation**,
over `67b45f7e`.

[Background](source-rs/distribution/applications/background/README.md) and the
[separate UI demo](source-rs/distribution/applications/omega-ui/session.rs) now
select the existing shared Services owner before acquisition. Background performs
its denied optional-storage request after the first interaction poll, then keeps
drawing; a refusal does not reset IDs or poison the stream. Its owner supplies
one notification and reader page alongside its existing private grant.

The [UI presenter](source-rs/distribution/libraries/omega-ui/presentation.rs) holds
surface state for one view epoch and borrows that same Services owner for every
present/retire operation. It no longer retains raw endpoints or implements a
second reply reader. Conditional surface removal, immutable image custody,
begin/commit/abort and the authored Omega UI library stay unchanged. Both apps
request one notification in their manifests. This is Rust boundary work; it
introduces no new Omega app behavior or global service lookup.

With upstream connections, chrome, Background and three ordinary app pairs, a user
matrix retains eleven connections and thirty-three notification rights. The
per-task notification metadata limit is forty-eight, independent of page/object
funding and the unchanged thirty-two-interest wait limit. The IPC fixture funds
32 objects so its six-stream test can exhaust notification metadata with spare
object funding. That exposed an old quota test assuming every funded object fit
as two endpoint handles. It now charges excess funding with notifications, fills
channel quota while endpoint slots remain free, and checks exact refunds. This
preserves the distinction between object funding and handle exhaustion.

Verification:

- F5 with Background and three simultaneous Notes passes slot reuse and exact
  cold persistence (`desktop/20261025T222116177415Z`). The harness requires the
  shared Background/denied-storage marker after entering a user.
- UI widgets, animated switch, conditional image removal, text editing,
  clear/reset, close/reopen and cold boot pass over the shared presenter
  (`omega-ui/20261025T222248143107Z`).
- Three nested desktops pass held input/images/presentation and cold restoration
  (`desktop/20261025T222611794446Z`). Four nested desktops also pass isolated
  saves, protected recursive close, repeated collection and cold reopening
  (`desktop/20261025T222827606536Z`).
- The complete IPC guest passes both object-quota refusal with endpoint slots
  spare and notification-metadata exhaustion with object funding spare, alongside
  the existing shared custody, stale reply and final-refund cases.
- ARM64 portability, host tests, userspace/kernel and changed-package all-feature
  Clippy, layout, formatting, whitespace and changed Markdown links pass.

Logs: `build/omega-component/consumers-{desktop,ui,ipc,nested,four,arm64,host,
check-final,kernel,features,layout,format}.log`. These are single-CPU correctness
and lifecycle proofs; no latency improvement or production capacity is claimed.

**Next:** migrate input-producer/capture broker and data connections, preserving
source authorization, immutable masked output, interruption and cleanup. Then
move control/wake streams and explicit copied fixtures. Those changes must fund
both ends before lending child budgets; shared transport is not complete yet.


Implementation update 2026-10-25: **shared app broker and interaction**, over
`c28fb499` (including the incoming deletion/erasure documentation).

Omega Notes, Colors and chooser hosts select shared transport before entering the
app. One [Services owner](source-rs/platform/libraries/user-runtime/services/services.rs)
keeps acquisition and interaction on the same connection; its
[transport adapter](source-rs/platform/libraries/user-runtime/services/transport.rs)
retains IDs across moved-endpoint acquisitions and immutable drawing requests.
Optional service refusals leave the broker usable; ambiguous transport failures
retire it without replay or copied fallback. Successful cached acquisitions do
not rebuild the client. The Omega app ABI and authored behavior are unchanged.

Independent Split owns the same adapter in its output coordinator and selects
shared interaction as well as storage for its children. Notes and a recursively
admitted Split use those slots. Ordinary input wake hints, lifecycle control,
producer/capture streams, Background, the separate Omega UI demo and explicitly
copied fixtures remain to migrate. Leaf/Split interaction is still synchronous;
this does not make a leaf consume input while waiting for its drawing reply.
The stock matrix's independent input/presentation pipeline remains in place.

Funding is explicit at both ends. Ordinary Omega apps declare two notifications;
Split declares thirty allowances and retains its own transport before lending to
children. The nested Split gets 117 private/69 reader pages and twenty channel
allowances. Its owner needs up to 152 private/136 reader pages to retain images
beside that loan. Shared Server removal may already close the exact child data
handles; subsequent admission cleanup tolerates only BAD_HANDLE for those names.

Actual failures caught and corrected:

- The third ordinary Notes exhausted the matrix's eight private mapping slots,
  despite available page funding. The metadata cap is now sixteen. The compile-time
  aperture assertion caught the resulting overlap, so the framebuffer aperture
  moved to `0x0000_0080_0800_0000`. Page funding/permissions remain enforced; this is
  a bounded metadata capacity change, not a grant of RAM or framebuffer authority.
- Split's preflight must inspect the remaining channel limit after lending four
  notification allowances to its two Notes. Its nested loan also needs extra
  reader funding while the owner retains images. Missing that owner headroom
  admitted both inner Notes, then failed an immutable image handoff.
- The small Notes recovery profile must match its root admission's increased
  channel and reader grants; the initial run rejected root admission before use.
- The undeclared-storage fixture had overwritten the packed manifest header,
  clearing notification funding alongside the import count. It now changes only
  the count bits, preserving the executable's declared resource request.

Shared retained calls now report delayed/resumed transport IDs without expiring
application work. Incomplete grant/frame custody still has its separate deadline.
A setup refusal records its channel and status; interaction failure retains the
operation diagnostic. These diagnostics do not authorize replay.

Verification:

- Normal F5 fresh/cold passes three Notes, independent drafts, slot reuse and the
  exact persistent-state oracle (`desktop/20261025T215901844549Z`).
- Independent Split passes recursive same-code nesting, resize, protected close,
  repeated reclamation, bounded refusal and cold reopen
  (`desktop/20261025T220529093841Z`).
- Notes lost-reply recovery passes exact text/history/quota and cold restoration
  (`matrix-database/20261025T220709436628Z`). The instrumented provider remains
  at 239/240 load pages.
- Three nested desktops pass held polls/images/presentation, fresh input before
  the held presentation completes, recursive close, isolated saves, repeated
  collection and cold reopen (`desktop/20261025T220748599963Z`).
- Undeclared/incompatible optional storage both leave Notes editable with dirty
  close protection; Colors retains persistence, including cold reopen
  (`desktop/20261025T221041696596Z`, `desktop/20261025T221102160620Z`).
- Independent producer/regrant/resize and masked capture/hidden rendering pass
  fresh/cold (`desktop/20261025T221121985989Z`, `desktop/20261025T221202983624Z`).
- ARM64 portability, host policy tests, userspace/kernel and all-feature
  matrix/Split/Notes/channels Clippy, source layout and format checks pass.
- Cursor preferences pass inherited/producer suppression, target death and cold
  reopening (`desktop/20261025T221245015743Z`).
- Late target-stop during staged launch and failed existing-app selection pass
  continued routed input, protected drafts, reclamation and cold reopen
  (`desktop/20261025T221330901106Z`, `desktop/20261025T221406910614Z`).
- The complete x86 IPC guest passes custody, stale/partial replies, setup failure,
  metadata exhaustion and final logical/physical refunds. Its parent now funds
  24 pages: the sixteen-object test must retain spare page capacity after its
  worker loan, and asserts that distinction before the expected refusal. The
  first run still funded sixteen total and hit page exhaustion too early.
- Formatting, whitespace and changed Markdown links pass.

Logs: `build/omega-component/broker-{desktop-final,split-final,recovery,nested,
imports-undeclared,imports-incompatible,participants,capture,cursors,scene,
selection,ipc,arm64,host-final,check-final,features-final,kernel,layout,format-check}.log`.
The resource/diagnostic failures described above were corrected before the passing
runs. Guest evidence is single-CPU; shared payloads are not a measured latency gain.

**Next:** migrate the remaining ordinary copied consumers through the same owner,
then control/wake and producer/capture streams. Recalculate finite notification,
wait-set and mapping capacity before expanding the live connection set: the
ordinary user matrix already approaches its 32 notification-handle limit. Keep
asynchronous launch cancellation and input/presentation ownership intact. App
updates, object watches and public atomic batches remain separate open outcomes.


Implementation update 2026-10-25: **shared endpoint-reply custody**, over `ab24fa34`.

The next consumer migration exposed a dependency: the service broker and app
interaction use the same connection, and lazy acquisition transfers endpoint
rights. Replacing drawing calls alone would break later storage/producer/capture
acquisition. The prerequisite now exists in the shared transport; Omega app and
Split interaction have not switched yet.

The [v3 frame](source-rs/platform/contracts/connection.rs) declares its exact
attachment kind. The [incoming join](source-rs/platform/libraries/user-runtime/connection/incoming.rs)
requires matching frame/grant IDs and kinds, retains one half of each, and expires
incomplete custody. New `IPC_TRY_RECEIVE_CAP` exposes the existing kernel endpoint
validation/installation path without parking. It does not add a service registry,
change endpoint authority, or move distribution policy into the kernel.

`Client::call_retained_endpoint` returns an owned guard; unclaimed/rejected rights
close on Drop. Explicit `into_handle` transfers custody to app code. Memory/plain
client operations reject endpoints. Service request dispatch still accepts only
bytes/immutable memory; moved endpoint requests are rejected and released.
`Server::reply_endpoint` now supports shared requests. Err guarantees the right
never left its source. Once handoff publishes, even a later signal/publication
failure consumes custody; Ok means that move committed, not delivery or use.
An already accepted right is not revoked by later connection closure. Stale
requests cannot consume a fresh endpoint. The server never silently queues borrowed
endpoints. See the [transport contract/evidence](source-rs/platform/libraries/user-runtime/connection/README.md#moved-endpoint-replies).

Verification:

- The x86-64 IPC guest passes 16 repeated move/use/drop rounds, plain replies,
  stale/duplicate reply refusal preserving a fresh right, unknown grant IDs,
  wrong attachment kinds, orphan expiry, frame-first arrival, plain-client
  rejection, peer death and exact final logical/physical refunds.
- A full response mailbox refuses an endpoint before handoff; the sender then
  uses and closes that same right and queues a plain refusal. A closed notification
  target forces failure after handoff; the sender's old handle is invalid and the
  receiver can still accept and use the committed right. Both cases refund fully.
  Bad output addresses/reserved syscall fields preserve the queued grant.
- ARM64 release portability passes the nonblocking endpoint probe through SVC,
  wrong-direction denial, incompatible receive preservation, moved-right use and
  exact final accounting. Its report now includes the two extra client messages;
  the first run reached the probe but failed the obsolete 35-message expectation.
- Normal F5 fresh/cold passes three Notes, draft protection, save/close/reuse,
  Notes/Colors persistence and the exact disk oracle
  (`desktop/20261025T214445035554Z`).
- Notes lost-reply recovery passes exact text/history/quota and cold restoration
  (`matrix-database/20261025T214553575274Z`). The instrumented provider still fits
  at 239/240 image pages; no cap increase was needed.
- Three nested desktops pass held input/images/presentation, input before held
  rendering completes, protected close, isolated saves, repeated collection and
  cold reopening (`desktop/20261025T214632641396Z`).
- Host policy tests, userspace and all-feature matrix/channels Clippy, UEFI/core
  Clippy, source-layout, formatting, whitespace and changed Markdown links pass.

Logs: `build/omega-component/custody-{ipc-final,arm64-final,desktop,recovery,nested,
host,check-final,features,kernel-final,layout,format-check}.log`. The shared endpoint
protocol tests are x86-64 evidence; ARM64 exercises the new kernel/runtime syscall
and its existing portability suite, not the complete endpoint-reply protocol.

**Next:** put service acquisition and interaction on one retained shared broker
client, preserving lazy optional services and endpoint failure semantics. Then
migrate Omega app exchanges and independent Split through that owner. Funding,
ordinary F5, nested interaction, cancellation and persistent state remain required
consumer evidence; these transport cases do not substitute for it.

Implementation update 2026-10-25: **recursive stock interaction uses shared IPC**,
over `d9d91e6c`.

The normal stock matrix's separate parent interaction stream now uses
[Pipeline<2> through its connection adapter](source-rs/distribution/matrix/service/parent/interaction/connection.rs).
Its [reply entrance](source-rs/distribution/matrix/service/parent/interaction/interaction.rs)
keeps the existing tagged-input and visual-operation owners. Each submitted
transport ID belongs to exactly one of those owners; its response must also match
the expected application packet. Input can complete while presentation waits.
Startup, funding and cursor operations still wait for logical owners to retire.
Late application replies remain live; partial attachment custody keeps its bounded
deadline. Shared setup failure never retries through copied payloads.

Parent services explicitly enable pipelining on child interaction slots. Stored
admission reserves both selected server ends before lending resources. The small
Notes/Colors compositions fund both new interaction ends, including private and
reader pages and notifications. Shared removal closes exact endpoint incarnations;
child cleanup tolerates only the already-closed handle result. Root display,
recursive storage, Omega storage and Split storage retain their one-call clients.
Omega leaf interaction, Split interaction and explicitly combined-stream fixtures
remain copied. No distribution policy or service identity moved into the kernel.

The deeper test exposed an existing runtime wait-buffer defect: a larger live
interest set could straddle a user-page boundary, which the bounded kernel copy
correctly rejects with BAD_ADDRESS. Mixed waits now align their entire 512-byte
maximum record set; copied IPC waits align their 256-byte set. Compile-time size
checks prevent future capacity growth from invalidating this invariant. The guest
exercises both full-size arrays and refusal of an unauthorized final entry even
when the first entry is ready. The kernel's copy boundary is unchanged.

The delayed-poll stimulus previously sent an unrelated tag as an early reply and
then answered the same request again. Shared correlation correctly rejected it.
The delay fixture now holds the actual request without a provisional reply. Pure
packet tests retain unmatched copied-tag coverage; shared guest tests retain
unknown/duplicate/stale transport-ID rejection. This is not permission to accept
mismatched tags on a shared stream.

Verification:

- Final normal F5 fresh/cold passes the user picker, three Notes instances,
  independent drafts, capacity refusal, protected close/slot reuse, Notes/Colors
  saves and exact disk verification (`desktop/20261025T213313046014Z`).
- Three nested desktops pass fresh and cold boots, five-second held input/image/
  presentation work, fresh input at every ancestor while presentation waits,
  saved Notes/Colors, protected close and repeated subtree reclamation
  (`desktop/20261025T212742038520Z`).
- IPC guest passes the full-width wait controls and existing shared pipeline,
  replacement, custody and exact-resource-refund cases (`channels-test`).
- Small matrix desktop passes nested clipping, physical focus, subtree recovery
  and both private colors across cold reboot (`matrix-desktop-test`).
- The combined-stream matrix-storage guest passes scoped roots, recursive
  delegation, revocation, CoW, stored program admission and replacement.
- Notes lost-reply recovery passes drafts, exact text/history/quota and cold
  restoration (`matrix-database/20261025T213037774362Z`).
- Late launch-target stop and failed existing-app selection pass sibling/routed
  input, protected drafts, saving, repeated switches, subtree reclamation and
  cold reopen. See `interaction-scene.log` and `interaction-selection.log`.
- Independent participants pass input/cursor grants, same-matrix regrant, resize
  revocation, sibling survival, saved documents and cold reopen
  (`desktop/20261025T213229991426Z`).
- Host policy tests, workspace userspace Clippy, all-feature matrix/channels
  Clippy, ARM64 runtime Clippy, all 37 harness unit tests, layout and its eight
  tests pass. ARM64 is compile evidence for this slice, not a fresh guest run.

Logs: `build/omega-component/interaction-{nested-fixed,ipc,small,combined,recovery,
scene,selection,participants,desktop-final,host,check-final,features,arm64-check,
harness,layout,layout-tests,format-check}.log`. Formatting, whitespace and changed
Markdown navigation pass. The instrumented recovery provider still occupies
239/240 image pages; the harness's existing cap check guards this tight limit.

**Next:** migrate Omega leaf interaction/rendering, then independent Split and the
remaining clients/endpoint custody. Keep those changes behind the existing
Omega-facing application boundary; new Rust transport exploration does not move
app behavior back out of Omega. Recursive shared transport is demonstrated here;
no latency improvement or complete copied-syscall retirement is claimed.

Implementation update 2026-10-25: **bounded independent shared calls**, over
`8c4a4db7`.

Tracing the next migration exposed a requirement: the stock matrix keeps tagged
input polling alive while a presentation reply is delayed. Directly substituting
the one-call shared Client would serialize those requests and regress that work.
The transport prerequisite now exists; recursive interaction is not migrated yet.

[Pipeline](source-rs/platform/libraries/user-runtime/connection/client.rs) supports
one to four independently outstanding requests and returns each transport ID on
completion. Ordinary `Client` remains the one-call alias. A full local window
returns BUSY; an unconsumed mailbox returns WOULD_BLOCK before offering an object
or consuming the request ID. Neither refusal replays or retires older work.
Unknown, duplicated or retired reply IDs fail closed. Actual transport failure
retires all outstanding work; cancellation cannot undo earlier service effects.

The [server adapter](source-rs/platform/libraries/user-runtime/server/shared.rs)
requires explicit `enable_pipeline` selection; ordinary shared slots reject the
pipeline setup. It bounds app-held requests plus queued plain replies to four.
Consumption notifications let another request or queued reply progress without
waiting for an earlier application response. Ordinary one-call connections do not
add these notifications. Immutable reply objects retain caller-owned retry/refusal
semantics; the server does not silently queue borrowed object references. Slot
removal preserves the selected policy while dropping old queued replies, custody
and request IDs. No kernel syscall or application authority was added.

The instrumented recovery provider crossed the existing 240-page image cap.
Shared-runtime release builds now use size optimization, matching its existing
dev-profile choice; publication and teardown stay explicit out-of-line boundaries.
The provider fits at 239 pages. This remains tight. Both harness runners now check
compiled user images with [elf_limits.py](tools/boot-harness-rs/elf_limits.py)
before bundling. It reads the current kernel cap, rounds each PT_LOAD memory
extent separately and includes BSS. It reports the image name and required pages;
it does not replace the kernel's ELF/permission validation or raise its limits.

Verification:

- IPC guest passes 16 four-call rounds, unpublished mailbox refusal with an
  attachment, window saturation, attached and queued plain replies, exact IDs,
  input completion beside a held render-like request, peer death, and refusal by
  a service that did not select pipelining. Existing malformed grant/frame,
  incomplete custody, timeout and conservation controls also pass.
- Replacement exercises both one-call and pipeline connections. The latter is
  removed with published, queued and app-held replies; fresh setup restarts IDs
  without reviving the old reply or object-transfer authority. Final logical and
  physical budgets return to the enclosing fixture baseline.
- Final ordinary F5 fresh/cold passes three Notes instances, protected drafts,
  saved Notes/Colors and exact disk verification (`desktop/20261025T211103513135Z`).
- Final Notes lost-reply recovery passes drafts, exact text/history/quota and cold
  restoration (`matrix-database/20261025T211038448594Z`).
- Three nested desktops with delayed interaction pass protected close, isolated
  persistence and repeated subtree reclamation (`desktop/20261025T210422335233Z`).
  This deeper artifact predates the final compact runtime build; final F5 and
  recovery above cover that build.
- Host policy tests, workspace userspace Clippy and final all-feature matrix/
  channels Clippy pass. ARM64 runtime compilation and the existing portability
  guest pass (the new pipeline controls are x86-64 guest evidence). All 37 harness unit
  tests pass, including image-cap/BSS/boundary rejection; layout, its eight tests,
  formatting, whitespace and changed Markdown navigation pass.

Logs: `build/omega-component/pipeline-{ipc-final,desktop,desktop-final,
recovery-final,host,check,features-final,harness,arm64,arm64-check,format-check}.log`.

**Next:** put the matrix's asynchronous tagged-input/presentation dispatcher on
`Pipeline<2>`, preserving its separate logical reply ownership, retained late
operations, immutable-image lifetime and scene rollback. Then migrate leaf
interaction/rendering and endpoint-custody consumers. The four-call proof is a
prerequisite, not ordinary recursive shared rendering or a performance claim.

Implementation update 2026-10-25: **shared storage reaches Omega apps and Split**,
over `47801f88`.

The [Omega host](source-rs/platform/libraries/omega-app/host/host.rs) now owns its
storage connection through `Scope::connect`. App storage operations keep their
existing step ABI, scoped authority and immutable attachments. Failed exchanges
retire the connection without replay. The chooser, Notes and Colors declare one
notification allowance, and both root and nested launcher/picker admission fund
it. The first desktop check caught a missed root-picker grant; that path now
matches the nested picker. No Omega application behavior moved into Rust.

[Split's entrance](source-rs/distribution/applications/split/session/session.rs)
selects shared child storage before admission. Its storage coordinator retains
one upstream Client across setup reads, helper admission and asynchronous child
forwarding. Local scopes borrow that client; deepening changes the child roots
without restarting its request sequence. The loop waits on both connection
lifetime and notification interests, with original storage and partial-transfer
deadlines. The input producer and capture viewer use that same storage client to
load their programs, while retaining their existing interaction protocols.

Funding is explicit at each end. Split's two Notes gain a notification and reader
page each; Split reserves its own connection backings and notifications. Its
ordinary declaration is 25 channel/notification allowances, and the nested Split
receives 15. The small Notes/recovery compositions fund both ends of both leaf
connections. Their root now carries six shared streams, exceeding the old
16-notification-handle ceiling. The kernel's bounded metadata allowance is 32;
object funding and close/control rights remain separate. The six-stream guest
case fills the remaining handle slots while funding remains, checks refusal,
continues round trips and verifies final handle/resource refunds. No kernel
service identity or distribution-specific scheduling rule was added.

Verification:

- F5 fresh/cold: user picker, three Notes instances, draft protection, saved
  Notes/Colors and exact disk oracle (`desktop/20261025T204720893380Z`).
- Split fresh/cold: repeated collection, same-code nesting, isolated durable
  edits, recursive dirty-close and deeper capacity refusal
  (`desktop/20261025T204815675250Z`).
- Independent participants fresh/cold: cursor/input grants, revocation, sibling
  survival and saved documents (`desktop/20261025T204927166151Z`).
- Masked capture fresh/cold: recipient byte inspection, hidden editing, restored
  content and persisted state (`desktop/20261025T205107187271Z`).
- Three recursive desktops with delayed interaction pass protected close,
  repeated subtree reclamation, isolated persistence and cold reopen
  (`desktop/20261025T205208183246Z`).
- Undeclared optional storage stays absent while Notes keeps dirty-close behavior
  and Colors retains its own binding (`desktop/20261025T205226214866Z`).
- Notes lost-reply recovery and cold restore preserve exact text/history/quota
  (`matrix-database/20261025T204940482103Z`).
- IPC passes six-stream capacity, malformed/grant/replacement controls and exact
  reclamation. Full x86-64 smoke and ARM64 portability guests pass.
- Host policy tests, workspace userspace Clippy, all-feature stock matrix/Split/
  channels Clippy and the Notes-recovery desktop feature check pass. The desktop's
  unrelated combined `--all-features` configuration produces an existing unused
  `database_fresh` warning; real shell and Notes-probe compositions were checked
  separately. Kernel Clippy, formatting, whitespace and changed Markdown navigation
  pass. Layout and its eight tests pass (73 Omega / 151 Rust package edges).

Logs: `build/omega-component/omega-storage-{desktop,split,participants,capture,
nested,optional,recovery,ipc,smoke,arm64,host,clippy,feature-clippy,desktop-clippy,
kernel-clippy,format-check}.log`.

**Next:** migrate recursive interaction/rendering, then the remaining service
and endpoint-custody consumers. Connection setup and local Scope operations are
still synchronous; there is one outstanding request per stream. Finite per-task
handle/backing caps and single-CPU guest evidence remain limitations. This slice
makes ordinary storage use the shared transport; it does not establish a latency
improvement or complete the IPC outcome.

Implementation update 2026-10-25: **recursive shared storage and transport custody**,
over `fb0c1752`.

Stock matrices now retain one shared Client for their separate parent-storage
stream. [upstream.rs](source-rs/distribution/matrix/service/parent/storage/upstream.rs)
selects that connection or the combined copied stream used by older headless
fixtures. Local Scope calls borrow the persistent client; forwarded requests and
binding refreshes yield through notification/lifetime interests and include the
partial-attachment deadline. Scope stamps, authority and no-replay failure rules
remain unchanged. Ordinary desktop verification now requires the nested shared
storage marker after entering a user.

The generic Server permits selection of an empty local dispatch slot. Removal
releases transport custody but preserves the explicitly selected protocol for a
replacement; old request incarnations cannot deliver late replies into it. Stock
matrices preselect their child storage slots, and cleanup tolerates endpoints that
shared transport already closed. [Replacement proof](source-rs/distribution/lab/channels/connection/replacement.rs)
checks empty selection, remove/reinstall, restarted request IDs and rejected late
object delivery.

Capacity checks surfaced real composition issues in the integrated desktop. The new
nested connection needs a writer page, reader page and notification allowance
at each end; profiles and child subdivision now fund these explicitly. Dynamic
nested admission also retains the server's transport allowance before lending
capacity to the new child; otherwise a successful growth request could leave no
room to accept that child's connection. Separately,
four live connections retained all sixteen local notification handles, so starting
input as the fifth failed. Setup now releases its redundant local signal right
after delegation. Each live connection retains three handles, with no kernel cap
increase. The [five-stream proof](source-rs/distribution/lab/channels/connection/capacity.rs)
checks live round trips, bounded exhaustion and final handle/budget refunds. Its
profile funds eight endpoint queues plus five notifications; queue exhaustion is
still tested against the actual grant.

The instrumented recovery provider briefly grew to 241 image pages, above the
240-page lab admission cap. Keeping notification teardown out of line removes
duplicated failure-path code and restores 240 pages without widening that cap.
This provider remains close to the image-size ceiling; future growth should be
diagnosed at build time rather than presenting only BAD_EXECUTABLE during boot.

Verification:

- Normal F5 fresh/cold passes user copy, three Notes instances, protected close,
  saved Notes/Colors and the independent disk oracle:
  `desktop/20261025T203525652943Z`.
- Three nested desktops with delayed interaction pass fresh/cold, repeated subtree
  reclamation, protected drafts and isolated persistence:
  `desktop/20261025T202833814753Z`.
- Nested binding timeout and revocation, plus stop during an ordinary launch read,
  pass sibling input, protected drafts and cold reopen:
  `desktop/20261025T203008733039Z`, `20261025T203043445469Z`,
  `20261025T203124401114Z`.
- Small integrated desktop passes clipping, physical input, provider/subtree
  recovery and cold persistence (`matrix-desktop-test`). Scoped storage also
  passes its combined-stream compatibility, private roots, recursive attenuation,
  revocation, programs and provider replacement (`matrix-storage-test`).
- Provider recovery passes pre/postcommit faults and exact cold state:
  `matrix-database/20261025T203409398718Z`. Notes recovery passes draft retention
  and exact text/history/quota: `matrix-database/20261025T203434694657Z`.
- IPC passes replacement and five-stream capacity controls, existing malformed
  frames/grants, abandoned setup, peer death and exact logical/physical refunds.
- Host tests, workspace userspace Clippy and all-feature matrix/channels Clippy
  pass. The user runtime compiles for ARM64; no new ARM64 guest or kernel smoke
  claim is made for this slice. Layout, its eight tests, formatting, whitespace
  and changed Markdown navigation pass.

Logs: `build/omega-component/recursive-storage-{desktop-final,nested,
  binding-timeout,binding-revoke,launch-stop,small,recovery,notes,scopes,
  ipc-final,host,clippy,feature-clippy,arm64-check}.log`.
The deep/fault desktop artifacts precede the final out-of-line cleanup adjustment;
final F5, recovery, scoped storage and IPC checks cover that adjustment.

**Next:** migrate the Omega-facing storage client and independent Split host as a
coherent slice, including explicit per-app notification funding and manifests.
Omega app logic itself is unchanged here. Recursive interaction/rendering, endpoint
attachments and startup control messages remain copied. Connection setup and local
Scope calls still wait synchronously. One outstanding request per shared stream,
finite handle/object caps and single-CPU evidence remain limitations; this is not
a latency claim or completion of the shared-IPC outcome.

Implementation update 2026-10-25: **multiplexed shared connections and root storage**,
over `0b1d3d49`.

The [server](source-rs/platform/libraries/user-runtime/server/shared.rs) now borrows
one explicit `SharedSlot` per selected control/client connection. Each has separate
request correlation, attachment custody and setup/partial deadlines. Its bounded
wait includes each notification and lifetime endpoint, rotating dispatch fairly.
Registration refuses a notification reservation beyond the wait-set capacity.
Dynamic disconnect and owner-supplied link reconnection reset that slot's transport
state; old Request incarnations cannot reply into its replacement. Server drop
releases shared custody. Other services pay for references, not large stack arenas.

Both root/storage links now use that mechanism. Follow
[storage.rs](source-rs/distribution/matrix/service/parent/storage/storage.rs) for
scope decisions, [provider.rs](source-rs/distribution/matrix/service/parent/storage/provider.rs)
for shared connection/lifetime, and `pending.rs` for yielding publication and bulk
read exchanges. Local `Scope` calls borrow the persistent client, preserving its
IDs and custody across temporary wrappers. Owner regrant still controls provider
replacement; failure retires the connection and never automatically replays a write.
The storage provider selects the private root from its authenticated link slot and
keeps existing scope checks. Ordinary F5 requires the shared-storage marker as well
as visible operations and the independent storage oracle.

Profiles reserve two writer pages, reader pages and notification allowances for
storage. Review found a real composition ceiling: SOURCE, display and two storage
writers exhausted the old four-owned-object cap, leaving no scratch publication
slot. The kernel's bounded owned-object limit is eight; page funding and reader
rights stay independent. A guest exercises four writers plus scratch, metadata
exhaustion at eight, and final logical/physical refunds. The provider reserves its
two transport pages before sizing its heap, preserving F5's 160-page engine heap.
This is still bounded lab capacity, not unlimited connection scalability.

Verification:

- Normal F5 fresh/cold passes, including user copy, three Notes instances, protected
  close, saved Notes/Colors and exact disk oracle: `desktop/20261025T195516451531Z`.
- Scoped storage guest passes both roots, recursive attenuation/revocation, bulk
  programs, launched siblings and provider replacement (`matrix-storage-test`).
- Provider precommit/postcommit stalls, replacement, live siblings, ambiguous CREATE
  reconciliation and cold reopen pass: `matrix-database/20261025T195706816045Z`.
- Notes postcommit recovery preserves the draft and exact text/history/quota across
  cold reopen: `matrix-database/20261025T195732191173Z`.
- Three nested desktops with delayed interaction pass fresh/cold, including dirty
  close, saved Notes/Omega Colors and subtree reclamation:
  `desktop/20261025T195810554239Z`.
- Final IPC and full x86 smoke pass with the increased object cap. IPC includes
  simultaneous control/two-client streams, equal IDs with reversed replies,
  partial setup expiry beside a healthy peer, wait-set refusal and exact refunds.
- Host tests, workspace userspace Clippy and all-feature matrix/channels/storage
  Clippy pass; x86 smoke-feature kernel Clippy passes.
- After the capacity adjustment, F5 fresh/cold passes again:
  `desktop/20261025T200215606196Z`. Provider recovery and exact cold state pass again:
  `matrix-database/20261025T200317902257Z`. ARM64 release portability passes, including
  shared mailbox round trips, notification readiness, memory protection and rollback.
  The earlier scoped/Notes/nested artifacts above predate the capacity adjustment.
- Layout and its eight tests, format, whitespace and changed subsystem navigation
  checks pass.

Logs: `build/omega-component/shared-storage-{desktop,scopes,recovery,notes,nested,
  ipc-final,smoke,arm64,desktop-final,recovery-final,host,clippy,feature-clippy,
  kernel-clippy}.log` and
`shared-multiplex-ipc.log` in the same directory.

**Next:** migrate recursive matrix storage and the Omega-facing storage client,
including their endpoint handoff, per-peer backing and combined wait-set budgets.
Connection setup and local Scope calls still wait synchronously; yielding provider
reconnect setup is another follow-up before claiming a fully yielding service loop.
Inner interaction/rendering streams and boot control messages remain copied.
The current mailbox still carries one outstanding request per stream and at most
64 application bytes with one immutable attachment; endpoint attachments remain
unimplemented. The encompassing IPC outcome stays open.


Implementation update 2026-10-25: **yielding shared display and image connection**,
over `8c9bc356`.

The [root display adapter](source-rs/distribution/matrix/service/parent/display.rs)
now sends display commands and immutable image grants through the shared connection.
Presentation and owner scene transactions submit/poll one step at a time; the
matrix loop waits for both reply notifications and peer lifetime alongside child
and storage work. Existing synchronous image/cursor/cleanup operations use that
same connection. Nested matrix interaction and storage still use their old path.

The platform Client exposes begin/poll, pending state, readiness interests and the
partial-transfer deadline. Synchronous helpers reuse that state machine. A second
begin cannot replace an outstanding call. The generic Server can explicitly upgrade
its owner control connection, preserving its existing authority; changing control
pairs retires old shared state before a new incarnation can receive anything.
No application frame can claim control authority. Grant/frame correlation and
failure cleanup remain the transport's existing responsibilities.

The provider and small lab roots now reserve both sides of the display connection.
F5 boot health/label messages still precede endpoint handoff and shared upgrade;
there is no copied payload fallback afterward. This does not migrate every boot
message or recursive hop, eliminate all synchronous service work, or establish a
performance improvement.

Verification:

- Normal F5 fresh/cold, Notes instances, protected close, Colors and the storage
  oracle pass: `desktop/20261025T193807695088Z`. The fixture requires both the input
  and display shared-connection markers plus rendered/input behavior.
- Three nested desktops with held presentation/delayed interaction pass fresh/cold,
  including repeated subtree reclamation: `desktop/20261025T194204390047Z`.
- IPC suite passes async begin/poll and second-request rejection, control object
  replies, stale control reply rejection and control setup expiry, alongside all
  previous attachment/mapping/funding checks and exact logical/physical refunds.
- Delayed-stop app selection passes fresh/cold, including routed input, protected
  drafts and reclamation: `desktop/20261025T194430597997Z`.
- Bootstrap input death, root stall and display stall each recover:
  `bootstrap/20261025T194506239011Z`.
- The small-profile matrix desktop passes nested clipping, physical focus, subtree
  recovery and cold-reboot persistence: `build/boot-harness-rs/matrix-desktop-test`.
- Host tests, workspace userspace Clippy, all-feature channels and matrix Clippy pass.
  Source-layout checks and their eight policy tests, Rust formatting and whitespace
  checks pass.
- Full x86 release smoke passes, including copied-channel compatibility, fault
  containment, display recovery and memory reclamation. No ARM64 rerun is claimed
  for this userspace migration.

Logs: `build/omega-component/display-shared-{desktop,nested,selection,bootstrap,
  matrix,ipc,host,check,channels-clippy,matrix-clippy,smoke}.log`.

**Next:** broaden shared service multiplexing and migrate the storage-facing
connections, preserving scoped authority, immutable bulk replies, deadlines,
provider replacement and cancellation. Omega-facing adapters and inner matrix hops
remain explicit subsequent work; the encompassing shared IPC outcome stays open.

Implementation update 2026-10-25: **immutable attachments on shared requests/replies**,
over `bf83610f`.

The [connection](source-rs/platform/libraries/user-runtime/connection/README.md)
now joins one shared frame with one kernel-mediated immutable read grant. The
endpoint transfers only an 8-byte correlation ID and typed authority; application
bytes remain in the shared mailbox. Both halves must match before dispatch.
The receiver retains at most one frame and one grant, with an explicit 300-tick
incomplete-transfer deadline. This deadline does not time out admitted service work.
The server continues other dispatch while waiting for a missing half.

The provider retains its source Object. Refusal before offering the grant permits
a plain error reply on the same request; failure after offer/publication retires
the connection without replay. A stale retained reply cannot offer an object into
a replacement stream. Peer death preserves already accepted immutable backing;
partial setup/ingress and queued grants release their exact charges on retirement.
The experimental framing is v2; this is not a frozen ABI or legacy fallback.

Verification:

- Host tests and userspace checks pass. Shared IPC guest tests pass repeated
  attached request/reply forwarding, recipient quota refusal followed by a plain
  reply and later successful transfer, timeout with retained stale object reply,
  complete final reply across server death, wrong IDs/kinds and orphan expiry.
  Malformed request senders also prove other-client progress and no premature
  dispatch. The full fixture restores exact logical and physical baselines.
- Normal F5 fresh/cold, pointer/typing, independent Notes instances, protected
  close, Colors and the storage oracle pass: `desktop/20261025T192632887289Z`.
- Layout (73 Omega / 151 Rust edges), eight boundary checks, formatting and diff
  checks pass. Entrances retain the frame/grant join and custody owners explicitly.
- Logs: `build/omega-component/attachment-{host,check,channels-clippy,ipc-final,desktop}.log`.
  Kernel and ARM64 behavior did not change; their previous evidence is retained,
  not claimed as fresh attachment-path testing.

**Next:** integrate this attachment path into the ordinary root/display connection.
First expose yielding begin/poll operations and their notification/lifetime wait
interests so presentation, scene updates and other service work keep progressing.
Then migrate image forwarding and preserve delayed-presentation, cancellation,
bootstrap and nested-desktop evidence. Display and storage attachments still use
the old path today; this transport milestone does not complete their migration.

Implementation update 2026-10-25: **first shared-payload F5 service connection**,
over `c2b11a6c`.

Start at [the connection entrance and custody guide](source-rs/platform/libraries/user-runtime/connection/README.md),
then [server integration](source-rs/platform/libraries/user-runtime/server/shared.rs)
and [the input consumer](source-rs/platform/services/input/matrix.rs). Ordinary F5
physical input now sends pointer/key requests through two single-writer mailboxes
and notifications. Existing endpoints mediate typed setup and observe lifetime;
after setup they never serve as a copied-payload fallback. The owner still runs
source-authority rejection probes before handing the armed connection to input.

Kernel EVENT_WAIT now admits notification interests alongside IPC/device interests.
It validates the whole set, observes without consuming pending signals and wakes
only affected observers through the existing bounded readiness coordinator.
Notification drain remains separate. Observer cleanup precedes task-slot reuse;
no application frame or request protocol enters the kernel.

The platform client correlates one outstanding request and retires on transport
failure without replay. A retained call waits through slow service work, preserving
input's existing behavior under delayed presentations. The server advances setup
one grant/ack per dispatch, expires abandoned setup, and binds replies to the
original endpoint incarnation. A final published reply may be read after peer exit.
The profiles fund input's one writer page, reader page and notification object.
Small lab roots reserve the matching server resources beside their old child/image
allowances; the F5 hub already has capacity.

Scope is deliberately limited: one enabled shared stream per Server, 64-byte
payloads, no response attachments. Other desktop services, nested forwarding and
Omega host bindings still use the previous transport. This does not complete the
shared IPC outcome, change accepted cancellation semantics, establish SMP safety
or demonstrate a speedup.

Verification:

- Default host tests, kernel Clippy, core all-feature/test Clippy, userspace
  workspace Clippy and all-feature channels Clippy pass. The new guest connection
  proof covers 64 calls, wrong reply IDs, timeout retirement, peer death, partial
  and expired setup, independent control progress, stale replies and exact refunds.
- Normal F5 fresh/cold desktop and storage oracle pass:
  `desktop/20261025T184352430898Z`. Later desktop runs explicitly require the
  shared-source marker as well as visible input behavior.
- Three nested desktops with held presentation/delayed interaction pass fresh/cold,
  including protected close, persistence and repeated subtree reclamation:
  `desktop/20261025T185404985548Z`.
- Selection `late-stop` passes fresh/cold:
  `desktop/20261025T185720984150Z`.
- Transient input death/stall bootstrap recovery passes with exact reclamation:
  `bootstrap/20261025T185935140537Z`. The host test now derives expected process
  capacity from the profile instead of assuming four slots. Use the documented
  minimal source image `launcher-recovery/20261024T054041211303Z/clean/guest.raw`;
  the first run on the full desktop fixture correctly refused an upgrade above
  its root quota before boot. No production quota was relaxed for that test.
- Full x86 smoke and kernel-only smoke pass. The latter's expected marker boundary
  now matches `lab/lab.rs`: device/shared-object proofs require bundled-user.
- ARM64 mailbox exchanges with mixed notification/IPC readiness pass, including
  exact session frame/heap reclamation. This is not full ARM64 service migration.
- Layout (73 Omega / 151 Rust edges), eight boundary checks, formatting and diff
  checks pass. Logs live under `build/omega-component/connection-*.log`.
- The smaller matrix-desktop profile passes visible focus, clipping, subtree recovery
  and cold color persistence after explicitly funding the server half of SOURCE:
  `matrix-desktop-test` / `connection-matrix-desktop.log`. Notes and database-recovery
  profiles received matching allowances; their dedicated recovery suites were not
  rerun in this slice.

**Next:** migrate an attachment-bearing service connection, keeping typed grant
acceptance separate from payload publication; extend bounded multiplexing and
preserve storage/launch custody. Then move Omega-facing adapters onto the same
boundary. Do not bypass the existing slow-response and cancellation regressions.

Implementation update 2026-10-25: **reusable shared buffers and platform mailbox**,
over `3748fda4`.

Start at the [shared-buffer boundary](source-rs/platform/libraries/user-runtime/memory/shared/README.md),
then [mailbox.rs](source-rs/platform/libraries/user-runtime/mailbox.rs). The kernel
extends its existing funded page-object arena with irreversible mutable sharing:
one original RW/NX producer and transferable RO/NX views. A typed setup handoff
cannot be received as an immutable object; frozen mapping, legacy sealing and
executable admission reject live mutable backing. Mapping/unmapping, queued
discard, creator death and final release reuse existing lifetime/accounting.
Readers retain backing after producer death, including across sponsor replacement.
This does not implement arbitrary multi-writer grants or transfer of the writer.

The runtime consumes private ownership before sharing. It never lends a Rust
reference or slice into peer-mutable bytes: bounded aligned machine accesses copy
words, with acquire/release operations in its architecture leaf. The platform
mailbox has two independently owned buffers, one outstanding message per direction,
a sequence/acknowledgement and up to 256 payload bytes. Full returns WOULD_BLOCK;
too-small receives do not acknowledge; sequence exhaustion fails. Captured bounds
remain local even if a hostile peer changes headers. Payload semantics still need
application validation. Timeout does not acknowledge or cancel an operation.

The [ordinary-process proof](source-rs/distribution/lab/channels/shared/shared.rs)
exchanges 128 variable-sized payloads without per-message copied IPC, allocation
or remapping. It includes malformed length/sequence/acknowledgement, backpressure,
foreign handles, wrong typed receive, bad copyout, exact read-only/execute faults,
unmap/remap, retained creator funding and abandoned recipient escrow. ARM64 uses
the same mailbox for 32 full-size exchanges and retains exact session frame/heap
reclamation. Neither this nor acquire/release code is evidence for an SMP kernel.

Verification:

- Default host tests pass, including three new mutable-sharing authority/lifetime
  tests. Core all-feature/test Clippy, guest kernel, runtime and channels-lab
  Clippy pass. x86 IPC and ARM64 payload proofs pass.
- Normal F5 fresh/cold desktop and storage oracle pass:
  `desktop/20261025T172055247621Z`.
- Kernel smoke and final IPC suite, including queued-discard cleanup, pass.
- Three nested desktops with held presentation/delayed interaction pass fresh/cold,
  including protected close, persistence and repeated subtree reclamation:
  `desktop/20261025T172315156744Z`.
- Selection `late-stop` passes fresh/cold, retaining the draft and restoring the
  selected app: `desktop/20261025T172511380932Z`.
- Layout (73 Omega / 150 Rust edges), eight boundary checks, formatting and diff
  checks pass.
- Logs: `build/omega-component/shared-{host,core-clippy,kernel-check,
  userspace-clippy,channels-clippy,ipc-final,arm64,desktop,smoke,nested,selection}.log`.

**Follow-up:** the first real service connection is implemented above. Remaining
work includes response/attachment custody and broader bounded multiplexing.
Preserve launch/selection semantics and migrate Omega host bindings with their
consumers. Other F5 connections still use copied messages; copied setup/grant
handoff remains temporary. Do not call the encompassing
outcome complete or infer a latency improvement from this lab. No owner decision
was needed; this exploratory Rust direction is explicitly authorized.

Implementation update 2026-10-25: **funded kernel notifications and process proofs**,
over `2506f024`.

Start at [notifications/notifications.rs](source-rs/kernel/core/notifications/notifications.rs)
and its [boundary guide](source-rs/kernel/core/notifications/README.md). Separate
wait, signal and control grants use caller-local monotonic names. The primitive
remembers coalesced early signals, supports wake-one/all and combined signal/wait,
and fixes one completion outcome before waking through the bounded wait coordinator.
Negative combined-call results mean no signal committed; nonnegative outcomes
describe the subsequent wait. Control supports close, permanent withdrawal of all
wait rights and cancellation of current waits. None revokes application work.

Objects share the existing admitted transport quota; sixteen notification handles
per task have a separate bound. Grants, unpublished recipient escrow and pending
wait completion retain the original account's charge through death and replacement
until final release. No trap-time allocation or ambient PID wake is introduced.
The existing copied channel mediates setup grants temporarily; notification rounds
themselves carry no payload and allocate no per-request identifiers.

The platform [runtime entrance](source-rs/platform/libraries/user-runtime/notification/notification.rs)
exposes the mechanism. The [ordinary-process proof](source-rs/distribution/lab/channels/notifications/notifications.rs)
covers bad creation/send/receive pointers, wrong and foreign rights, typed escrow
preservation, coalescing, 64 combined-call round trips, two-worker wake-one/all,
cancellation, closure, withdrawal, death while parked and retained funding across
creator replacement. ARM64 runs 32 notification rounds through SVC alongside its
copied-message and exact reclamation checks. This is single-CPU evidence, not SMP.

Verification:

- Default host tests pass, including seven notification/IPC model tests and the
  bounded notification wait-wave test. Core all-feature/test Clippy, guest kernel,
  runtime and channels-lab Clippy pass. Layout, eight boundary tests, formatting
  and diff checks pass.
- IPC guest suite and smoke pass. ARM64 executes the 32 new notification rounds,
  32 existing echoes, process isolation, deadline parking and exact reclamation.
- Normal F5 fresh/cold passes independent Notes/Colors and the storage oracle:
  `desktop/20261025T162540073029Z`.
- Three nested desktops with `--interaction-delay` pass fresh/cold, including input
  through seven matrix realms during held presentation, delayed poll/image
  recovery, persistence and repeated reclamation:
  `desktop/20261025T162753135949Z`.
- Selection `late-stop` passes fresh/cold, restoring the selected app and retaining
  its draft: `desktop/20261025T162926872440Z`.
- Logs: `build/omega-component/notifications-{host-final,core-clippy,
  clippy-kernel,clippy-user,ipc-copyout,smoke,arm64-final,desktop,nested,selection}.log`.

**Next:** reusable authorized writable shared backing, followed by platform
publication/backpressure and real request/reply payloads over it. Preserve mapping
isolation and live-reader lifetime; do not expose safe Rust slices over memory an
untrusted peer may mutate. Then migrate ordinary service/Omega consumers and retire
copied payload consumers. The encompassing outcome remains open; F5 still uses its
existing service transport. No new owner decision was needed.

Implementation update 2026-10-25: **switching running apps yields through scene
commit and restoration**, over `e32d3f71`.

Start at [workspace/workspace.rs](source-rs/distribution/matrix/service/children/workspace/workspace.rs),
then [selection.rs](source-rs/distribution/matrix/service/children/workspace/selection.rs)
for retained SELECT_APP requests and completion, and `custody.rs` for exact
caller/task checks and authenticated cancellation. The same bounded parent scene
owner now stages both existing apps without changing their live local visibility,
input routes or cached content. Only successful commit installs the new selection.
The chrome request is answered afterward, through its original connection only.

Relevant owner mutations/disconnects cancel selection; caller and task liveness
are checked before commit and again at completion. Before commit, cancellation
aborts. After accepted commit, a second yielding transaction restores the previous
scene before replying or processing deferred mutations. Selection itself never
stops either app. LOSS reports BUSY through switch/restoration. Failure to restore
fails closed. An accepted scene can briefly appear before compensation.

The late-owner-stop test found a scheduling bug: a restoring scene created during
top-of-loop completion could park before its first send, delaying the owner reply
until timeout. Newly prepared scenes now schedule their first send immediately.
An earlier test failure was a harness coordinate hitting the task's close button;
the selection workload now clicks inside the task label.

Verification:

- `desktop_test.py --selection-async MODE --timeout 120` passes fresh/cold for
  `complete` (`desktop/20261025T152313117577Z`), `failure`
  (`desktop/20261025T152403241169Z`), `stop`
  (`desktop/20261025T152440070400Z`) and `late-stop`
  (`desktop/20261025T152557631006Z`). New B-key input must reach the nested matrix
  during the three-second hold and appear in the preserved draft afterward.
  Failure keeps both tasks; owner STOP reclaims its exact target; late STOP must
  restore. Repeated switches, protected close, save/cold reopen and subtree
  reclamation pass. These gates/probes are absent from normal F5.
- Normal F5 fresh/cold, three independent Notes instances, repeated switching,
  capacity refusal, protected close, Colors and the storage oracle pass:
  `desktop/20261025T152634259145Z`.
- Ordinary launch's late-owner-stop regression passes fresh/cold, retaining its
  draft and restoring after the admitted task is reclaimed:
  `desktop/20261025T152718550187Z`.
- Three nested desktops with `--interaction-delay` pass fresh/cold: input through
  seven matrix realms during a held presentation, delayed poll/image recovery,
  protected close, persistence and repeated subtree reclamation:
  `desktop/20261025T152754807755Z`.
- Default host tests, guest all-feature matrix Clippy and desktop owner-probe
  Clippy pass. Layout (73 Omega / 150 Rust edges), eight boundary checks,
  formatting and diff checks pass. Full kernel/ARM64 suites were not rerun.
- Logs: `build/omega-component/selection-{complete2,failure,stop,late-stop2,
  desktop,launch-regression,nested,host}.log`. Earlier failed experiments remain
  alongside them as described above.

**Next:** the accepted shared-memory IPC/notification outcome above is the broader
implementation frontier. Preserve the launch/selection custody and cancellation
evidence while replacing transport. Remaining synchronous paths include
lifecycle-driven fallback selection, launch compensation, admission/retirement,
root physical-source dispatch and some image/startup/recovery helpers. This does
not establish a latency bound, concurrent pixel commits or a fix for the earlier
intermittent partial-key observation. No new owner decision was needed.

Implementation update 2026-10-25: **ordinary launch activation yields and
publication is local**, over `4a191eff`. Artifact directory stamps below are UTC.

Start at [the launch boundary guide](source/distribution/matrix/service/children/launch/README.md),
then [launch/scene.rs](source-rs/distribution/matrix/service/children/launch/scene.rs)
for task/ticket custody and
[parent/scene/scene.rs](source-rs/distribution/matrix/service/parent/scene/scene.rs)
for the prepared visual transaction. Omega's existing Activate command now yields
between begin, surface updates, commit and abort. Tagged input progresses on the
shared interaction link; visual mutations remain serialized. Preparation changes
neither live visibility nor input routes. Successful commit installs the selection.
Publication only notifies children and records login state: the admitted child's
startup requests remain held through Finish, so it has no drawn surfaces to replay.

Authenticated owner changes and caller disconnects cancel pending scenes. Before
COMMIT, cancellation aborts; after submission, a successful commit must be
compensated by restoring the previous exact task before rollback. Cancellation
already known at the reply leaves local input/selection unchanged. Restoration
also handles an owner that already reclaimed the admitted task, without touching
a replacement task. Unknown transport outcomes and failed abort/restoration fail
closed. Compensation can briefly expose the accepted scene before restoration.
Kernel, public ABI and compiled Omega launch sequencing are unchanged.

Live tests found and fixed two cancellation edges: LOSS was deferred behind the
visual gate instead of promptly answering BUSY, and restoring the previous scene
was incorrectly skipped if an owner STOP had already emptied the admitted slot.

Verification:

- Six `desktop_test.py --scene-async MODE --timeout 120` cases pass fresh/cold:
  `complete` (`desktop/20261025T065153479916Z`), `revoke`
  (`desktop/20261025T065428190687Z`), requester `stop`
  (`desktop/20261025T065530817785Z`), staged `failure`
  (`desktop/20261025T065927555667Z`), `late-revoke`
  (`desktop/20261025T070107363686Z`) and `late-stop-target`
  (`desktop/20261025T070725315265Z`). Each requires fresh B-key delivery during
  the three-second hold, the preserved sibling draft, protected close, save/cold
  reopen and subtree reclamation. Canceled launches must not publish; same-slot
  retry succeeds when the launcher survives. Both late cases require an accepted
  commit and explicit restoration. This proves routed input during visual waits;
  it does not promise an app blocked in its own drawing call consumes it then.
- Normal F5 fresh/cold desktop, independent Notes, Colors and storage oracle:
  `desktop/20261025T070139489076Z`.
- Three nested desktops with `--interaction-delay`, fresh/cold: input through all
  seven matrix realms during a held presentation, delayed poll/image recovery,
  protected drafts, persistence and repeated subtree reclamation:
  `desktop/20261025T070355451990Z`.
- Default host `cargo test --locked` passes, including the new compiled-Omega
  canceled-Activate case and immutable scene projection test. Guest all-feature
  matrix Clippy, host matrix Clippy and desktop owner-probe Clippy pass. Layout
  (73 Omega / 150 Rust edges), eight boundary checks, formatting and diff checks
  pass. Full kernel/ARM64 and unrelated launch/runtime suites were not rerun.
- Logs: `build/omega-component/launch-scene-{complete,revoke2,stop,failure,
  late-final,late-stop-target,desktop,nested,final-host}.log`.

**Next within launch/scene work:** extend yielding visual transactions to selecting
already-running apps. The shared-memory IPC outcome above is the broader transport
frontier; retain this milestone's custody and cancellation evidence during migration.
That path, teardown/compensation, task admission/retirement, root physical-source
dispatch and some startup/recovery/image helpers still contain synchronous work.
The matrix is not fully nonblocking and pixel commits are not concurrent. The
earlier intermittent partial-key observation remains open; this milestone is not
a latency bound or complete input-loss fix. No owner decision blocks this work.

Implementation update 2026-10-24: **tagged input progresses beside asynchronous
presentation**, over `73cb81f5`. Artifact directory stamps below are UTC.

Start at [the transport guide](source-rs/distribution/matrix/transport/README.md),
then [the sole reply reader](source-rs/distribution/matrix/service/parent/interaction/interaction.rs).
The stock matrix permits one tagged poll beside one presentation request on its
existing scoped interaction link. Poll and drawing replies have separate custody;
unmatched tags cannot complete a presentation. Concurrent poll IDs exclude the
negative-status high word and never wrap. The public contract documents this
bounded overlap; packet shapes, startup capabilities and kernel IPC are unchanged.

Input queries remain serviceable from a publishing child. A snapshot changing
view authority or surface capacity waits for commit/abort, then is consumed before
parking. Geometry-changing work and admitted-child startup requests remain held.
This proves input delivery through matrix routers, not processing by an app that
is itself blocked in a synchronous drawing call, nor concurrent visual commits.
Root physical-source dispatch, image calls and ordinary launch activation/publication
still contain synchronous work.

The real channels exposed another requirement: one kernel message slot cannot
hold both replies. The matrix opts into the runtime server's caller-owned fixed
reply arena, retaining one additional copied reply per client and pausing that
client's request intake while full. Stale incarnations are discarded; excessive
pipelining disconnects only that client. Full request channels also retry safely.
Idle backpressure retries use one-tick deadlines until writable-channel wakes
exist. Other servers retain their existing behavior unless they opt in; memory
and endpoint transfers keep explicit custody/retry handling.

Two failures found by the live tests are fixed: the last-client disconnect now
precedes the server's control-only wait, and a close's grace extends when the matrix
resumes after being busy, rather than expiring during a long idle busy interval.
No new owner decision or kernel change was needed.

Verification:

- Normal F5 fresh/cold desktop, independent Notes, Colors and storage oracle:
  `desktop/20261025T064002435158Z`.
- Three nested desktops, fresh/cold: a five-second presentation hold requires
  fresh keyboard input in all seven distinct matrix realms before release.
  Delayed poll/image recovery, stale-tag rejection, protected close, Notes/Colors
  persistence and repeated subtree reclamation also pass:
  `desktop/20261025T063809492128Z`.
- Real owner revocation during ordinary launch binding preserves the sibling
  draft/save, prevents grant/publication, reclaims the admitted task and permits
  same-slot retry on both boots: `desktop/20261025T064128081883Z`.
- `run.py --ipc-test --release --timeout 60` passes, including the new worker
  filling real kernel channels, ordered retained replies, intake backpressure,
  stale-connection replacement and excess pipelining. Existing admission,
  sponsorship, recursive lifetime and channel reclamation checks still pass.
- Default host `cargo test --locked` passes, including 26 stock matrix tests
  (24 compiled Omega policy cases and two reply classification cases) and the new
  view/capacity compatibility test. Guest all-feature matrix/channels Clippy,
  host matrix Clippy, layout (73 Omega / 150 Rust edges), eight boundary checks,
  formatting and diff checks pass. Full kernel/ARM64 suites and the other launch
  failure modes were not rerun.
- Logs: `build/omega-component/input-present-{final-nested,desktop,revoke,
  final-ipc,final-host}.log`. Earlier experimental runs remain alongside them;
  the initial channel-full failure and idle-close failure are resolved above.

**Next:** move launch scene activation/publication to yielding host mechanisms
under the existing Omega workflow, retaining prepared geometry and atomic
commit/abort, cancellation custody, and a coherent scene/focus visibility boundary.
The transport prerequisite is now in place; those launch steps themselves are
still synchronous. The earlier intermittent partial-key observation remains open;
this milestone is not a latency-bound or complete input-loss fix.

Implementation update 2026-10-24: **ordinary launch storage binding yields to
the service loop**, over `4c884a94`. Artifact directory stamps below are UTC.

Start at [the launch boundary guide](source/distribution/matrix/service/children/launch/README.md),
then [launch/binding.rs](source-rs/distribution/matrix/service/children/launch/binding.rs)
and [storage/binding.rs](source-rs/distribution/matrix/service/parent/storage/binding.rs).
Omega's existing Bind command now starts a nonblocking upstream refresh. Rust
retains its result until queued owner mutations run, rechecks task/caller/scope
custody, and only then installs the grant and reports completion. A new private
Omega query accepts a result only while its exact ticket/command is outstanding.

Yielding exposed a startup-order obligation: requests from the admitted child must
wait until publication or rollback. The hold covers only that child's interaction
and storage requests; siblings continue input and rendering. The launch progresses
past its held requests while ready owner mutations retain priority. Stale queued
requests/attachments and replacement-connection disconnects are discarded. Loss
surveys remain BUSY for the full pending launch, including completion-to-commit.

Storage revocation is now local: cleanup no longer waits for a service to withdraw
its own authority. Future operations still validate stamped upstream bindings.
Malformed replies, channel failure and the existing 100-tick binding timeout
retire the untagged storage stream. The actual timeout code survives retirement,
rather than being obscured by the resulting authority loss. Existing apps and
input survive a separate storage failure; their live drafts are not automatically
saved and storage requires re-establishment. Legacy shared streams remain fatal.
Manual owner grants, launcher recovery and startup still use the synchronous grant
helper. Kernel, public protocol and ordinary F5 composition are unchanged.

Verification:

- Normal F5 fresh/cold desktop, independent Notes instances, Colors and storage
  oracle pass: `desktop/20261025T055833899294Z`.
- Held ordinary binding with visibly responsive Notes, protected draft, save and
  cold reopen passes: `desktop/20261025T060116729387Z`.
- Real routed owner revoke and requester stop during held binding pass fresh/cold,
  with no grant commit/publication, exact task reclamation and surviving sibling
  draft/save. Both first query LOSS and require BUSY:
  `desktop/20261025T060959598123Z` and `desktop/20261025T061039417128Z`.
- Missing reply fixture drops one real binding reply, then uses the normal
  empty-channel deadline and stream retirement. Both boots preserve the live
  draft and protected close, explicitly discard it for cleanup and reopen empty:
  `desktop/20261025T060923825529Z`. This final run checks exact TIMED_OUT reporting;
  it does not claim successful saving or automatic reconnection after retirement.
- Existing post-admission placement failure and successful same-slot retry pass:
  `desktop/20261025T060500745146Z`. Existing read revocation also passes.
- Three-level nested desktop fresh/cold, recursive dirty close, Notes/Omega Colors
  persistence and repeated subtree reclamation pass:
  `desktop/20261025T060630158753Z`.
- Existing launcher recovery (fresh/cold, explicit retries) and owner revocation
  pass with the minimal one-user source image unchanged:
  `launcher-recovery/20261025T061107001836Z` (`recover`, `owner-revoke`; other
  launcher modes and root/nested startup-fault suites were not rerun this turn).
- Default host `cargo test --locked` passes; twenty-four stock matrix tests execute
  compiled Omega, including pending Bind acceptance, stale/wrong completions and
  cancellation after task admission. Guest all-feature and host-test matrix Clippy,
  desktop probe Clippy, layout (73 Omega / 150 Rust edges), eight boundary tests,
  Rust formatting and diff checks pass. Full kernel/ARM64 and unrelated runtime
  suites were not rerun.
- Logs: `build/omega-component/binding-{desktop,complete,revoke,stop,timeout,
  timeout-final,revoke-final,stop-final,place,read-revoke,nested,recovery,host}.log`.

**Next:** activation and publication still share the upstream interaction reply
stream with input polling. `upstream_busy()` deliberately prevents polling while
presentation owns it; making only scene waits nonblocking would not deliver fresh
input. Separate scoped input/presentation transport or implement safe concurrent
reply dispatch before moving those steps, preserving atomic commit/abort and
cancellation. This is the concrete remaining transport frontier, not an owner
semantic blocker or a claim that visual launch is fully asynchronous. The earlier
intermittent partial-key observation remains open.

Implementation update 2026-10-24: **ordinary stored-program launch orchestration
executes in Omega**, over `35537b05`. Artifact directory stamps below are UTC.

Start at [the Omega launch entrance](source/distribution/matrix/service/children/launch/main.omg)
and its [boundary guide](source/distribution/matrix/service/children/launch/README.md). The F5 path
for app/user-matrix launches issues read, retire-picker, admit, bind, place,
activate, publish and finish commands through Omega. Partial failure issues task
rollback and, when needed, picker recovery. Rust dispatches those commands from
[launch/launch.rs](source-rs/distribution/matrix/service/children/launch/launch.rs),
with separate caller validation, custody and mechanism leaves. `mod.rs` remains
plumbing. Ordinary child LAUNCH and owner RUN_OBJECT use the same workflow.

The operation allocator is shared with launcher recovery and never recycles IDs.
Completions must match both ticket and command. Rust retains the exact request,
caller task/connection, parent storage identity when known, sealed image and
admitted task. It rechecks authority/liveness before consequential steps. Relevant
processed owner changes cancel pending work; queued owner commands precede
admission. Replies/receipts cannot target a replacement caller. Rollback stops only
the task this operation admitted. Retiring the login picker deliberately transfers
its accepted selection to matrix custody, preserving the existing login flow.

The read remains asynchronous, with sibling input/presentation progressing.
Binding, admission, activation and publication still use synchronous Rust host
mechanisms; explicit completions do not mean every step yields. Failed teardown
is fatal to the matrix, never acknowledged as successful reclamation. This changes
no kernel or public ABI and adds no owner semantics decision. One ordinary launch
is retained per matrix, consistent with its existing serialized storage path.

Verification:

- Full-F5 nested launch fixtures pass fresh/cold: delayed completion
  `desktop/20261025T053455332112Z`, owner revocation during read
  `desktop/20261025T053528342571Z`, requester stop during read
  `desktop/20261025T053602973451Z`, post-admission binding failure
  `desktop/20261025T053633090462Z` and placement failure
  `desktop/20261025T053716318014Z`.
- Each fixture proves sibling Notes accepts input before read release, keeps its
  draft, protects close, saves and cold-reopens. Cancellation admits no task;
  injected post-admission failures reclaim it before publication. Revocation and
  both failure cases then launch successfully in the same slot. All finally close
  the subtree. Gates/owner probes/failures are disposable-profile features only.
- Normal F5 fresh/cold desktop and storage oracle pass:
  `desktop/20261025T053755848922Z`. Three-level nested desktops, recursive dirty
  close, Notes/Omega Colors persistence and repeated subtree reclamation pass on
  fresh/cold boots: `desktop/20261025T053839272304Z`.
- All three nested startup faults (picker, selected user, background) pass using
  that current-code nested disk, retaining an outer unsaved draft and preserving
  the source image: `nested-failures/20261025T053938021113Z`.
- All five root startup cases pass, including invalid/crashing Login and manual
  retry with saved data intact: `startup/20261025T054007748870Z`. The existing
  launcher recovery flow passes fresh/cold with repeated explicit retries and
  preserved sibling drafts: `launcher-recovery/20261025T054042898919Z` (`recover`
  mode; the other six launcher-specific modes were not rerun this turn).
- Default host `cargo test --locked` passes. Twenty-three stock matrix tests run
  the compiled Omega policies, including all role/storage launch combinations,
  failures at each stage, rollback/picker obligations, wrong/duplicate completions,
  cancellation and replacement. The first live pass also succeeded in all five
  modes; final fixtures add explicit admission/publication trace assertions.
- Stock matrix guest all-feature and host-test Clippy, desktop guest probe Clippy,
  layout (73 Omega / 150 Rust edges), eight boundary tests, Rust format and diff
  checks pass. Full kernel/ARM64 and unrelated runtime suites were not rerun.
- Logs: `build/omega-component/launch-{host-checks,final-complete,final-revoke,
  final-stop,final-bind,final-place,final-desktop,final-nested,final-nested-failures,
  final-startup,final-recovery}.log`.

**Next:** use these explicit stages to remove remaining blocking host exchanges
from ordinary launch (start with storage binding/publication), preserving the same
command/completion custody and rollback behavior. Then move more service-loop
orchestration into Omega. Do not replace the live F5 service with the independent
synchronous matrix lab. The earlier intermittent partial-key observation remains
open; this work does not claim to resolve it.

Implementation update 2026-10-24: **launcher recovery is a ticketed asynchronous
Omega workflow**, over `8394d522`. Artifact directory stamps below are UTC.

Start at [the recovery entrance](source/distribution/matrix/service/children/recovery/main.omg)
and its [host boundary guide](source/distribution/matrix/service/children/recovery/README.md).
Omega emits read/admission commands once, waits for matching completions and
chooses fallback or cancellation. Rust retains authenticated request custody,
actual IPC, sealed images, task admission and teardown. The storage exchange now
carries a typed caller-or-recovery origin instead of delivering an unlabelled
local result to whichever launcher exists.

Recovery tickets are nonzero, monotonic and shared across launcher replacements
for the matrix process's lifetime. Exhaustion refuses another read; stale or
duplicate completions cannot advance replacement work. Rust retains an image only
for an accepted completion and admission must name that exact ticket. Existing
one-automatic-attempt and authenticated explicit-retry budgets are unchanged.

Cancellation retains existing deferred owner-command ordering: a request needing
a busy storage stream waits, and recovery cannot admit ahead of that queued
request. Processing stop drops recovery even when the already-reclaimed launcher
makes STOP report `PEER_CLOSED`. Successful revocation drops it too. This does not
promise immediate interruption of a provider read. Rust still drains/retires the
wire exchange; admission and fallback remain synchronous host mechanisms. No
kernel, capability grant, production deadline or public protocol changed.

Verification:

- New full-F5 nested fixture holds a real recovery read while sibling Notes accepts
  input, keeps its draft, protects close, saves and reopens on cold boot. Recovery
  occurs exactly once: `desktop/20261025T050147903361Z`.
- Routed owner revocation during that read prevents admission/fallback, preserving
  the same sibling checks on fresh/cold boots: `desktop/20261025T050256958458Z`.
- Routed owner stop likewise passes fresh/cold: `desktop/20261025T050723547764Z`.
  The first stop fixture expected success from an already-dead task and failed its
  assertion; it now checks the existing `PEER_CLOSED` reply plus no recovery.
  Gate/fault/owner probes are enabled only in disposable test profiles.
- All seven existing launcher modes pass (automatic recovery, clean exit, startup
  fault, owner stop/revoke and fallback clean/fault), including fresh/cold recovery
  and repeated explicit retries: `launcher-recovery/20261025T050757988421Z`.
  The minimal one-user source image is unchanged.
- Ordinary F5 fresh/cold desktop passes, including three Notes instances,
  independent drafts, protected close, Colors, persistence and the storage oracle:
  `desktop/20261025T050959493604Z`.
- Default host `cargo test --locked` passes. Stock matrix has twenty tests,
  including compiled command/completion tests for wrong and duplicate tickets,
  cancellation, replacement, bounded retries, admission/read refusal and a Rust
  allocator exhaustion test. No mock substitutes for the compiled Omega policy.
- Stock matrix guest all-feature and host-test Clippy, desktop guest Clippy with
  both new owner probes, layout (72 Omega / 150 Rust edges), eight layout boundary
  tests, Rust formatting and diff checks pass. Full kernel/ARM64 and unrelated
  runtime suites were not rerun.
- Logs: `build/omega-component/async-{recover,revoke,stop,stop-final,launcher-final,
  desktop-final,host-tests}.log`.

**Next:** extend the command/completion approach to ordinary stored-program
launches, preserving the authenticated caller and reply custody in Rust. Keep this
an incremental migration of real F5 orchestration; do not substitute the separate
synchronous Omega matrix lab or invent a general public async ABI prematurely.
The earlier intermittent partial-key observation remains open.

Implementation update 2026-10-24: **local picker/background startup recovery
executes in Omega**, over `7e8c585f`. Artifact directory stamps below are UTC.

Start at [the stock Omega matrix guide](source/distribution/matrix/README.md), then
[startup/main.omg](source/distribution/matrix/service/children/startup/main.omg). Omega owns picker
attempts/pending work, selection-error display and background pending/spent state.
The Rust lifetime handler now dispatches its decisions. Local picker failures get
two total attempts; selected-user failure returns to a fresh error-reporting picker;
background failure gets one retry. Busy child-control/loss work preserves queued
recovery. Owner-managed root pickers do not acquire a second local recovery loop.

Rust retains verified outcomes, local slot roles, realm object IDs, authentication,
program reads, admission and teardown. Admission/placement rejection after retiring
a picker reports selection failure through the same policy. Picker counting now
saturates at exhaustion instead of risking an overflowing byte if the host repeats
admission; no ordinary flow or budget changed. This remains a distribution policy
migration, not an Omega kernel or completed Omega matrix service loop.

The root invalid-Login fixture found an existing ownership gap: admission failure
queued local picker recovery even with `owner_recovers`, then the missing local
picker IDs killed the matrix. Selection-failure handling now respects that flag,
as crash handling already did. Root recovery stays with the enclosing desktop;
local nested admission failures still reset their own picker budget.

Verification:

- Fresh/cold F5 with three nested desktops, recursive dirty-close protection,
  Notes/Omega Colors persistence, repeated subtree reclamation and isolation:
  `desktop/20261025T031736949912Z` (final owner/local handling).
- Using that new disk (current code at every level), all three injected nested
  failures pass: picker exhaustion, repeated selected-user failure and background
  exhaustion with usable inner chrome/Colors. An outer unsaved Notes draft survives
  every cycle; the source image is unchanged. Artifacts:
  `nested-failures/20261025T031906920998Z` (final source).
- The first fault replay exposed an old close-button coordinate in the fixture
  after the extra Omega UI launcher icon shifted running tasks. Recovery had worked,
  but the test selected the task instead of closing it. The fixture now uses the
  shared `Nested.task_close` helper; the complete rerun above passes. Failed attempt:
  `nested-failures/20261025T031159675499Z`.
- All five root startup cases pass, including invalid/crashing Login, unchanged
  saved records and successful manual retry: `startup/20261025T031604708743Z`.
  The invalid-Login failure that exposed the owner/local bug is recorded at
  `startup/20261025T031425319152Z/bad-login`.
- Ordinary F5 fresh/cold desktop, three Notes instances, close protection, Colors
  and independent storage oracle: `desktop/20261025T031650348460Z`.
- Default host `cargo test --locked` passes. Stock matrix has seventeen compiled
  policy tests; five new tests cover all 128 retirement fact/role combinations,
  attempt exhaustion, busy retention, selection reset, independent budgets and
  owner-managed admission rejection without a local restart or budget reset.
- Stock matrix guest all-feature and host-test Clippy, Rust formatting, layout
  (72 Omega / 150 Rust edges), eight layout boundary tests and diff checks pass.
  Full kernel/ARM64 and unrelated runtime suites were not rerun.
- Logs: `build/omega-component/startup-{host-tests,nested,nested-final,failures,
  failures-recheck,failures-final,root,root-recheck,desktop}.log`.

**Next:** define and prove a small asynchronous host boundary before moving the
stock matrix's orchestration into Omega. Keep authenticated Rust request custody
and opaque completion tickets; exercise delayed parent/storage replies while the
matrix continues serving children. The independent Omega matrix lab still uses a
synchronous adapter and must not replace the F5 loop as-is. This is a proposed
migration step, not a new public ABI or owner semantic decision. The earlier
intermittent partial-key observation remains open.

Implementation update 2026-10-24: **stock launcher recovery decisions execute
in Omega**, over `029f3a43`. Artifact directory stamps below are UTC.

Start at [the stock Omega matrix guide](source/distribution/matrix/README.md), then
[recovery/main.omg](source/distribution/matrix/service/children/recovery/main.omg). The existing
single native export now dispatches configuration, close/loss and recovery.
Omega owns the opaque retry phase and the wait/cancel/read/admit/fallback actions.
The Rust phase enum is gone. One automatic attempt is still allowed; authenticated
explicit retries each permit one attempt without replenishing it. Clean exits,
owner stop/revocation and fallback failure never become automatic retry requests.

Rust retains exact task/endpoint authentication, saved placement and storage
identity, asynchronous parent reads, image custody and teardown. Policy receives
only verified value facts. This does not migrate the full stock matrix event loop,
change capability grants or add a desktop workflow. Initial fallback supply still
preserves a running launcher's budget; replacement/revocation cancels pending work,
including an acknowledged retry.

Verification:

- All seven launcher recovery modes pass: recovery, clean exit, startup fault,
  owner stop, owner revoke, fallback clean exit and fallback fault. Fresh/cold
  recovery preserves the sibling Notes draft, later input, close protection and
  save, with exact restart counts and repeated explicit retries. Artifacts:
  `launcher-recovery/20261025T025437249273Z`.
- The fixture upgrades recognized stock code before injecting its launcher fault.
  Input: `launcher-recovery/20261024T054041211303Z/clean/guest.raw`, a minimal
  one-user image, verified unchanged. The first attempt used the full default
  desktop image and the fault installer refused its extra code at the root quota
  before QEMU started (`launcher-recovery/20261025T025329841253Z`). No quota or
  production storage policy was relaxed; use the small fixture for this test.
- Normal F5 fresh/cold desktop passes: three Notes instances, independent drafts,
  protected close, slot reuse, Colors, save/discard/cancel and the storage oracle.
  Artifacts: `desktop/20261025T025649933604Z`.
- Default host `cargo test --locked` passes, including twelve compiled stock
  policy tests. Five new recovery tests exercise retry exhaustion, repeated
  manual attempts, admission/read failure, authority/slot cancellation, busy
  waits, fallback changes and authenticated-request facts.
- Stock matrix guest all-feature and host-test Clippy pass. Layout (71 Omega /
  150 Rust edges), eight boundary tests, Rust formatting and diff checks pass.
  Full kernel/ARM64 and unrelated runtime suites were not rerun.
- Logs: `build/omega-component/recovery-{host-tests,launcher,launcher-recheck,
  desktop}.log`.

**Next:** move the remaining local picker/background startup recovery choices
from `source-rs/distribution/matrix/service/children/startup/lifetime.rs` into
Omega. Preserve owner-managed versus locally managed picker recovery, bounded
attempts, nested selection failure and sibling survival; keep actual admission
and cleanup in Rust. This is a proposed next slice, not an owner semantic change.
The earlier intermittent partial-key observation remains open. Do not promote the
independent matrix lab's synchronous parent adapter into the desktop event loop.

Implementation update 2026-10-24: **normal F5 close/loss decisions execute in
Omega**, over `ce2e94f9`. Artifact directory stamps below are UTC.

Start at [the stock Omega matrix guide](source/distribution/matrix/README.md).
The shared native export now dispatches configuration/placement and
[lifecycle policy](source/distribution/matrix/service/children/control/main.omg). The former
`configuration-native` consumer is now `native`; configuration is a plain package
that the chooser still imports directly. Lifecycle owns survey admission,
pending-answer aggregation, conservative loss, close wait/refuse/stop and the
already-stopped result. It carries only value facts and two survey state words.

Rust retains authenticated requests, exact task/endpoint identity, asynchronous
parent/storage progress, deadlines, stale FIFO reply draining and reclamation.
The temporary bridge packet is isolated from callers' deep IPC stacks. No kernel
mechanism, capability grant, desktop budget or UI workflow changed. This is a
bounded production policy migration, not the entire matrix event loop in Omega.

A clean LOSS answer must now be exactly eight zero words. Previously the matrix
ignored reserved reply fields when deciding that no work would be lost. Malformed
clean-looking replies now preserve the app; the contract documents canonical
reserved fields. The real nested IPC fixture injects this case. Existing delayed
clean/then-dirty replies, control failure, replacement and close/exit races remain
covered, without treating transport failure as task death.

Verification:

- Normal F5 fresh/cold desktop, Notes save/discard/cancel, Colors and independent
  storage oracle: `desktop/20261025T022949628899Z`.
- Three-level nested desktop, continuous pointer movement, delayed parent work,
  close during the storage hold, recursive dirty-close protection, repeated
  reclamation and cold reopen: `desktop/20261025T023133829682Z`.
- `--matrix-test --release`: malformed reserved replies, recursive loss,
  stale clean FIFO replies, isolation and replacement pass.
- `--matrix-storage-test --release`: control failure, close/exit races and
  replacement receipts pass. Its first run found an obsolete fixture assumption:
  bit 4 now means INPUT_PRODUCER. The invalid-flag probe now uses unsupported
  bits instead, and the complete rerun passes.
- Default host `cargo test --locked` passes. Stock matrix has seven compiled
  policy tests: three configuration/placement oracles plus four lifecycle tests
  covering all 32 child subsets, all admission-busy flags, errors, malformed
  words and monotonic uncertainty.
- Targeted Clippy passes: stock matrix guest all features and host tests;
  desktop guest default and storage-lab/trace-storage. The attempted desktop
  all-features union hits the existing unused `database_fresh` field because it
  combines alternative lab entry paths; no unrelated code was changed for it.
- Layout (70 Omega / 150 Rust edges), eight layout boundary tests, Rust formatting
  and diff checks pass. Full kernel/ARM64 runtime suites were not rerun.
- Logs: `build/omega-component/lifecycle-{desktop,nested,host-tests,matrix-test,
  matrix-storage-test,matrix-storage-recheck}.log`. Guest logs are under the paths above or
  `build/boot-harness-rs/{matrix-test,matrix-storage-test}`.

**Next:** migrate the stock launcher's bounded crash/retry/fallback decisions to
Omega while retaining exact task/authority checks and asynchronous reads in Rust.
Exercise the existing launcher recovery fixture with a healthy sibling holding
an unsaved draft. Do not promote the independent matrix lab's synchronous parent
adapter into the desktop loop. The earlier intermittent partial-key observation
remains open. No new owner semantic decision is required for this migration.

Implementation update 2026-10-25: **Omega matrix accepts a real upstream grant
and attenuates it to nested children**, over `eb28b007`.

The same independent Omega program now accepts an optional supplied broker in
its entry environment. It acquires interaction once, derives its three initial
placements from the actual parent dimensions/key mask, and refreshes parent
snapshots before dispatch without losing the retained local request or caller
identity. Invalid/refused bootstrap cannot fall back to the synthetic root grant.
The bridge remains private to the lab; no kernel or common app-host API changed.

`python tools/boot-harness-rs/matrix_composition_test.py --upstream` boots a real
Rust parent provider, the Omega matrix and three ordinary peer processes. The
[parent fixture](source-rs/distribution/lab/matrix-composition/nested/parent.rs)
uses platform World/server/admission and never modifies the inner World directly.
[Behavior checks](source-rs/distribution/lab/matrix-composition/nested/behavior.rs)
cover two coordinate translations, sibling isolation, queued press/release order,
withheld keys, forged owner operations, resize/clipping, stale epochs and
revoke/regrant without replay or automatic descendant revival. Existing semantics
invalidate child placements when the parent's view epoch changes, including on
resize; the first fixture incorrectly expected automatic clipping and was corrected
to require explicit placement. Restart fixtures also reset held pointer state.

Each parent cancels/restarts the matrix with live peers three times and checks
resource counters. It then returns with the matrix and peers still alive; the
supervisor verifies subtree reclamation by restarting the complete composition
with the same loans and unchanged charges. The matrix retains its previous
13-channel and nine-private-page grants. The new parent fixture has three channels
for its supplied broker/interaction connection; no desktop budget changed.

Verification:

- Nested QEMU proof: `omega-matrix-upstream/20261025T003303886919Z` (final,
  includes queued input before revocation and exit with live descendants).
- Original Omega proof: `omega-matrix/20261025T002944651380Z`.
- Original Rust comparison: `matrix-composition/20261025T003044894303Z`.
- Six compiled Omega policy tests pass, including supplied parent geometry,
  request/caller preservation across refresh, parent transport loss and refusal
  before admission. Command: `cargo test --locked -p cathedral-three-peer-matrix
  --lib --features omega`.
- Default host `cargo test --locked` passes; log:
  `build/omega-component/upstream-host-tests.log`. Affected guest Clippy with
  `omega,upstream-proof` passes. Layout (68 Omega / 150 Rust edges), Rust formatting
  and diff checks pass.
- No F5 desktop, full kernel runtime or ARM64 QEMU rerun: this change is confined
  to the independent lab, its profile and documentation.

**Next:** move a bounded stock matrix lifecycle decision into Omega, with a real
F5 regression test. Preserve the stock asynchronous upstream loop: this proof's
adapter serializes parent POLL with local requests and honors existing input
retirement backpressure, but does not consume wake notifications or measure
latency. A slow parent can delay local dispatch; this is not a production runtime
extraction. Rendering and storage forwarding remain outside this proof. The
previous intermittent partial-key observation is still open. No new owner choice
blocks the engineering work.

Implementation update 2026-10-25: **diagnostic boot repaired; shared matrix
configuration/placement and an independent matrix execute in Omega**, over `aa84ff7c`.

The traced display failure was a user-stack guard fault: vector 14, error 6,
address `0x80001ffffc`, instruction `0x800000e067` inside `time::trace` in the
then-built display ELF. The diagnostic writer's 4096-aligned scratch array was
inlined into callers, inflating their frames by several KiB. The crossing-page
write now uses isolated, 256-aligned scratch (the maximum diagnostic write),
which still cannot cross a page. No task stack or resource grant grew. Failed
bootstrap collection now logs returned/fault outcomes before consuming them.
Reproduction: `desktop/20261024T235906067468Z`; fixed full traced four-level
fresh/cold proof: `desktop/20261024T235947413509Z`.

`--trace-input` now also records key/state at source and non-motion event contents,
local channel/group and realm at nested receipt and native Omega delivery. It
includes cancellation/retirement, and the chooser opts into the common host trace.
This is opt-in diagnostic output, not a new input protocol or global identity.
The prior `deep` -> `de` occurrence remains unexplained: it did not recur in either
full replay. The new records expose transport receipt, not app-side acceptance.

The [shared distribution configuration](source/distribution/matrix/service/children/configuration/README.md)
now owns the Omega CTHMAT codec and checked bottom-bar placement. The chooser
imports it instead of owning a private copy. Stock matrix startup and authorized
live layout execute its compiled export through
[source-rs/distribution/matrix/policy/policy.rs](source-rs/distribution/matrix/policy/policy.rs).
Authority checks, admission and IPC remain with their existing owners. The Rust
contract representation stays as the installer/tool format and test oracle.

Verification: compiled conformance covers all 26,624 single-byte mutations,
resource-limit/legacy cases, UTF-8 names and 100,000 geometry combinations. Full
host tests, all-userspace Clippy and source-layout (68 Omega / 149 Rust edges)
pass. The final traced four-level fresh/cold desktop with the shared configuration
also passes: `desktop/20261025T000330943022Z`, log
`build/omega-component/matrix-input-events.log`. Ordinary F5 fresh/cold proof:
`desktop/20261025T000847314830Z`. Settings cancel/live apply/retention/cold
persistence and lost-save reconciliation:
`settings/20261025T001104625287Z`, log `matrix-settings.log`. The first settings
attempt used the heavily populated traced nested disk and correctly refused an
upgrade above its root quota; the successful run used the documented one-user
fixture. No quota was raised or user data removed. No kernel mechanism changed.

The [independent three-peer Omega matrix](source/distribution/lab/matrix-composition/README.md)
also runs the unchanged live IPC isolation/supervision proof. Omega selects
admission, per-child service grants, placement, request dispatch and lifecycle.
Its private lab adapter executes the existing World/server/admission mechanisms,
retaining endpoint-authenticated Request custody across native calls. It does
not add matrix-specific operations to the common application host. The Omega
profile explicitly funds nine state/packet pages. Storage endpoints prove
custody only, not actual filesystem scopes or storage forwarding.

- `python tools/boot-harness-rs/matrix_composition_test.py --omega`:
  `omega-matrix/20261025T001808889160Z` (final), with the first pass at
  `omega-matrix/20261025T001303384827Z`.
- Original Rust comparison proof after shared admission refactor:
  `matrix-composition/20261025T001435958058Z`.
- Four compiled Omega matrix tests: refusal before placement, forged child
  commands stay child dispatch, replacements retain service policy, malformed
  requests/disconnects and admission errors. Run with `cargo test --locked
  -p cathedral-three-peer-matrix --lib --features omega`.
- Default host tests include the stock matrix configuration tests. All-userspace
  and affected all-feature guest Clippy, both matrix host library/test Clippy,
  layout boundary tests (8), packaging tests (5), ARM64 user-runtime Clippy
  and formatting pass.
- Artifacts/logs: `build/omega-component/{matrix-*,omega-matrix-*.log,rust-matrix-proof.log}`.
  Kernel/ARM64 runtime suites and every specialized fault fixture were not rerun.

**Follow-up (completed in the newer entry above):** prove the supplied upstream
connection and nested attenuation while keeping the lab bridge private. Then
move a bounded stock lifecycle branch to Omega using those operations; the
stock matrix's supervision/rendering/storage loop remains Rust. The partial-key
observation is still open, with source/nested/native-delivery traces now available.
No owner semantic decision blocks these engineering steps.

Implementation update 2026-10-24: **installed chooser session moved to Omega**,
over `95163d36`. Normal F5's login picker and desktop taskbar now run the
[complete Omega chooser](source/distribution/applications/chooser/README.md),
alongside Omega Notes and Colors. Omega owns service acquisition sequencing,
realm listing, per-channel pointer edges, task selection/close, settings draft
and save reconciliation, force-stop confirmation, surfaces and rasterization.
The pure policy from the previous step is called directly inside Omega; the
normal executable no longer uses the temporary Rust decision adapter.

The tiny Rust entry declares required services and enters the existing common
`omega-app` host. That host adds bounded incoming `ENTRIES` attachments,
ordinary startup argument delivery, the existing task clock and optional timed
waits. It checks row count/extent, canonical metadata and cursor ordering,
copies only declared rows and releases incoming custody before resuming the app. Names, roles, settings
and launch policy remain distribution code. No kernel mechanism, supervision,
authority grant or resource budget changed. The complete native component is
143,488 bytes; state and packet fit the host's existing nine-page reservation.

The old Rust chooser session is explicitly gated behind `rust-probes` for the
existing recovery/startup-fault fixtures, like the Notes probes. Ordinary F5 has
no automatic fallback. `recovery-probes`, `launcher-startup-fault` and
`picker-startup-fault` select that probe path; `trace-storage` works with the
normal Omega app. Both native components use the supported pinned compiler and
existing packager. Generated code and package-manager workspaces stay ignored.

Verification:

- Twenty Windows tests execute the actual chooser exports (nine pure policy,
  eleven complete-session tests): user copy/launch, full-page listing, malformed
  input, settings preservation/validation, lost acknowledgement readback,
  intervening edit refusal, dirty `LOSS`, exact-ticket close, confirmation
  invalidation, idle task refresh and image epoch races. Five host transfer tests
  cover extents, staging and canonical ordered entries.
- Full default `cargo test --locked`, all-userspace Clippy, default/all-feature
  chooser guest Clippy, host library/test Clippy, formatting, packaging tests
  and source-layout checks pass. Host checks used
  `--target-dir ../build/chooser-checks`. The freestanding binary is checked on
  `x86_64-unknown-none`, not as a Windows binary.
- Default F5 composition: login, independent Notes instances, taskbar switch/
  close, capacity refusal, Colors persistence and cold oracle:
  `build/boot-harness-rs/desktop/20261024T234030867608Z`.
- Settings cancel/save/live geometry, retained running apps, cold persistence
  and independent defaults: `build/boot-harness-rs/settings/20261024T232749764149Z`.
- Lost save acknowledgement reconciled through real storage and cold reopen:
  `build/boot-harness-rs/settings/20261024T234124836572Z`.
- Four recursive desktops, protected close, repeated subtree reclamation,
  deepest Notes/Colors persistence and cold reopen:
  `build/boot-harness-rs/desktop/20261024T233806926362Z`.

Logs are `build/omega-component/chooser-session-*.log`. The initial nested run
(`desktop/20261024T233006146553Z`) found stale taskbar entries after collection:
the common host waited indefinitely after the last surface notification, whereas
the Rust chooser refreshed periodically. WAIT now accepts an optional deadline;
the chooser requests a 100-tick refresh. Timeout resumes with an empty wake and
never synthesizes input. A compiled regression checks removal without input.
The independent-defaults settings run precedes this fix and a visual parity
correction; final F5, recovery, nested and compiled tests use the corrected
component. Kernel,
ARM64 and every specialized fault fixture were not rerun for this app migration.

**Outstanding observations:** `desktop/20261024T233351949583Z` reached four
levels, but the initial Notes input showed only `de` after the harness sent
`deep`. This did not recur in the full fresh/cold replay above; its cause is
unresolved, not dismissed as host load. An attempted `--trace-input` replay
(`desktop/20261024T233702361764Z`) failed both display bootstrap attempts with
PEER_CLOSED, before storage or the chooser started. Ordinary tracing of storage
still works. Preserve these artifacts; restore diagnostic boot and inspect the
source-to-leaf event stream before declaring the deep-input path stable.
These are engineering investigations, not owner semantic questions.

**Next:** investigate those input/diagnostic observations first. Then inventory
the stock matrix's remaining distribution policies and move one bounded slice (saved configuration and placement decisions) into Omega,
with compiled conformance tests before replacing its Rust caller. Reuse that
configuration codec in the chooser instead of growing two implementations.
Keep generic task/IPC supervision in platform and desktop choices in distribution.
The common host does not yet expose the operations needed to run a whole matrix
in Omega; do not disguise matrix-specific orchestration as generic host work.
Background is another small app port, but rewriting the Rust kernel is not the
next migration milestone. No owner decision blocks this work.

Implementation update 2026-10-24: **installed Notes behavior moved to Omega**,
over `442da2eb` (including the prior Colors migration `9e6785ff`). Normal F5,
Split and nested desktops now run
[Omega Notes](source/distribution/applications/notes/README.md), alongside Omega
Colors. Its Rust entry declares imports and enters the shared platform host.
Omega owns bounded document edits, `NTS1` validation, save/reload/copy and uncertain
write reconciliation, close/loss decisions, per-group cursor preference, retained
surfaces and frame-demand acknowledgment. No kernel mechanism changed.

The [host](source-rs/platform/libraries/omega-app/README.md) now accepts explicitly
bounded storage BULK attachments and stages one outgoing object up to 64 KiB.
Incoming byte counts are checked before mapping; returned padding is never copied.
Outgoing chunks must form a complete contiguous prefix before freezing and sending;
errors/exit release custody. Notes sends its 46,080-byte text image in two chunks,
retaining unchanged decorations. Its opaque state/packet still use nine private
pages. The component is about 52 KiB; its installed ELF is about 85 KiB. This is
still a synchronous x86-64 bridge, not the final Omega app ABI.

Guest testing caught two migration regressions and fixed them:

- Split's 32-page Notes reservation could not hold the nine-page host beside old
  and replacement text images plus decorations. Notes now gets 40 pages there
  and in the Notes lab. The inner Split receives 112 pages; the desktop caps
  ordinary programs requesting descendants at 144 private pages, still bounded
  by the owner offer. Ordinary leaves retain their 128-page ceiling. These are
  distribution grants, not enlarged kernel limits.
- Binding storage before placement raced the lab owner's realm assignment. Notes
  now waits for a valid view before initial binding, preserving the first input
  across loading. A compiled-export regression covers delayed placement.

The coexistence replay also exposed an existing surface-funding problem: an
ordinary Split consumed all spare surfaces merely because it requested descendant
capacity, preventing a sibling Notes launch. Ordinary descendant hosts now receive
at most sixteen surfaces, including spare room for producer/capture children.
Expansion to the full configured ceiling requires an explicit surface profile;
the declared stock nested desktop retains its existing policy.

Ten Windows-hosted tests execute the actual Notes export; the original Rust
schema test remains independent. Three host boundary tests cover transfer bounds;
all eleven compiled Colors tests still pass. Default Rust host tests, userspace
Clippy, Notes all-feature guest Clippy, Python packaging tests and source-layout
checks pass. Guest evidence under `build/boot-harness-rs/`:

- Default F5, three Notes instances, close choices, save/copy/cold-open and the
  exact storage oracle: `desktop/20261024T212429957429Z`.
- Four nested desktops, deepest Notes/Colors persistence and repeated teardown:
  `desktop/20261024T212530428453Z`.
- Capture of hidden Notes, sibling input, repeated grants/teardown and cold-open:
  `desktop/20261024T214205821134Z` (final sixteen-surface policy).
- Optional storage withheld, draft protection and independent Colors persistence:
  `desktop/20261024T212947413622Z`.
- Lost committed reply, draft-preserving reconciliation, long/middle edits,
  glyphs and exact durable history/quota through cold boot:
  `matrix-database/20261024T213540707866Z`.
- Split alongside ordinary Notes/Colors, resize, same-code inner Split,
  bounded further refusal, recursive close and cold reopen:
  `desktop/20261024T214241174330Z` (final sixteen-surface policy).

The default, four-level and optional-service runs precede the final delayed-
placement fix; recovery, capture and nested Split exercise it. Native checks
were rerun on the final source. Logs live in `build/omega-component/` under
`notes-final-checks.log`, `notes-review-checks.log`, `notes-final-guest-0.log`
and `notes-verified-{capture,split}.log`.
The commit was rebased over the concurrent Omega UI tooling consolidation;
the combined tree passes all eleven UI tooling tests, five component packaging
tests and the layout checker. That upstream change relocates tooling/docs and
changes only a comment in native font source.

The explicit Notes fault/admission/producer/render feature sets now select
`rust-probes`; those fixtures retain the old Rust session and do not prove the
Omega editor. Native startup preserves `note-loaded`; detailed Rust editor timing
markers have not been ported. Storage tracing uses generic host spans keyed by
supplied realm; the timing summarizer accepts both implementations. Full kernel,
ARM64 and every fault fixture were not rerun for this userspace change.

**Next:** migrate the chooser/user-picker's model, selection and launch/close
policy into Omega, using the same scoped host. First inventory its incoming realm
entry attachments and layout/launch replies; extend only the transport it needs.
Keep matrix supervision and service mechanisms in Rust until a concrete Omega
consumer needs them. This is a stronger next test than writing more Rust UX or
attempting to port the whole matrix at once.

Implementation update 2026-10-24: **installed Colors behavior moved to Omega**,
over `c2f1421b`. Normal F5 and nested desktops install the same Colors executable;
its app logic now starts at
[Omega Colors](source/distribution/applications/colors/README.md). Omega owns
service requests, input decisions, `MC01` persistence/reconciliation, local
surface publication and close/loss policy. Rust retains the import record,
executable entry and a reusable
[platform transport host](source-rs/platform/libraries/omega-app/README.md).
The old Rust Colors library remains only for mechanism-lab probes.

The native export is 17,504 bytes in a roughly 45 KiB installed ELF. Its opaque
state/packet use nine private pages charged to the app; outgoing images have
separate bounded custody. Existing scoped services, kernel ABI, quotas and
launcher/profile selection remain in use. The host is x86-64 and synchronous;
incoming object attachments are explicitly unsupported. This is a transitional
binding, not the final Omega app ABI or standalone Omega OS boot.

[Native packaging](tools/omega-component/README.md) checks the component, hashes
source/compiler/artifacts, and serializes concurrent Cargo/editor builds. UI and
app packaging share the ELF validator. The app entrance descends into named
session, storage and rendering folders. Confirmation text now fits its bitmap;
input batches are bounded and owner control precedes wake hints.

Verified: eleven tests execute the actual Omega export against controlled peers;
they cover scoped requests, cold reads, uncertain writes/creation, malformed
records, close confirmation, pointer cancellation/rebind, hidden rendering and
input batching. Default Rust host tests, userspace Clippy, Python packaging tests
and source-layout checks pass. Guest evidence under `build/boot-harness-rs/`:

- Ordinary F5 desktop, save and cold-open with the host storage oracle:
  `desktop/20261024T204804258337Z`.
- Four nested desktops, deepest Omega Colors save/cold-open, independent outer
  color and repeated subtree teardown: `desktop/20261024T205111666166Z`.
- Lost committed storage reply, explicit reconciliation, close/reopen and cold
  boot on a retained-disk copy: `colors/20261024T205246324412Z`.

The first two runs precede the final named-continuation cleanup; the recovery
run exercises that code. No kernel mechanisms changed; full kernel/ARM64 and all
fault fixtures were not rerun. Native tests are Windows-hosted; guest execution
is x86-64. Build logs and receipts live in `build/omega-component/`.

**Follow-up at that baseline (completed above):** migrate Notes using the same host. First add
a bounded incoming-object interface for its bulk reads; keep transport/custody
in platform and document schema/draft/close decisions in distribution Omega.
Then evaluate the launcher/matrix migration against what Omega can actually
compile. Keep Rust work focused on missing mechanisms and conformance evidence,
rather than building a second long-lived application stack. The longer queues
below remain useful context, not a requirement to finish more Rust UX first.

Implementation update 2026-10-24: **local Omega UI build and combined desktop replay**,
over `4f163da2`. The supported Omega `16a35ba7f429` compiler is built in the ignored
`build/omega-ui/compiler` checkout. UI preparation selects `--compiler`, then
`OMEGA_BIN`, then that local optimized binary, then PATH. Ordinary F5 therefore
uses the prepared local compiler without changing the global Omega installation.
The [build guide](tools/omega-ui/README.md#build) gives reproducible setup commands;
the checked-in UI remains distribution source, not a sibling dependency.

Combined replay exposed and repaired two resource regressions:

- Split's fourth child left only 128 physical frames for its owner, with 117
  already charged; replacing a 14-page header failed. Capture admission now asks
  its owner for physical headroom through the existing `FUND` protocol, preserving
  its own load/private backing. Refusal follows the existing CANNOT CAPTURE path.
  Kernel limits and general subtree reservation defaults are unchanged.
- Eight launcher surfaces plus the background left 28 surfaces at the fourth
  nesting boundary, below the host's former 32-surface declaration. Stock startup
  and its manifest now share the actual 24-surface minimum. Extra surfaces are
  lent within the owner's ceiling and disjoint free range, independently of that
  minimum. The latency harness also checks the running-task area after the new
  fifth launcher icon instead of mistaking that icon for an uncollected task.

Verified on this Windows checkout: `cargo test --locked`, `cargo check-userspace`,
eight native UI tests and the usage example, nine Python packaging/setup tests,
source layout (52 Omega / 139 Rust edges), Rust formatting and diff checks.
The default F5 composition builds without an environment override
(`build/omega-ui/final-f5-build.log`). Native exports were rebuilt from checked-in source; the frame is 531,360 bytes.
Fresh/cold QEMU evidence under `build/boot-harness-rs/`:

- UI buttons, editing, dynamic region retirement/restoration and close/reopen:
  `omega-ui/20261024T201430679125Z`.
- Recursive pointer preferences: `desktop/20261024T200106102735Z`;
  independent AUTO suppression: `desktop/20261024T200200137927Z`.
- Masked capture, recipient bytes, hidden editing and saved state:
  `desktop/20261024T201506069107Z` (final tree).
- Four nested desktops, input, durable saves and exact cold-open content:
  `desktop/20261024T201312796565Z`, with the final `s` matrix optimization.

The cursor runs precede the later capture/surface-admission fixes; their unchanged
routing implementation was exercised in the combined UI tree. Full kernel smoke,
ARM64 and every fault fixture were not rerun for these distribution/build changes.
Timing numbers remain host-observed upper bounds, not guest-stage measurements.

Implementation update 2026-10-24: **self-contained Omega UI source**. The complete
library, native backend, fonts, examples and tests are checked in under
`source/distribution/libraries/omega-ui`; the Rust API is in
`source-rs/distribution/libraries/omega-ui`. Cargo uses that source directly;
there is no sibling UI checkout, submodule or source-path override. The
[build guide](tools/omega-ui/README.md#build) links the remotely available compiler
producer and documents `OMEGA_BIN`/PATH setup.

Verification starts from an archive of Git-tracked files, with no build output,
prepared components or sibling UI directory. Both native exports compile there;
the 531,360-byte frame matches the previously tested component exactly. Eight
Rust unit tests plus the usage example, six packaging tests, four library helper
tests and layout (52 Omega / 139 Rust edges) pass. Fresh/cold QEMU physical input,
region removal/restoration, editing and close/reopen pass from that checkout.
Replay paths and exact input/output hashes are retained in
`build/omega-ui/source-checkout.json` and `checked-in-source-*.log`.
This closes the missing-source F5 build gap described below. Backend bounds and
unverified ARM64, Unicode/IME and monitor-refresh behavior remain unchanged.

Implementation update 2026-10-24: **recursive pointer-target cursor preferences**,
integrated with `1687e07b` (including reusable Omega UI). F5 -> user -> Notes
(or Split) -> click a note -> **F6**. Notes
requests default/hidden for pointer aliases in that keyboard's local group;
input keeps working. Leaving the target restores the next target's policy.
Returning to the same live view can reuse its preference. Closing the target
restores owner policy without another motion event. Split and stock matrices
compose preferences recursively; hiding the physical pointer does not hide AUTO,
and AUTO can independently request suppression through its own Notes input group.

Start at the platform [cursor policy](source-rs/platform/libraries/matrix/world/input/cursor/cursor.rs):
current hover/drag target -> delivered child alias -> preference. The
[receiver](source-rs/platform/libraries/matrix/world/children/cursor.rs) rejects
unknown/key-only aliases, stale view epochs and invalid modes. Cancellation,
retirement and rebind clear preferences; keyboard-focus cancellation does not.
Stored coordinates allow geometry/death recomputation without fresh motion.
The runtime [forwarder](source-rs/platform/libraries/user-runtime/matrix/upstream/cursors.rs)
sends only changes, keyed by view, channel and cancellation cycle. Counter
exhaustion stays invalid rather than reviving after another cancellation.
Only the root adapter has the display control operation. No new syscall, device
grant, general cursor assets, relative lock, confinement or attribution promise.

Verified fresh/cold QEMU evidence before the concurrent Omega UI integration:

- Ordinary F6, continued typing/saving, independent AUTO, target death and
  Split-in-Split: `build/boot-harness-rs/desktop/20261024T185604001729Z`
  (`--cursor-preferences --trace-storage`).
- Disposable AUTO invokes F6 on its own group; its cursor hides while physical
  pointer/typing remain usable: `build/boot-harness-rs/desktop/20261024T185747833985Z`
  (`--cursor-producer --trace-storage`). The feature uses ordinary producer keys,
  not a privileged cursor test operation.
- Four stock nested desktops, editing/saving/closing and exact cold-open data:
  `build/boot-harness-rs/desktop/20261024T185832793508Z`
  (`--latency --nested-depth 4 --trace-storage`). These are host upper bounds,
  including input pacing/QMP/screenshot polling, not stage latency or an A/B
  performance claim. No input/redraw traces were requested in this run.

- Existing AUTO repeat/resize/retirement regression:
  `build/boot-harness-rs/desktop/20261024T185950798382Z` (`--participants --trace-storage`).
- Masked capture recipient byte inspection, hidden editing and cold state:
  `build/boot-harness-rs/desktop/20261024T190049500362Z` (`--capture`).

**Combined-tree verification.** The original integration omitted its Omega source,
so a fresh checkout could not build the F5 desktop. The source now lives in
`source/distribution/libraries/omega-ui`; the Rust API lives in the matching
`source-rs/distribution/libraries/omega-ui` package. Cargo builds the checked-in
source, with no sibling UI checkout or source override. The
[build guide](tools/omega-ui/README.md#build) documents the supported Omega compiler.
Current source-only and guest verification is recorded in the entry above.

The cursor build initially exceeded the former 262,144-byte executable-object
cap. A temporary `z` build permitted the pre-integration runs above. Incoming
Omega UI support independently raises funded object geometry to 1 MiB; the final
tree restores the stock matrix's existing `s` optimization. No extra limit increase
belongs to the cursor work. The pre-integration latency run is not evidence about
the final combined binary or compiler-setting performance. No new ARM64/GPU claim.

Next: measure actual input/routing/app/presentation stages before changing
forwarding or scheduling. The missing-source and combined-tree replay work is
closed by the entry above. Existing end-to-end
timings alone cannot separate harness waits from guest delay. A small app
interaction-description/semantic-action proof can follow using the accepted input
authority model. Further broad specification is unnecessary for these steps.
The later accepted input contract excludes mandatory agent attribution from child
input/cursor APIs; owner-side session UI remains separately authorized.

Current integration 2026-10-24: **reusable Omega UI library**, over `ab0d76f8`.
`cathedral-omega-ui` exposes caller-authored frames, independent field values,
stable control IDs and checked instance/build-local nodes. Applications compose
rows, columns, cards, text, four button variants, toggles, checkboxes, sliders,
progress and fields. Native Omega owns control behavior, editing, motion, layout
and pixels. The demo now owns its model/view in Rust and uses only this public
API; changing app layout does not require changing the native exports. Its volume
switch removes/restores a region, exercising atomic surface retirement/recreation.

The checked-in distribution library contains ABI 2. An app's Cargo dependency
prepares the same checked component automatically,
without a demo-name rule in the boot harness. The [integration guide](tools/omega-ui/README.md)
contains the lifecycle, typed example, build inputs and current bounds. Independent
host views, multiple fields/instances, malformed packets, invalid frames, motion,
and buffer reuse/stale tokens are covered through the actual compiled exports.

The producer remains optimized Omega `16a35ba7f429`; exact source, producer and
export hashes are in `build/omega-ui/component/receipt.json`. Generic component
code is 531,360 bytes and state is 44,584 bytes. Native program objects permit
256 pages (1 MiB); private/reader caps remain 1024/4096 and root quota remains
2 MiB. Ordinary apps retain their 128-private-page grant. Physical funding and
caller admission ceilings still gate allocation; the image ledger records extra
funded translation tables without applying a second fixed admission ceiling.
The maximum-object retirement fixture keeps its bitmap/reference/custody coverage
inside the unchanged aggregate account budget. Storage splits funding between
cache and outstanding replies; reserving the protocol maximum unconditionally
had reduced its cache and regressed user creation to 6.2 seconds. The balanced
policy restores the observed upper bound to approximately 283 ms under TCG.
Maintenance remains at half the root quota, independently of object size.

Verification: 275 focused Rust tests and the library's typechecked example;
Omega UI's library suite passes native and interpreted execution. Packaging,
source-layout (40 Omega / 133 Rust edges), host and guest clippy checks pass.
Fresh/cold physical input, dynamic region removal/restoration, text editing,
clear/reset and explicit presenter retirement/close/reopen pass at
`build/boot-harness-rs/omega-ui/20261024T181256060582Z`. The ordinary desktop,
protected drafts, capacity refusal, slot reuse, persistent Notes/Colors and cold
read-back pass at `build/boot-harness-rs/desktop/20261024T175140408246Z` with the
independent storage oracle. The full kernel smoke suite passes at
`build/omega-ui/reusable-full-smoke.log`, including maximal page-object mapping,
sealing, retirement, admission refusal and reclamation. Later source-only comment
and indentation changes are checked by comparing regenerated component bytes.

The backend is x86-64 with grayscale Inter and bounded ASCII editing. App code
still supplies scoped input, animation scheduling, persistence and model policy.
Unicode/IME, clipboard/modifier delivery in this demo, ARM64 and monitor-refresh
performance remain outside this result. The existing input-acquisition frontier
below remains open.

Implementation update 2026-10-24: **authorized masked capture through F5**, over
`e6f3fa48`. Enter a user -> Split -> CAPTURE. A separately admitted viewer
receives a fresh 200 by 104 pixel crop of the right Notes, with its known text
rectangle blacked out before transfer. HIDE RIGHT can remove that source from
the display while capture independently requests a frame. CLEAR retires the
viewer and its presentation. AUTO can continue editing the hidden note through
its private route. This is fixed demonstration policy, not general app-published
privacy masks or a screen recording UI.

The platform adds a separately declared/offered CAPTURE service, bounded
BEGIN/TAKE/CANCEL wire, recipient client and optional admission connections.
Ordinary `ALL` offers are unchanged. The stock matrix filters offers to services
it actually implements; only Split supplies capture. Positional legacy startup
cannot falsely bind a broker-only service. Shared admission retains partial
transfers for cleanup. No new kernel syscall or device authority was added.

Start at [Split capture](source-rs/distribution/applications/split/session/capture/capture.rs):
owner-selected endpoint/view identities -> funded private destination -> source
frame request -> acknowledged local scene -> masked raster -> sealed transfer.
Source readers are pinned during one dispatch, temporarily mapped and then
unmapped. The viewer can publish only in its own rectangle and has no storage,
input-production or private-pixel allocation grant. A pending frame expires after
300 ticks; expiry releases demand without killing the source. Source/recipient
loss or parent geometry change retires the demo. Already transferred readers
follow ordinary kernel custody rather than being revoked underneath a process.

Split now declares 20 channels/four descendants: two Notes, optional producer
and optional viewer. Its initial viewer reservation unnecessarily held 16 private
pages and starved provider output; actual IPC returned NO_MEMORY. The viewer now
reserves zero private pages and 32 readers, and errors are logged instead of
silently exiting. The output stays charged to the provider while retained. The widened header also
exposed an 80-page inner Split budget that could not cover two Notes reservations
plus old/replacement headers. Its local grant is now 96 pages (64 + 28, rounded),
within the enclosing 128-page account. No kernel budget limit changed.

Verified fresh/cold desktop runs:

- Actual recipient redaction/RGBX/page-padding inspection, hidden edits, repeated
  capture, release and saved state: `build/boot-harness-rs/desktop/20261024T181758330628Z` (`--capture`).
- Source exit before acknowledgment, sibling survival and repeated subtree reuse:
  `build/boot-harness-rs/desktop/20261024T182042104787Z` (`--capture-case source`).
- Recipient exit with pending work, three attempts in the same live provider:
  `build/boot-harness-rs/desktop/20261024T182219641612Z` (`--capture-case recipient`).
- Explicit timeout, source survival/restoration and repeated requests:
  `build/boot-harness-rs/desktop/20261024T182450709002Z` (`--capture-case timeout`).
- Ordinary viewer/source without observer/fault features, repeated delivery and
  cold saved documents with storage traces:
  `build/boot-harness-rs/desktop/20261024T182707033719Z` (`--capture-case ordinary --trace-storage`).

Split-in-Split regression with dirty close, resize, funded refusal and cold saved
state: `build/boot-harness-rs/desktop/20261024T183222489138Z`.
Four recursive stock desktops, repeated subtree reclamation/isolation and cold
saved data: `build/boot-harness-rs/desktop/20261024T183345130924Z`.

Workspace host tests pass (105 matrix / 19 platform-contract tests), as do ordinary
userspace clippy and capture/source observer-feature clippy. Layout passes with
40 Omega / 131 Rust edges. The traced stock matrix remains 259,448 bytes, near the
262,144-byte executable-object cap. The scalar capture path has no latency bound,
GPU/remote claim or new ARM64 evidence. Recursive source readiness, dynamic mask
publication, capture rate negotiation and general view APIs remain unimplemented.

Next: recursive pointer-target cursor preferences (including no-cursor), with
owner restoration on target change/death, through the ordinary desktop. The
accepted presentation contract permits that work; it now excludes automatic
upstream attribution from child cursor interfaces. Measure nested input/rendering costs
before changing forwarding or scheduling architecture. Keep this bounded capture
proof available while developing those next interfaces.

Implementation update 2026-10-24: **bounded capture policy and frame handshake**,
over `bcc02186`. This is capture implementation groundwork, **not a working F5
capture service**. The platform [capture entrance](source-rs/platform/libraries/matrix/capture/capture.rs)
now owns one endpoint/view-bound lease: authenticate, request, expire, retain a
completed result through delivery backpressure, and retire it on binding loss.
Its [geometry/raster helpers](source-rs/platform/libraries/matrix/capture/README.md)
validate crops/masks, render stable solid/image/cursor layers, redact before
sampling, clear RGBX/page padding, and refuse malformed backing without partial
publication. The concrete lab limits are eight masks, 64 layers and 32,768 output
pixels. These are bounded software mechanisms, not a frozen GPU or capture ABI.

`World::request_frame` adds independent demand for one source. POLL bit 54 asks
that source to query FRAME_DEMAND and acknowledge FRAME_READY with the returned
ticket after successful scene publication. Notes now supports the handshake;
default matrices do not yet request it. View/endpoint/surface-allowance changes
invalidate pending requests, and old acknowledgment/cleanup cannot affect a
replacement. Owner release removes this demand without changing visibility or
input. This is cooperative source readiness, not a display fence or proof that
a hostile app performed fresh work.

Host tests cover exact endpoint/source matching, stale requests, timeout/wrap,
regrant, counter exhaustion, retained payload drops, mask/crop/stride/padding,
cursor transparency and the composed hidden-frame/result/release flow. Workspace
host tests pass; final matrix suite has 104 tests. Ordinary userspace clippy,
matrix library/test clippy, layout, formatting and whitespace checks pass.
Existing hidden-edit/teardown/cold-reopen QEMU regression with storage tracing:
`build/boot-harness-rs/desktop/20261024T175232564736Z`. That run verifies unchanged
installed behavior; it **does not exercise a capture recipient or source frame
request over IPC**. The traced stock matrix is 259,448 bytes, still close to the
262,144-byte executable-object cap. No new kernel, ARM64 or GPU claim.

Next remains **end-to-end authorized masked capture in the desktop**:

- Add a separately declared/offered capture service, with an authenticated
  recipient endpoint. Existing ordinary app offers must not gain capture.
- Connect the lease to actual funded source readers/output pages, the source
  frame handshake and coherent retained-scene pinning. Lease request IDs and
  World frame tickets are distinct and must be associated by the provider.
- Freeze the successful masked output before transfer. Delivery backpressure
  retains one charged result. Reconcile participant loss before dispatch and
  release source frame demand on success, refusal, timeout or recipient loss.
- Install a real consumer/demo through F5; prove masked bytes reach that separate
  process while the source remains hidden, without focus/placement changes.
  Exercise source/recipient death and cold reopen. Recursive source readiness
  requires descendant coordination; current matrices do not implement that.

[Provider obligations](source-rs/platform/libraries/matrix/capture/README.md)
record the precise sequence and remaining boundaries. Caller-supplied identities,
plans and clocks are trusted provider inputs, not claims an IPC client may supply
as authority. A `Lease<T>` alone does not enforce kernel custody or fund storage;
those are required integration work. No owner question blocks the next step.


Implementation update 2026-10-24: **display demand without input revocation**,
over `fa202b62`. The platform [demand owner](source-rs/platform/libraries/matrix/world/demand.rs)
composes inherited display demand with local visibility and publishes a scoped
`NO_FRAMES` hint in `POLL`. Demand changes wake the child without changing its
view epoch, dimensions, input binding or surface allowance. Hiding excludes new
pointer hits but retains already-selected keyboard routes and pointer capture.
Input deactivation is a separate owner operation; the stock taskbar explicitly
uses it for the previous app after a successful switch. No kernel or default
app authority changed.

Notes and Colors keep processing state while deferring drawing. F5 -> user ->
Split now offers **HIDE RIGHT / SHOW RIGHT** alongside AUTO. Start AUTO, hide the
right note while it types, then show the updated document. Split forwards retained
sealed images to hide/show, without mapping child pixels. Its rendering entrance
now descends into header and visibility responsibilities; `mod.rs` is plumbing.
The control appears at widths of at least 384 pixels. Replay is per surface, not
a new atomic scene guarantee. Current demand is display-only, not capture demand
or a mechanism for forcibly stopping an app's own CPU spending.

Verification: workspace host tests (93 matrix / 18 platform-contract tests),
ordinary userspace clippy, demand observer features, matrix library/test clippy,
layout (40 Omega / 128 Rust edges), formatting, Python syntax and local doc links.
The new `--render-demand` desktop fixture requires actual hidden edits and no
right-child drawing submissions after acknowledgment; it checks fresh-hit
exclusion, usable sibling input, restored text, repeated AUTO, hidden-child
teardown, same-session slot reuse and exact cold-open text. Observer features
compile out of ordinary F5 and add no test authority. QEMU evidence:

- Hidden editing, no publication, teardown/reopen and cold saved data: `build/boot-harness-rs/desktop/20261024T173504057414Z`.
- Ordinary AUTO/STOP/resize and cold saved data: `build/boot-harness-rs/desktop/20261024T173355266040Z`; final policy with storage tracing: `build/boot-harness-rs/desktop/20261024T173701517937Z`.
- Four recursive desktops, repeated teardown, isolation and cold saved data: `build/boot-harness-rs/desktop/20261024T173552370947Z`.

The first demand observer attempt missed the kernel's serial prefix while
matching lines; the guest already logged all three hidden edits. The parser was
corrected, then the complete workflow passed, and the expanded teardown workflow
passed on fresh and cold boots. These are bounded lab proofs, not a latency bound,
a complete capture implementation or new ARM64/GPU evidence. The stock matrix
with storage tracing is 259,328 bytes, close to the 262,144-byte executable-object
cap; account for that lab limit before adding the capture provider.

Next: **authorized masked capture**, using this display-demand baseline. Define
its concrete bounded request/result and image/mask publication contract before
implementation: a separate supplied capability, only authorized output, masks
applied before delivery, coherent image/mask pairing, explicit pending/refusal
outcomes and funded retained backing. Capture demand must be counted separately
from display visibility, and participant loss must release it. Exercise a hidden
app from the ordinary desktop without raising it or stealing focus. The accepted
spec permits this work; no new owner decision blocks it. General device/control
vocabularies, mutable descriptors/shared registry pages, cursor preferences and
coordination acquisition remain queued.


Implementation update 2026-10-24: **bounded channel descriptions through nesting**,
over `e00916ce`. A producer can describe only its own granted channel ordinals,
using a UTF-8 label of up to sixteen bytes and advisory pointer/key categories.
The [description boundary](source-rs/platform/contracts/input/description.rs)
validates encoding; the [router](source-rs/platform/libraries/matrix/world/input/descriptions.rs)
rejects changes and late first publication after delivery, even if former
receivers have since gone away. Identical repeats are harmless. Unknown channels
remain supported. Labels are virtual hints, not authenticated origin or authority;
the categories do not impose an event filter.

A separately retained DESCRIBED boundary precedes first input on each receiving
edge. It survives queue overflow, cancellation and retirement, and nested routers
retain it before forwarding ordinary events. No descriptor lookup is added to the
routing hot path, and ordinary events carry no extra strings. Only channels
actually routed to a receiver become visible there. Directory reads include the
immutable descriptions under the same snapshot revision; the runtime bounds one
attempt to four rows, four descriptions and the terminal read. The stock adapter
publishes Pointer/Keyboard; installed AUTO publishes AUTO pointer/AUTO keys.
No kernel or default app authority changed.

Host tests cover canonical UTF-8, source/ordinal/generation authorization,
immutable descriptions, late-publication refusal, sibling privacy, nested local
names, interruption and retirement. Workspace host tests pass (90 matrix / 18
platform-contract tests), as do userspace and relevant host/receiver/desktop
clippy, layout, format and Python checks. QEMU evidence:

- Held input / STOP / descriptions / cold reopen: `build/boot-harness-rs/desktop/20261024T170712750540Z`.
- Overflow / retained descriptions / fresh input / cold reopen: `build/boot-harness-rs/desktop/20261024T170837064536Z`.
- Ordinary uninstrumented AUTO / STOP / resize / cold reopen: `build/boot-harness-rs/desktop/20261024T171308221692Z`.
- Recursive IPC locality, recovery, registration and producer checks: `build/boot-harness-rs/matrix-test`.
- Four nested desktops, repeated cleanup, isolation and cold saved data: `build/boot-harness-rs/desktop/20261024T171418714927Z`.

An initial observer run correctly encountered BUSY when another channel arrived
mid-snapshot; its test now accepts retained metadata from a later successful read
instead of requiring a log on that one attempt. The recursive fixture now checks
metadata boundaries and allows bounded progress before concluding a leaf queue is
empty: NEXT never guaranteed completion of input forwarding through ancestors.
Neither adjustment changes production routing. The traced stock matrix remains
255,224 bytes, within the 256 KiB object cap. These bounded proofs do not establish
a latency bound, full control vocabulary or new ARM64/GPU behavior.

Next: rendering demand and masked capture through the ordinary desktop. Hide an
app's presentation without suspending execution or withdrawing input; let an
authorized capture request pixels without moving/raising it; preserve masking and
release retained resources on participant death. General control/axis descriptions,
mutable descriptors, shared registry pages, separately delegated discovery,
cursor preferences, reassociation, independent group controllers and coordination
acquisition remain queued. No new owner question blocks that work.


Implementation update 2026-10-24: **scoped input binding directory**, over
`92b2f832`. The ordinary interaction protocol can enumerate only channel/group
aliases already routed into its receiver's view. Reads neither consume input nor
create bindings. A bounded runtime reader returns a coherent revision snapshot
or BUSY when membership changes; the existing coalesced wake hint signals changes.
Cancellation preserves the binding. Retirement marks its row until the receiver
consumes RETIRED; replacement channels receive fresh names. View replacement
invalidates old snapshots, and revision exhaustion fails observation closed.
Sibling activity does not change the directory; nested names remain local.

The [child entrance](source-rs/platform/libraries/matrix/world/children/children.rs)
now owns a sibling directory implementation and lifetime tests; mod.rs remains
plumbing. The [runtime reader](source-rs/platform/libraries/user-runtime/matrix/input.rs)
validates bounded ordered replies and leaves retry timing to the caller. No kernel
or default app authority changed. This is a binding directory, not the complete
producer descriptor registry: the key field is the receiver's view-wide ceiling,
not a hardware description. Friendly names/control descriptions, separately
delegated discovery and shared read-only tables remain outstanding.

Verification: workspace host tests (86 matrix), ordinary userspace clippy,
receiver-probe clippy, matrix library/test clippy, layout (40 Omega / 128 Rust
edges), format, Python syntax, documentation links and whitespace. The disposable
Notes observer reads through ordinary IPC and validates event membership and
post-consumption retirement removal. The host requires successful snapshots as
well as existing interruption, independent-input and exact saved-text evidence.
Fresh/cold QEMU proofs:

- Held input / STOP / replacement: `build/boot-harness-rs/desktop/20261024T165242365461Z`.
- Receiver stall / overflow / fresh input: `build/boot-harness-rs/desktop/20261024T165432758549Z`.
- Ordinary uninstrumented AUTO / STOP / resize / persistence: `build/boot-harness-rs/desktop/20261024T165547984154Z`.

The traced stock matrix is 254,648 bytes, still within the 256 KiB object cap.
These are bounded lab proofs, not general registry delegation, all concurrent
interleavings, a latency bound or a new ARM64/GPU result. The initial host test
used decreasing owner source names and correctly hit existing anti-reuse refusal;
its fixture now uses fresh increasing names. No routing policy changed for it.

Next: attach scoped friendly/control descriptions to the established binding
lifetime, preserving source/receiver separation and bounded discovery. Then work
through rendering demand and masked capture in the ordinary desktop: hidden apps
can remain interactive, authorized capture requests pixels without moving the
app, and participant death releases retained resources. General cursor preferences,
reassociation, independent group controllers and coordination acquisition remain
queued. No new owner decision blocks this sequence.


Implementation update 2026-10-24: **input interruption over desktop IPC**, over
`21052cfc`. The installed Split admission path now has disposable fault fixtures;
normal F5/AUTO behavior and routing policy are unchanged. Producer variants hold
pointer/key input until STOP, close their producer endpoint while holding, or
flood one channel during a deliberate Notes receiver stall. The receiver observer
checks holds, orphan releases/repeats, channel/group retirement ordering and
non-reuse of retired identities within the view binding. No test authority or
special dispatch was added to Split or the platform.

The pressure proof requires the flood to occur inside the observed stall. It sees
one channel cancellation, preserves a physical key queued to the same stalled
app, keeps the other Notes usable and verifies fresh producer input after the
interruption. Hold/disconnect proofs observe both held channels retire before
the group, then exercise physical input and a new producer generation. Each
workflow repeats in the same matrix, saves exact text and cold-opens it.
No routing bug was found by these cases; this closes an integrated evidence gap.

Verification: workspace host tests, ordinary userspace clippy, all three producer
probe feature checks, receiver-probe clippy, layout, formatting and Python syntax.
Fresh/cold QEMU proofs:

- Held input and explicit STOP: `build/boot-harness-rs/desktop/20261024T163424046472Z`.
- Producer endpoint loss while held: `build/boot-harness-rs/desktop/20261024T163625497753Z`.
- Receiver stall/overflow and fresh input: `build/boot-harness-rs/desktop/20261024T163737002217Z`.
- Ordinary uninstrumented AUTO, STOP, resize and persistence: `build/boot-harness-rs/desktop/20261024T163845515726Z`.

These are bounded nominal-QEMU fault proofs, not a latency bound, proof of every
scheduling interleaving, arbitrary device reconnect or an ARM64/GPU result. The
observer's bounded retirement history is test-only. Group-fair delivery does not
promise global ordering between independent groups; identity checks
track retirement, not an assumed increasing arrival order.

Next: implement scoped channel descriptions/discovery under
[the accepted channel contract](wiki/spec/human_surface/input_channels.md).
Receiving metadata for granted input must not enumerate ancestor registries or
grant injection. Preserve coherent snapshot/change observation and retirement
while testing the mechanism through the installed participant demo. General
cursor preferences, reassociation, independent group controllers, rendering demand/
masked capture and coordination acquisition remain queued. No new owner decision
blocks that work.


Implementation update 2026-10-24: **installed independent input participant**, over
`24c60002`. F5 -> user -> Split -> **AUTO** now admits a separate
[participant program](source-rs/distribution/applications/input-demo/main.rs).
It selects the right Notes document, types `bot`, then moves a yellow cursor.
Physical input can select/edit the left document independently. **STOP** retires
its grant, process and cursor; AUTO starts a fresh run. F1 saves either note.
This uses the installed default composition, not a disposable probe variant.

Split owns [admission, lifetime and endpoint dispatch](source-rs/distribution/applications/split/session/producer/producer.rs).
It validates the sealed service declaration, negotiates inherited input capacity,
reserves two channels and a private focus group, and allocates one cursor surface
from its existing range. The producer has no storage, framebuffer, global input
or drawing grant. Its broker cannot inject; its producer pair cannot acquire
services or perform other interaction operations. Early requests/attachments do
not fill a reply queue whose receiver has not transferred. Existing matrix policy
validates generation, permitted keys and owner-local geometry. Ordinary Notes
and other stock app offers are unchanged; kernel/platform authority is unchanged.

STOP, endpoint loss and view invalidation terminate the run. A stale upstream
cursor publication retires the participant rather than killing its Notes siblings.
Capacity widens only as unread channel/group retirements permit; cached service
acquisition never silently recreates a grant. The same matrix can explicitly
admit another participant after retirement. Split's existing fifteen-channel,
three-descendant budget covers two Notes and this third process. Its more tightly
funded DEEPER matrix cannot also run AUTO. This is a fixed distribution demo,
not a general producer registry, cursor preference service or automation API.

Verification: workspace host tests, userspace clippy, layout (40 Omega / 128 Rust
edges), format, Python syntax, documentation links and whitespace. The participants
workflow checks exact document text, independent selection, confined yellow pixels,
STOP, repeated admission within one matrix, resize revocation with usable siblings,
saves and cold reopen. Traced proof:
`build/boot-harness-rs/desktop/20261024T162105813807Z`.
The final untraced default build passes the same fresh/cold workflow:
`build/boot-harness-rs/desktop/20261024T162344973198Z`. Split is 154,416 bytes;
the participant is 32,952 bytes; the ordinary stock matrix remains 250,512 bytes
(254,608 with storage tracing), within the 256 KiB stored-object cap.
Existing Split/DEEPER regression also passes dirty close, resize, simultaneous
ordinary apps, same-code recursion, capacity refusal, repeated collection and cold
reopen: `build/boot-harness-rs/desktop/20261024T162204726167Z`.
An earlier expanded run passed live resize but its cold text comparison assumed
one line; the screenshot showed the correct wrapped text. The helper now handles
15-column wrapping and checks a following blank glyph. No runtime change was
needed for that failure. No new ARM64, GPU or broad scheduler/driver result is claimed.

Next: exercise producer interruption with held input and queue pressure through
this installed workflow, then add scoped channel description/discovery so apps
can distinguish supplied channels without exposing ancestor registries or origins.
General cursor assets, independent group controllers and reassociation remain
outstanding. Rendering demand/masked capture and coordination acquisition remain
queued. No new owner decision blocks this work.


Implementation update 2026-10-24: **scoped cursor presentation**, over `1d396617`.
The [owned-surface path](source-rs/platform/libraries/matrix/world/surface/README.md#bounded-cursor-presentation)
now carries a fixed 7-by-12 arrow with transparent empty pixels. SURFACE_CURSOR
uses an existing caller-local surface and its conserving allowance. Each matrix
validates/clips/translates it normally; the root compositor retains a small
primitive, with no full-screen cursor buffer or bitmap allocation. Ancestor
stacking and visibility still apply. Same-extent movement replays the cursor;
revocation/disconnect retires it. Staged publication includes cursor updates.
This is an experimental bounded shape, not an authenticated participant badge,
input grant, global overlay or complete cursor-assets/suppression API.

Stock and Split surface dispatch forward this primitive. The integrated desktop
fixture verifies real pixels through two matrix hops: transparency, unpublished
staging, commit, retained movement and right-edge clipping. Host proofs cover
recursive clipping, invalid/stale requests, visibility, retirement, damage and
occlusion. Default F5 still uses its physical cursor; no synthetic app is installed
yet. Input producer acquisition remains the preceding verified baseline.

Next: install a launchable multi-participant matrix in the normal F5 catalogue,
with ordinary app children and an explicitly admitted producer child. Its matrix
can grant production locally and fund a cursor surface from its own allowance,
without granting injection to every desktop app or borrowing the root's cursor.
Demonstrate independent physical/synthetic selection and nested containment. Keep
view invalidation/revocation explicit; do not silently renew a cached producer
grant. General target-dependent cursor preferences, independent group controllers,
reassociation, descriptor discovery and retirement backpressure remain outstanding.
Rendering demand/masked capture and coordination acquisition remain queued.
No new owner decision blocks this work.

Verification: workspace host tests (81 matrix / 22 display-library tests),
userspace clippy, host matrix/display library+test clippy, layout, format and
Python syntax. `build/boot-harness-rs/matrix-desktop-test` passes cursor pixel
checks, the existing admission/storage/presentation/provider-recovery proof,
physical input and cold persisted colors. Its first run exposed a stale recovery
fixture assertion that input-source loss erases focus. Accepted cancellation
semantics preserve route selection; the corrected fixture verifies that a fresh
explicitly authorized owner press reaches that remembered target and leaves the
sibling unchanged. The host counts subsequent physical-input events relative to
startup evidence, excluding intentional recovery injections. No production routing
semantics changed to satisfy that fixture. A host --all-targets clippy invocation
incorrectly tried to build the guest-only display binary for Windows; library/test
clippy and guest-target checks are the applicable checks.
The normal four-level desktop also passes repeated subtree cleanup, protected
close, isolated saved data and cold reopen:
`build/boot-harness-rs/desktop/20261024T160554291577Z`. The earlier attempted
regression left only its generated profile and no completed result; it is not
counted as evidence. The traced stock matrix remains 254,608 bytes (97% of the
256 KiB stored-object cap). No new ARM64, pressure-variant or GPU result is claimed.


Implementation update 2026-10-24: **explicit producer service acquisition**, over
`ab0d76f8`. The [service contract](source-rs/platform/contracts/services/README.md)
now recognizes INPUT_PRODUCER only when the sealed manifest declares it and its
owner explicitly offers it. Ordinary `imports::ALL` remains interaction/storage;
SUPPORTED identifies recognized contracts without granting them. Legacy positional
startup cannot supply production. No default installed app gains the new service.

The stock broker preallocates a separate producer pair at admission and attempts
its bounded grant at first acquisition: two channels, one private focus group and
click-to-focus permission. Nested hosts negotiate inherited capacity first.
The [receiver](source-rs/distribution/matrix/service/children/producer.rs) checks
endpoint incarnation and accepts only producer operations. That child's ordinary
interaction pair cannot substitute for it. Both halves must leave broker custody
before production; early traffic is discarded without filling the retained reply
queue. Disconnect retires the grant without stopping the app; teardown collects
untransferred rights. Complete acquisition refusals leave the shared broker usable;
ambiguous transport failures still close it to prevent misattributing late replies.

This is not automatic grant renewal: view invalidation, release and owner revoke
end production even if the cached endpoint remains. The broker does not recreate
pairs or silently regrant. Existing retirement backpressure remains. Independent
group controllers, reassociation, descriptor discovery and visible synthetic
cursor/focus integration remain outstanding. Next: connect an explicit producer
app to the ordinary desktop, handle grant/view lifetime, show independent control
through nested matrices and measure latency/stalls. Rendering demand/masked capture
and coordination acquisition follow. No new owner decision blocks that work.

Verification: workspace host tests (80 matrix / 17 platform-contract tests),
userspace clippy, host matrix/contracts clippy, producer probe/partial feature
clippy, layout (40 Omega / 125 Rust edges), format, Python syntax and whitespace.
Disposable desktop acquisition variants preserve normal editing, repeated
close/relaunch and cold persistence:

- Complete connection, denied unrelated operations, closure with app survival:
  `build/boot-harness-rs/desktop/20261024T152129374950Z`.
- Interrupted handoff, early traffic, concurrent capacity refusal with usable
  broker, and replacement: `build/boot-harness-rs/desktop/20261024T152437127786Z`.
- Manifest requests production but owner withholds it:
  `build/boot-harness-rs/desktop/20261024T152607943161Z`.
- Independent admission/acquisition, rollback and subtree reclamation:
  `build/boot-harness-rs/matrix-composition/20261024T152753195453Z`.
- Recursive matrix IPC, producer grants, capacity and locality:
  `build/boot-harness-rs/matrix-test`.
- Four nested default desktops, protected close, isolated saved data, repeated
  subtree reclamation and cold reopen (no producer probe features):
  `build/boot-harness-rs/desktop/20261024T152918865723Z`.

The first partial run exposed a fixture assumption that refusal must be NO_MEMORY;
retained bindings can also legitimately return BUSY. The corrected fixture checks
both without weakening its broker-survival and repeated-reclamation assertions.
The matrix ELF is 250,512 bytes, or 254,608 with storage tracing (95%/97% of the
256 KiB stored-object cap). No visible
second cursor, general producer registry, GPU/remote or ARM64 result is claimed.


Implementation update 2026-10-23: **negotiated recursive input capacity**, over
`d76d0808`. The child-only `INPUT_CAPACITY` operation sets channel/group ceilings
for its current connection. It validates the view epoch and refuses reductions
below retained aliases, including unread retirement boundaries. Empty event
queues do not free channel identities. Limits survive view changes on the same
connection; disconnect resets them. Delivery preflight refuses extra names before
mutating focus, capture or held-key state.

The [stock parent adapter](source-rs/distribution/matrix/service/parent/input.rs)
now negotiates an incoming ceiling before granting local producer capacity.
[Headroom](source-rs/platform/libraries/matrix/world/input/capacity.rs) includes
retiring local allocations. Once those retire, the service widens upstream
capacity automatically. This supersedes the prior blanket refusal of nested
local producers. Reductions can still return BUSY while previously admitted
upstream names remain live. No existing binding is silently evicted to fit one.

Explicit producer submissions receive a capacity refusal. An already-consumed
hardware or upstream sample instead interrupts only its stream and suppresses
residual holds, preserving the adapter and parent grant. The physical adapter
must keep servicing input even if a child has opted out of new bindings. This
is not end-to-end input acknowledgement or guaranteed delivery under pressure.
The existing retirement backpressure limitation remains.

The [IPC scenario](source-rs/distribution/desktop/scenario/input_capacity.rs)
admits a nested local producer alongside inherited input, refuses excess parent
input without breaking the view, drains retirement, then verifies restored
upstream capacity in the subsequent two-channel producer test. The first run
exposed a fixture assumption about a hardware channel surviving earlier subtree
replacement; the test now explicitly establishes that channel before negotiation.
Seven host regressions cover retained names, group versus channel limits,
connection/view lifetime, refusal atomicity, inherited loss and hardware sampling.

Next: explicitly opted-in manifest/broker acquisition using these producer grants,
then a visible synthetic cursor and independent focus in ordinary F5. Do not add
production implicitly through `imports::ALL` or lend owner endpoints. The stock
producer grant still uses the existing authenticated child interaction connection;
independently acquired endpoints and their custody remain to be implemented.
Independent group controllers, reassociation and descriptors remain outstanding.
Rendering demand/masked capture and coordination acquisition remain queued.
No owner decision blocks this work. The latest desktop matrix ELF is 249,832
bytes (95% of the 256 KiB stored-object cap); account for that remaining headroom.

Verification: full workspace host tests (80 matrix tests), matrix/contracts host
clippy, userspace clippy, layout (40 Omega / 125 Rust edges), format and whitespace
checks. Final `build/boot-harness-rs/matrix-test` passes the nested capacity,
producer lifecycle and existing recursive confinement proof. Four nested desktops,
protected close, saved inner data, repeated subtree reclamation/isolation and
cold reopen pass at `build/boot-harness-rs/desktop/20261024T063955617025Z`
with TCG absolute input. Earlier
recovery/management, independent composition, GPU/remote and ARM64 evidence is
unchanged and is not a new result for this slice.

Implementation update 2026-10-23: **explicit child producer grants**, over
`7d170864`. The [producer entrance](source-rs/platform/libraries/matrix/world/input/producer.rs)
reserves a channel quota and private focus group for an already authenticated
child connection. Grant-local ordinals never select another producer's channels;
per-child generations reject stale submissions without exposing sibling grant
counters. Click-to-focus is a separate right from pointer delivery. Only an
explicitly granted producer can query its authorized input extent. Ordinary
interaction endpoints still deny all owner registration/control operations.

Release, owner revoke, connection loss, changed child view or changed upstream
grant ends production and retires held input. Retirement storage remains charged
until recipients drain it; the group then retires automatically. This is a
delegation mechanism over the existing authenticated child connection, not yet
manifest-based service acquisition or a separately acquired producer endpoint.
No import was added to `imports::ALL`, and no F5 app gains production implicitly.

The stock root reserves fixed keyboard/pointer channels at BIND, before dynamic
admission. The [IPC fixture](source-rs/distribution/desktop/scenario/producers.rs)
grants an ordinary sibling two producer channels, routes its pointer/key through
another matrix, rejects wrong/excess/stale grants and verifies release and process
death retirement. A host proof separately tests delivery without focus permission.

**Next capacity gap:** a nested matrix may legally receive all four upstream
channels. Assuming only two will arrive overpromises space if another producer
appears above it. Nested stock hosts therefore refuse new local producer grants
until the protocol negotiates an upstream channel/group ceiling. Root producers
still forward recursively. Implement that allowance exchange and its retirement
rules before enabling local producer acquisition at arbitrary depths; do not
paper over it by treating physical input as the only possible ancestor input.
The shared library accepts owner-selected reserved inherited capacity, but does
not negotiate that agreement. Its caller must reserve the actual agreed maximum.

Then connect manifest-based producer acquisition, show physical/synthetic groups
with separate cursors and focus in F5, and test overflow/stalls/latency. The
existing retirement backpressure limitation remains. General independent group
controllers, live reassociation and descriptor discovery remain outstanding.
Rendering demand/masked capture and coordination acquisition remain queued.
No new owner decision blocks this engineering work.

Verification: full workspace host tests (73 matrix tests), matrix/contracts host
clippy, userspace clippy, format, layout (40 Omega / 125 Rust edges) and whitespace
checks. `build/boot-harness-rs/matrix-test` covers actual producer IPC and the
existing recursive confinement/lifetime proof. Nested Split with focus, resize,
dirty-close, persistence and cold reboot passed at
`build/boot-harness-rs/desktop/20261024T062631594481Z`. That desktop run preceded
the final explicit nested-producer refusal; it does not request producer grants.
The stored desktop matrix ELF in that run is 245,640 bytes (about 94% of 256 KiB).
No new four-level desktop, recovery/management, GPU/remote or ARM64 result is
implied; earlier evidence remains under its recorded baseline.

Implementation update 2026-10-23: **input registration and explicit retirement**,
over `d6af9dc8`. The [registration entrance](source-rs/platform/libraries/matrix/world/input/registration.rs)
allocates owner-controlled channels and focus groups. Local and inherited names
have separate namespaces. Retiring a channel discards its pending events, clears
holds and queues an explicit boundary for every recipient. Its slot is reusable
only after those boundaries drain or the recipients lose their bindings.
Groups have a separate retirement boundary: replacing the last channel preserves
the group's remembered selection. Child aliases increase within a view binding;
exhaustion refuses instead of wrapping. App reception frees retired pointer state.

The [recursive IPC fixture](source-rs/distribution/desktop/scenario/registration.rs)
churns channels and groups beyond table capacity without resetting geometry,
checks local/inherited name collisions, and rejects stale submissions. The seven
owner-only operations remain denied on ordinary child interaction endpoints.
This does not yet provide app producer acquisition, admission quotas, delegated
group control or disconnect cleanup for producer grants. Never lend an owner
endpoint as a shortcut to those permissions.

Stock and Split routers let descendant retirement notifications drain before
polling another upstream snapshot. Local requests continue. This bounds tables
but can delay unrelated upstream input behind a slow descendant; it is an explicit
lab limitation, not a latency or denial-of-service isolation guarantee. Earlier
entries saying registration/retirement are entirely absent are superseded.

Next: add explicitly opted-in producer acquisition with endpoint/incarnation and
view-generation checks, quotas, disconnect retirement and separate group-control
rights. Reserve fixed source/inherited capacity before dynamic admission; F5's
current adapter still lazily binds local channels 1/2 in group 1. Do not extend
`imports::ALL` into an implicit grant to every app. Then show physical and synthetic
groups, separate cursors/focus and nested forwarding in ordinary F5, measuring
latency and testing stalls/overflow. Rendering demand/masked capture and
coordination acquisition remain queued. No owner answer blocks this work.

Verification: full workspace host tests (68 matrix tests), matrix/contracts host
clippy, `cargo check-userspace`, stock matrix trace/delay feature clippy, layout
(40 Omega / 125 Rust package edges), format and whitespace checks. QEMU evidence:

- `build/boot-harness-rs/matrix-test`: final recursive IPC run includes eight
  repeated channel replacements and eight distinct group lifetimes, namespace
  isolation and denied child injection, followed by the existing confinement proof.
- `build/boot-harness-rs/desktop/20261024T061011356969Z`: four nested desktops with
  TCG absolute input, protected close, saved inner data, repeated subtree
  reclamation/isolation and cold reopening.
- `build/boot-harness-rs/desktop/20261024T061213681896Z`: Split within Split,
  independent focus, resize, dirty close, funded refusal and cold persistence.
- `build/boot-harness-rs/matrix-composition/20261024T061629926144Z`: independent
  three-peer acquisition, interrupted handoffs, rollback and subtree reclamation.

The desktop stock matrix ELF is 245,280 bytes, about 94% of the current 256 KiB
stored-object limit. Account for that headroom before adding the producer service.
No fresh GPU/remote, ARM64 or latency guarantee is implied; recovery/settings and
management evidence below belongs to the preceding baseline.

Implementation update 2026-10-23: **channels sharing focus groups**, over
`418ef6ad`. The [input entrance](source-rs/platform/libraries/matrix/world/input/input.rs)
now separates bounded channel state from group selection. F5's authenticated
[source adapter](source-rs/distribution/matrix/service/requests/source.rs) supplies
keyboard and pointer as separate channels in one group. Child-local aliases
preserve association through nested matrices without exposing ancestor names.
Events within a group remain ordered across channels; independent groups drain
fairly. Motion coalescing cannot cross another channel's event in that group.
Keyboard-only cancellation preserves pointer capture, with one reserved queue
slot so a focus change cannot discard a full queue of pointer edges.

[App reception](source-rs/platform/libraries/matrix/reception.rs) tracks pointer
edges per channel and view binding. Chooser controls, Notes, Colors, Split and
recovery use it. This isolates holds, not documents/dialogs: application state
remains intentionally shared unless that application chooses otherwise. Nine
new host tests cover reception, shared selection, independent captures, nested
event ordering, inherited keyboard cancellation and bounded group association.

The recovery fixture exposed an existing startup race: a user matrix can
configure itself before the outer desktop supplies LAUNCHER_FALLBACK. That first
supply used to discard its already-armed recovery. Initial supply now preserves
a running launcher's remaining retry allowance; replacement/revocation and
supply during pending recovery still invalidate the saved context. CONFIGURE
on an already configured desktop remains idempotent, without rearming retries.

This remains a partial service implementation. Four channels/groups and sixteen
ordinary events per channel (plus the keyboard-cancel boundary) are lab bounds.
General producer registration/retirement, control grants, rebinding, descriptor
discovery and a visible synthetic producer are not implemented. NEXT word seven
and POLL tag bits 17..31 now carry the group; rebuild programs together. Earlier
entries saying one stream equals one route or that F5 supplies one combined
stream are superseded.

Next: explicitly granted producer registration and lifetime, including namespacing
upstream versus local sources and safe retirement with unread events. Do not
silently grant production to every app through `imports::ALL`. Then demonstrate
physical and synthetic groups, separate cursors and focus, and nested forwarding
in ordinary F5. Rendering demand/masked capture follow; coordination acquisition
remains queued. No owner answer blocks this work.

Verification: full workspace host tests (59 matrix tests), matrix/contracts host
clippy, `cargo check-userspace`, stock matrix trace/delay feature clippy, layout
(40 Omega / 125 Rust package edges), format and whitespace checks. Documentation
relative targets checked. QEMU artifacts under `build/boot-harness-rs/`:

- `desktop/20261024T052905822760Z`: ordinary F5 apps, draft isolation, close choices,
  persistence and cold reboot.
- `desktop/20261024T052548991673Z`: four nested desktops with TCG absolute input,
  saved inner data, repeated teardown/isolation and cold reopening.
- `desktop/20261024T052753935286Z`: alternative Split within Split, focus, resize,
  dirty-close and funded refusal, with cold persistence.
- `launcher-recovery/20261024T054041211303Z`: all seven modes, including automatic
  and explicit retry, startup failure, clean exit, owner stop/revoke, fallback
  failure/exit, retained dirty Notes and cold persistence. Input work initially
  failed this test; the startup race above was diagnosed and fixed before this run.
- `settings/20261024T054516922878Z`: Cancel, live/saved arrangement, running apps,
  three boots and independent new-user defaults.
- `management/20261024T054557452216Z`: hung app termination, both Cancel choices,
  stale/ungranted authority, launcher recovery, replacement apps and cold persistence.
- `matrix-test`: final `--matrix-test --release --timeout 90` passed recursive
  input, confinement, revoke/resize and subtree replacement over actual IPC.

The first three desktop runs precede the subsequent fallback-policy correction;
the recovery/settings/management fixtures include it. No producer-registration,
GPU, remote-rendering or performance guarantee is implied. The stock matrix ELF
is 236,816 bytes, 90% of the current 256 KiB stored-object limit. The independent
three-peer composition passed scoped acquisition, interrupted handoffs, rollback
and subtree reclamation at
`matrix-composition/20261024T054757404596Z`.

Implementation update 2026-10-23: **independent input routing foundation**, over
`1f4c81e5`. The [matrix input entrance](source-rs/platform/libraries/matrix/world/input/input.rs)
now owns four bounded upstream streams with separate focus, capture, held keys
and cancellation. Each child has sixteen queued events per stream and fair drain;
its compact names are local to that view binding. NEXT word six and POLL word
five's high 32 bits carry the name. Rebuild current programs together; this is an
experimental wire change, not a production ABI or saved-program compatibility promise.

Overflow cancels only that stream across its recipients, including a hovered
child different from its keyboard target. Cancellation remembers local selection.
Inherited cancellation clears held state because the sender suppresses the old
release; waiting again would lose the next fresh press. Revoking capture also
clears the old button state while suppressing the residual hold. Nine new host
tests exercise these boundaries, independent typing, fair drain, source limits,
denied child injection, child-local names and nested forwarding.

This is not the complete focus-group/channel service. One stream currently owns
one route state. There is no producer registration, independent channel/group
association, directory, dynamic stream retirement or multi-cursor desktop yet.
F5 still supplies one physical stream. All stream names must fit the four-name
binding limit; a malformed/over-capacity inherited packet invalidates that parent
grant. Future registration must refuse admission before forwarding such traffic.

Current direction: finish two independent input groups in the ordinary F5 desktop,
with endpoint-derived producer authority, scoped registration/retirement, separate
focus-group association and app consumers that track button state per channel.
Then add a visible synthetic producer alongside physical input, recurse into an
installed nested matrix, and exercise isolation, disconnect and overflow there.
Do not introduce a human/agent default class or equate the final channel identity
with its group. Follow with rendering-demand/masked-capture evidence. Coordination
acquisition below remains queued; no new owner decision blocks this input work.

Verification: workspace host tests, 50 matrix host tests, matrix/contracts
host clippy, `cargo check-userspace`, stock matrix trace/delay feature clippy,
layout/format/whitespace checks. The ordinary F5 desktop test passed TCG absolute
input and cold persistence (`build/boot-harness-rs/desktop/20261024T050904504038Z`).
Four nested desktops, saved inner data, repeated teardown/isolation and cold reopen
passed WHPX with explicit relative input
(`build/boot-harness-rs/desktop/20261024T050640988167Z`). These desktop runs precede
the final capture-release bookkeeping fix, which has its own host regression.
The final `--matrix-test --release --timeout 90` passes recursive input, revoke,
resize and replacement through real IPC (`build/boot-harness-rs/matrix-test`).
The independent three-peer matrix passes scoped acquisition, interrupted handoff,
rollback and subtree reclamation
(`build/boot-harness-rs/matrix-composition/20261024T051206394187Z`).
WHPX without `--relative-pointer` failed the harness's absolute-transport assertion:
no absolute-pointer-ready marker was emitted. Do not claim WHPX absolute support.
The blanket host `clippy --all-targets` command is unsuitable for guest binaries;
use the target-specific alias and scoped host checks. Minor formatting and host
test lint corrections accompany this slice.

Specification sweep 2026-10-23: the current [presentation contract](wiki/spec/human_surface/rendering.md)
uses coherent accepted-layout hit-testing without exact displayed-frame matching
or a one-frame lag guarantee. Capture explicitly serves authorized, already-masked
output. Cursor preference follows per-pointer hover/capture targets recursively,
including no-cursor requests; hiding grants neither capture nor pointer lock.
The new [resource contract](wiki/spec/human_surface/rendering_resources.md) separates
image authority/version from backing location: local shared import, conversion
and remote upload are supported routes. Producer death retires live registrations;
physical reclamation waits for actual users. Remote silence is not confirmed death,
and copied resources follow destination/session lifetime and authority.

Older entries below that leave all focus, cursor handoff, scene/input selection
or capture-mask boundaries open are superseded by these partial specifications.
Exact drawing vocabulary, precise scene/mask and IPC publication, and shared-cache
sponsorship/residency remain open. Later accepted drawing/resource rules above
supersede the old platform-text and miss-appearance questions; protected owner UI
is separate from cursor provenance.
Documentation only: relative-link/anchor and whitespace checks; no runtime changes,
GPU/remote implementation, performance measurements or new QEMU verification.
Follow-up evidence should test consistent local targeting during layout changes,
scoped cursor suppression/restoration, masked capture without original-buffer access,
producer death with retained readers, destination capacity and remote disconnect.

Owner update 2026-10-23: repeated launches of the same program over one realm
are accepted; concurrency or refusal is the app's policy. See the
[accepted contract](wiki/spec/execution/components_and_matrices.md#multiple-instances-over-one-realm).
The question is removed from OWNER_QUESTIONS.md; older entries calling it blocked
are superseded. The owner has now ratified the
[coordination and checked shared-memory IPC operation](wiki/spec/communication/ipc.md)
for the shipped OS/default matrix: scoped registration and introductions,
independent executable/path/secret constraints, app-controlled refusal and
protocol-enforced payload custody. The wire protocol and implementations remain
outstanding. This supersedes the earlier standalone named-lock proposal.

Owner clarification 2026-10-23: coordination controls introductions, not ongoing
authority over established peers. Services enforce ongoing permission/cancellation
in their own protocols; the first participant gains no implicit power to revoke
peer mappings or independent rights. See the
[accepted boundary](wiki/spec/communication/ipc.md#established-connections-and-service-authority).
Connection closure must preserve live memory-view obligations. The 2026-10-24
[notification contract](wiki/spec/execution/notifications.md) now settles kernel
wait outcomes; library protocols own message publication, cancellation and grant
acceptance. Follow the [implementation outcome](#shared-memory-ipc-and-notification-outcome),
not the superseded copied-transport owner question.

Owner clarification 2026-10-23: a failed or terminated matrix takes down its
entire owned subtree. No surviving descendant adoption; restart admits new
instances and may reopen authorized durable state. Independently owned external
providers are outside that subtree. See the
[matrix lifetime contract](wiki/spec/execution/components_and_matrices.md#matrix-failure-and-subtree-lifetime).
The Rust lab already cancels owned children on supervisor exit/fault; this
documentation ratification does not add runtime behavior or fresh QEMU evidence.
Budget refund routing and external-operation lifetimes remain separate contracts.

Owner clarification 2026-10-23: execution spends the executing program's budget
within its ancestor ceilings, including when it serves another program. Services
and matrices have no special scheduling status; request quotas and backpressure
belong to their own policies. The
[execution-budget contract](wiki/spec/resources/execution_budgets.md) closes CPU
sponsorship as an owner blocker. Caller-funded execution is deferred pending a
concrete need, not a required coordination milestone. Historical entries calling
that baseline unresolved are superseded; other resource sponsorship is unchanged.
This is a documentation ratification of the current lab direction, not a new
runtime implementation or fresh scheduling verification.

Owner clarification 2026-10-23: [hardware-separated programs](wiki/spec/execution/isolation.md)
are the current isolation architecture for apps, matrices and platform providers.
PCC-based single-address-space isolation is off the roadmap, not a future proof
threshold to implement. Reconsideration requires CHERI support and a separate
contract. Explicit shared IPC mappings remain supported. The old default-isolation
question is closed; optional PCID validation coverage remains separate and deferred.
This documentation decision introduces no runtime changes or fresh QEMU evidence.

Owner clarification 2026-10-23: default chrome/background programs come from the
system realm and everyday selections are replaceable. The outer system retains
protected recovery chrome: a physical OS-key long press brings it back even when
custom chrome or its matrix fails. Inner matrices need not include this escape
hatch; they may receive a system-realm copy and choose their own overrides.
See the [accepted surface contract](wiki/spec/human_surface/compositor_and_seat.md).
Override slot format and recovery implementation are not frozen. The Rust lab
does not yet implement the protected long-press path; current child close and
launcher restart are not that guarantee. No runtime behavior changed here.

Owner ratification 2026-10-23: the [input foundation](wiki/spec/human_surface/input.md)
now specifies scoped buffered delivery, explicit loss/resynchronization, distinct
physical/symbol/text/action representations, accepted interaction targets and
matrix-controlled mappings. Latest interaction descriptions do not replace event
history. Raw canvas/controller subscriptions remain valid; rich semantics are a
separate quality claim. No per-frame advertisement or universal revision counter
is required. Focus topology, views, rendering and scene/input commits remain open.
The Rust key ABI and sixteen-event matrix queues are partial evidence only; no
production wire encoding, text stack or shared input ring has landed here.

Owner ratification 2026-10-23: [logical input channels](wiki/spec/human_surface/input_channels.md)
now identify independently routed streams, with no special human/agent/default
classes. Apps can publish one acceptance interface and assign incoming channels
to their own roles. Stable identities are distinct from compact event references;
scoped channel discovery carries metadata outside the event hot path. An explicitly
mapped read-only directory beside a multiplexed ring is a supported direction,
not an implemented transport. Source continuity is explicit owner policy; port
changes or matching names do not confer control. Focus/views/rendering remain open.
No runtime changes or new runtime verification accompany this documentation update.

Owner clarification 2026-10-23: producers register channels with their supplied
input compositor/router under explicit authority. Receiving apps describe accepted
input but do not publish received channels. The service owns registry mutation and
may expose scoped read-only shared views; unrelated registry rows must not leak
through those mappings. Nested input compositors can maintain their own registries
and driver/agent/forwarded sources. Hardware access remains a separate grant; no
ambient lookup bypasses a matrix to reach a "real" outer compositor. See the
[registration boundary](wiki/spec/human_surface/input_channels.md#producer-registration-and-registry-ownership).

Input follow-up: design the stream/snapshot boundary and exercise press/release
during a stalled consumer, overflow/resynchronization, disconnect while held,
remapping while held, multiple source contributions and nested forwarding. Include
two identical controllers, directory enumeration racing a change, source rebinding,
retirement with unread events, safe compact-reference reuse and slow metadata readers.
Keep timing and scene/focus questions separate rather than silently choosing them
in an input transport patch. This ratification changed docs/comments only and
added no runtime verification evidence.

Owner direction 2026-10-23: retain the current recursive input forwarding chain
for now. Revisit transport if measured latency warrants it, separating router
scheduling and request/reply costs from event copying and per-hop bookkeeping.
Bounded batches, shared queues and explicitly authorized direct delegation are
candidates, not selected changes. Shared memory does not remove a filtering
router's need to execute or permit exposing unrelated ancestor input. No runtime
changes or new performance measurements accompany this direction.

Documentation audit 2026-10-23: the accepted IPC contract now makes whole-page
partitioning within one allocation explicit, including alias/padding authority
and the separate publication-stability obligation. The rendering proposal records
page-aligned leases versus arbitrary rectangles and reference forwarding as
alternatives, not a selected surface ABI. Input forwarding remains unchanged;
focus/views, scene commits, masks and production rendering choices remain open.
Documentation checks only; no new runtime or performance evidence.

Owner ratification 2026-10-23: [focus groups and routes](wiki/spec/human_surface/focus_routes.md)
now specify one controlling owner per registered group, authorized channel
association, recursive local selection and separate pointer capture/presentation.
An ancestor stopping forwarding does not clear a nested matrix's selection;
interrupted streams still require scoped cancellation and held-input cleanup.
Select-only versus select-and-deliver is local policy. Router-owned events are
cancelled on interruption; published IPC events are not assumed retractable.
The old topology and queued-focus questions are superseded; cursor presentation
handoff remains open in OWNER_QUESTIONS.md. IPC wait closure is now specified;
application commit/cancellation belongs to each protocol.
Follow-up evidence should cover independent groups, denied cross-group control,
click recursion, held-input switches, capture independence, stopped upstream
forwarding with independent local input, and safe route retirement. This is a
documentation acceptance: no general multi-group implementation or new runtime
testing is claimed; the current forwarding chain remains unchanged.

Owner ratification 2026-10-23: [rendering and presentation](wiki/spec/human_surface/rendering.md)
now separates owner-managed placement, rendering demand, input selection and
execution. Minimize/restore are matrix UX; virtual placement and bounded capture
need not relocate shared presentations. Fullscreen propagates only through
agreeing owners, with scoped demand status rather than a global fullscreen PID.
Cursor surfaces require bounded presentation authority, not whole-output writes.
Image references and completion dependencies must permit avoiding mandatory
ancestor images/CPU waits; no GPU, direct-scanout or zero-copy guarantee is claimed.
The channel spec also records authorized source merging/switching into one virtual
channel without forged physical provenance or independent app state. Follow-up
experiments should cover scoped demand, capture without placement changes, local
versus propagated fullscreen, independent input and placement, and backing reuse
only after consumers finish. Drawing/text, scene commits, masks, resource policy
and cursor visual handoff remain open. Docs only; existing runtime is unchanged.

Queued coordination slice after the current input milestone: add funded coordination acquisition to the existing
broker and prove simultaneous first registration, rejected identity spoofing,
constraint combinations, accepted/refused introductions and dead-holder cleanup.
Exercise duplicate Nested launches through F5 while unrelated apps keep running.
Reuse the existing grant-transfer and custody checks while moving transport to
the [shared-memory and notification outcome](#shared-memory-ipc-and-notification-outcome).
Reusable mailbox access needs its checked/admitted provider and lifecycle before
exposing stable leases. Present copied channels and sealed attachments are not
that complete implementation. Path rename semantics remain separate; notification
closure and actual-enforcement revocation are no longer owner blockers.

Updated 2026-10-22 (local date). Current milestone: **the hop on hardware, and
global shared leaves**, on baseline `f8bab679` (where a forwarded request
spends its time). The harness runs the guest under the Windows hypervisor with
`--accel whpx` as well as under TCG: 32 bare round trips between two tasks cost
118 to 144 Ãƒâ€šÃ‚Âµs there against 3,140 to 4,824 under TCG, and the hop between
matrices sits inside the trace line's own noise. The seven supervisor ranges
every root shares are global now, so the kernel's translations survive the two
root switches of a trap: 15 percent off the round trip on hardware, nothing
under TCG, whose emulator flushes everything on every root switch. The newest
section at the end records the measurements and the open items.
Saved-state testing is an explicit launch choice. The section before it
distinguishes current-program four-level verification from failures in
retained executable snapshots.
The fresh-lab reset is `30572c9f`. Compact recursive desktops landed as `c7571191`,
and the launch request ABI repair as `a287ae39`. No recursive legacy snapshot
upgrade is required. The saved-disk quota/headroom repair remains in place. Current
verification and limitations are recorded in the newest section at the end.
Previous baseline: atomic named creation and deeper recursive matrices at `102e416c`.
The saved-lab launch stages changed program installations in host scratch space, then verifies and
publishes a result that fits the assigned disk extent. Legacy 4 MiB desktop
volumes are backed up and grown offline to 8 MiB without changing their records;
the root's logical allowance is now 2 MiB. Final quota admission follows all
stock and named replacements, with reachability collection under pressure.
Current data and pins survive.
See the latest evidence at the end of this handoff. Earlier baseline: declared service
imports and optional Notes persistence at **`8d1972a`**, followed by readable
chooser labels at **`787344e`**, existing-user label upgrade at **`4f1c6c8`**,
app-owned close at **`a40ed23`**, user tiles/absolute input at **`4901e65`**,
the transient-picker handoff at **`30f169b`**, and the retained-disk Colors fix
below. Store-based admission validates the program's required/optional interfaces against owner
offers before spawning. Check Git for
later work.

## Start here

F5 boots one desktop to a chooser over the folder of users. **Preserve that as
the default experience.** From the repository root:

```powershell
python tools/boot-harness-rs/run.py --release --window --fresh
```

This is what the normal [VS Code launch entry](.vscode/launch.json) runs.
**Cathedral: Build & Run in QEMU** uses `--fresh`: it backs up the disk and receipt
under `build/boot-harness-rs/before-fresh-*` and installs current seeds on a fresh
disk on every launch. Recreate users there. **Cathedral: Resume Saved Lab in QEMU**
omits `--fresh` for explicit persistence testing. The owner explicitly does not require compatibility
or recursive migrations for old lab users; use a fresh lab when templates change.
The composition is [distribution/profile.json](source-rs/distribution/profile.json).
It uses x86-64 QEMU/TCG and UEFI.

- The full-screen user picker shows a horizontal row of avatar tiles. **NEW
  USER** copies the hidden template; click a user to enter its workspace. Up
  to three users fit the current bounded listing; the add tile then disappears.
  No scrolling, passwords or logout yet. Restart QEMU to select another user.
- The user's compact bottom taskbar shows **NOTES**, **COLORS**, **SPLIT** and **NESTED** icon tiles. The
  launcher itself and implementation controls are hidden. Click an app tile,
  then the app to focus it. Arrows and Enter work in a focused taskbar.
- **SPLIT** opens two persistent Notes side by side in a separate matrix process.
  Click either child to type. Its own top-right X asks both children about drafts;
  UNSAVED NOTES keeps it open. Save/discard drafts with the child apps' X dialogs.
  F1 saves to separate Left/Right realms; both survive relaunch and cold reboot.
  Outer Settings resizes the two columns while retaining both drafts.
  **DEEPER** checks for unsaved work, then enters another Split with independent
  saved notes. Reopen Split and click DEEPER to return there after a restart.
  Further nesting beyond its funding reports **NO ROOM**, retaining live drafts.
- **NESTED** opens its own user picker and the same desktop, with Notes, Colors
  and another Nested tile. Close other outer apps first. The current budget fits
  four nested desktops beneath the outer one, with Notes at the deepest level.
  Further launches remain bounded by conserved grants. Use Fresh Lab to discard
  stale template snapshots and create users with current code throughout.
  Inner saves survive closing/reopening Nested and reboot. The outer
  taskbar X refuses to close the subtree while an inner app has unsaved work.
- **SETTINGS** at the right of the taskbar (or F2 with taskbar focus) adjusts
  bar height and outer spacing. Save persists and applies the layout to running
  apps; Cancel discards local edits and returns to apps. Apps retain their state.
  SAVED; RESTART means persistence succeeded but live application was refused.
  Retry Save or restart to use the saved arrangement.
- Notes keeps up to eight notes of 240 ASCII characters. Letters, digits, spaces,
  Enter, Left/Right and Backspace edit. **F1** saves/reconciles, **F2** reloads,
  **F3** duplicates the saved note, **F4** switches notes. Colors changes and
  saves its color on **Enter**.
- Notes and Colors have an upper-right **X**. Clean apps exit; dirty Notes offers
  **SAVE / DISCARD / CANCEL**. A failed save stays open. Colors asks **CLOSE
  ANYWAY** if its last storage result is uncertain. Each app-tile click launches
  another instance (up to three, subject to funding). Buttons beside the icons switch
  instances; their X requests close and refuses unsaved work. App-owned X offers
  Save/Discard/Cancel. Multiple Notes instances share saved data but keep separate drafts.
- **END APP** above Settings handles an unresponsive app. It warns about unsaved
  work and defaults to Cancel; confirmation stops the exact observed app/subtree.
  F3 opens it with launcher focus. A recovered launcher can manage the existing
  app. This is delegated authority, not a protected system-wide escape key.
- After storage recovery is exhausted, **RETRY SAVES** appears above Settings.
  Click it (or F4 with strip focus) for one explicit attempt. It preserves live
  drafts and does not reset the automatic restart budget. F1 in Notes first
  reconciles an uncertain save; press again to save an unsaved reconciled draft.
- If the launcher exhausts its retry, **LAUNCHER UNAVAILABLE / TRY AGAIN** takes
  its place. A click or Enter with that screen focused requests one new attempt,
  preserving the app and current arrangement. Failure of that screen stays stopped.
- The default input service negotiates QEMU's absolute virtual mouse. The guest
  cursor follows the host pointer within the window; PS/2 remains the fallback.
  Apps still receive only local matrix input, never the physical device grant.
- The old STORAGE ON/OFF toggle and launcher CLOSE row are gone. Required/
  optional service binding and the matrix LOSS/CLOSE mechanism still exist;
  their engineering controls do not belong in the normal desktop.

Interactive state is stored in **`build/boot-harness-rs/cathedral-storage.raw`**:
the users you made, their notes and colors. Normal F5 sets this disk aside and
starts fresh; **Resume Saved Lab** reuses it. On that saved-lab path, missing media is created; existing
media is never automatically reformatted. The programs are installed into the
image on first launch and republished as new revisions when their bytes
change; the sidecar `cathedral-storage.raw.programs` records what is installed.
The desktop's explicit compatibility upgrade refreshes recognized stock
launchers, Notes/Colors and recognized stock matrix executables in existing
users, preserving their saved data and object identities. Changed installations
save a complete content-named `.before-compact-<digest>.raw` backup before work.
All migration stages run on a private host scratch disk, including reachability
collection under quota pressure. The final result must fit the assigned volume
after any explicit legacy growth and pass cold record comparison before it is
written to the locked disk.
A refused preparation leaves that disk and its installation receipt unchanged.
Existing older migration backups remain useful recovery points. Customized code,
live data, pinned revisions, metadata and saved layout retain their prior rules.
An image from before this layout holds other objects at the numbers the
programs need; the installer refuses to overwrite anything that is not a
program, so move that image and its `.programs` sidecar (if present) to backup
paths before relaunching to create a fresh desktop. The backup retains the old
data; it is not migrated into the new desktop. This refusal happens before QEMU
opens, so check the launch terminal if F5 appears to do nothing. `notes-storage.raw` and
`matrix-database-storage.raw` belong to the notes and probe desktop fixtures.
Automated tests use separate disposable images.

## The architecture, in the owner's words

Realms are the attenuated filesystem. Matrices are programs that attenuate,
heavily or not. If the boot matrix is correct, holds every capability, can
launch programs, and the filesystem is database-backed and attenuable, the rest
of the matrix falls out for almost free. The real logic is then that a launched
program gets its own subtree for free: everything nested under the program's
object, because files are folders in this database.

The current bounded shape boots a startup owner that runs the providers, root
matrix and user picker; a selected user matrix loads its installed launcher.
The chooser executable serves both picker and launcher as separate processes.
This demonstrates composition and attenuation, not authenticated login or the
full windower design.

## What the store milestones established

The thesis is that cheap file copies make cheap world copies. A root holds
numbered objects arranged as a tree; chunks and manifests are shared within the
root; a copy writes a head, an event and an edge; a subtree copy does that for
every object beneath the source. A matrix grants a child a subtree rather than
a root, stamps that scope on every request it forwards, and the provider
enforces every stamp, so a program sees its realm and nothing beside it.

Programs live in the store. The harness installs the profile's `programs` into
the notes root as objects, trimmed to their loadable bytes, nested as the
profile says, with `folder` entries as empty containers. A matrix runs one with
`RUN_OBJECT`, and now a child runs one with `LAUNCH`: the matrix reads the
object through the requester's own binding, so a program can only run what it
can see, admits the sealed reply as the sibling's code in the slot the owner
opened with a standing `LAUNCH_POLICY` and `LAUNCH_PLACE`, and grants it that
object's subtree. `CLOSE` asks the host to stop the sibling; the host asks the
sibling `LOSS` on its control link first and keeps it on any answer but
nothing. Each matrix remembers its own child storage grants; explicit revocation
and child death clear that policy. After provider replacement the owner restores
the direct user's realm and asks it to restore its own children.

A user is a subtree copy of the template user: a matrix executable with the
chrome, Notes, Colors, Split (with its own Notes code) and a saved `Matrix`
configuration inside it. The user
matrix loads its own launcher path, placement and budgets within the outer
owner's ceiling. `desktop_test.py` drives this from F5
and an independent host model verifies the whole layout; the headless storage
fixture runs a program from its realm, launches a sibling and closes it.

The kernel's contribution stays one syscall for code from sealed objects. It
now allows 48 reader rights per task and a reader page budget of up to twelve
times the per-object cap, because a matrix hub pins every image its subtree
shows, holds old and new during replacement, and lends the same budget to each
nested matrix. The kernel still knows no matrix, realm or store.

## Decisions and working expectations

- Cathedral is one monorepo: an independently usable **kernel**, a blessed common
  **platform** outside it, and one replaceable **distribution**. Desktop policy
  belongs to the distribution; capability enforcement belongs to its owning
  kernel/platform mechanism. Contracts live with the boundary owner.
- A matrix is an ordinary program. The instance handed provider links is the
  storage root and the one handed display/input endpoints adapts hardware;
  every other instance only forwards and attenuates. No authority policy branches on depth. Stock startup intent may select picker
  or saved-workspace initialization; nothing tunnels through a matrix to use its
  authority: its owner grants, its children request, the provider decides.
- Delegation narrows and never widens. A grant names a scope inside what the
  granting matrix holds; the provider, not the matrices, is what makes the chain
  binding. Hosting is copying: a matrix that hosts programs holds them in its
  own realm, and a child launches only what its own binding can read.
- Use the design's vocabulary. A realm is a subtree held through a binding;
  a world is a matrix's realm in the wiki's sense (the object a matrix owns,
  with its state beneath it), which the Rust OS does not yet implement. Do not
  coin desktop-level names for storage structure.
- Boot launches and storage-less fixtures keep the boot catalogue, because the
  provider itself must come from boot. The matrix's RUN by catalogue image
  serves those compositions; RUN_OBJECT and LAUNCH serve the store.
- Timeouts derive from one number. The matrix contract's `STORAGE_DEADLINE_TICKS`
  bounds a forwarded storage request; nested hops and program reads add a
  margin, the runtime's storage client waits twice it, and the desktop's
  provider watchdog probes past it, so a stalled provider is always reported by
  the matrix's verdict first. These numbers are **ticks**: the current timer
  runs nominally at 100 Hz, so the 1200-tick base deadline is **twelve
  seconds**, not 1.2 s. Keep units explicit and measure actual spans before
  tuning deadlines.
- Do not add debt to bridge old and new: when a protocol or layer is replaced,
  retire the old one from every composition in the same change and delete it.
  Git history is the only retention wanted.
- Discoverability matters: a named entry shows sequencing/dispatch and leads
  into semantic subfolders. Rust `mod.rs` is plumbing only, beside the substantive
  named entrance. Avoid forwarding-only layers and unrelated folder dumping grounds.
- Keep the OS responsive and understandable. Rendering/input changes need an
  actual QEMU interaction check, not just compilation or a serial success marker.
- Commit and push progress to `main` after each session. Check the working
  tree first and preserve unrelated work. `.codex/` is untracked and
  deliberately untouched.
- Record unresolved owner-level semantics/trust choices in
  [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md). Ordinary implementation bugs and
  refactoring do not require an owner question. Existing design prose contains
  ambitions and unresolved conflicts; do not treat it all as implemented fact.

## What exists and where to enter

| Area | Current implementation | Reading entrance |
| --- | --- | --- |
| Kernel | UEFI exit, owned mappings/stacks, frame/heap allocation, interrupts/timer, preemption, isolated user processes, capability syscalls, IPC, admission/teardown from boot templates or sealed objects, recursive resource funding and CPU ceilings; 256 private and up to 512 reader pages per task, sixteen reader rights | [Boot](source-rs/kernel/boot/uefi/main.rs), [user tasks](source-rs/kernel/core/user_tasks/user_tasks.rs), [code grants](source-rs/kernel/core/user_tasks/admission/pool/README.md) |
| Architecture | Shared core with x86-64 and an ARM64 QEMU portability proof; ISA and board responsibilities separated | [Architecture](source-rs/kernel/arch/README.md), [ARM64 bounds](source-rs/kernel/arch/isa/aarch64/README.md) |
| Matrix | Same service recursively owns children, local surfaces, focus, input, realm delegation with scope stamps and attachments, programs run from its realm by owner or child, standing launch policy per slot, close-after-asking, bounded budgets and deadlines; counts unanswered child requests instead of killing a slow child | [Service loop](source-rs/distribution/matrix/service/service.rs), [protocol](source-rs/platform/contracts/matrix.rs), [child launches](source-rs/distribution/matrix/service/children/children.rs), [storage forwarding](source-rs/distribution/matrix/service/parent/storage/storage.rs) |
| Display/input | Software compositor, retained rectangles and sealed bitmap objects, scoped pointer/focus routing, physical PS/2 input | [Display](source-rs/platform/services/display/main.rs), [input](source-rs/platform/services/input/main.rs) |
| Storage | Userspace ATA PIO, guarded redb volume, objects up to 256 KiB, shared content, subtree copies, names/types on edges, scoped relative lookup and indexed type filtering, pins and bounded collection | [Provider](source-rs/platform/services/storage/main.rs), [store](source-rs/platform/libraries/database-objects/README.md), [contract](source-rs/platform/contracts/storage.rs) |
| Desktop | Provider admission/supervision, named login paths, chooser over users, user ceiling and startup request; each user matrix loads its own arrangement. Probe desktop under `lab/` uses the boot catalogue without the shell feature | [Main](source-rs/distribution/desktop/main.rs), [session](source-rs/distribution/desktop/session/session.rs), [shell](source-rs/distribution/desktop/session/shell/shell.rs), [lab](source-rs/distribution/desktop/lab/desktop.rs) |
| Chooser | Lists its realm, launches an entry as its sibling, copies the first entry on NEW, asks the host to close | [Chooser guide](source-rs/distribution/applications/chooser/README.md), [session](source-rs/distribution/applications/chooser/session/session.rs) |
| Notes | Whole-document publication in the binding's realm, bulk reads, duplicate through the store's copy, note switching, explicit reconciliation, sealed bitmap presentation, answers the close question with its draft state | [Session](source-rs/distribution/applications/notes/session/session.rs), [storage](source-rs/distribution/applications/notes/session/storage.rs) |
| Harness | Builds, trims and installs programs and folders; formats and seeds images; host oracles over copies of guest images | [Harness guide](tools/boot-harness-rs/README.md), [seeder](tools/storage-lab/engines/volume/seed.rs) |

The [distribution map](source-rs/distribution/README.md) connects executable and
IPC seams. [STATUS.md](source-rs/STATUS.md) has broader mechanism/proof coverage;
[the Rust guide](source-rs/README.md) has ABI/build details.

The kernel is still bounded: twelve task accounts, a small boot-supplied code
catalogue for boot launches, one CPU, static device grants and finite
page/channel budgets. ARM64 is a headless portability proof, not the desktop.
There is no production DMA/IOMMU driver stack, SMP, GPU stack, audio stack or
production DBFS. Do not equate the object store or BLAKE3 layout research with
a finalized filesystem format.

## Composition facts worth knowing

The owner starts display (Launch 0), input (1), root matrix (2) and object
storage (3) from boot. The programs are objects 1 to 6 of the notes root: the
chooser; the folder of users; inside it the template user, a matrix
executable; and inside the template its chrome (the chooser again), Notes and
Colors. Identical bytes share content. These numbers are installer locations,
not runtime naming policy: [shell.rs](source-rs/distribution/desktop/session/shell/shell.rs)
resolves `Login` and `Users`; the picker resolves `Template`; each user matrix
reads its `Matrix` configuration and resolves its launcher path. Roles come from each executable's entry: a matrix
takes 0, the desktop executable's 2 runs Colors with argument 2, Notes and the
chooser take 2.

Root slot 0 holds the chooser, shown only the users folder; root slot 1 is
opened by a standing policy and holds whatever the chooser launches, a user
matrix. The desktop notices it by polling `STORAGE_BINDING` on that slot and
requests `CONFIGURE` after collecting the picker. The user loads its own chrome
into slot 0 over its realm and opens slot 1 for an app using its saved policy.
NEW copies the template's executable, programs, metadata and configuration;
copied path strings stay unchanged. Apps do not know global coordinates or
siblings. Storage replacement restores the direct user's binding, then requests
`RESTORE_STORAGE`; the matrix retains responsibility for its children's grants.

Page budgets cover retained images at each matrix hop and admitted program code.
The root has 640 private/1280 reader pages; the user matrix receives 576/1024.
Chrome receives 96 private pages; each app receives 128/128. Display has 512
reader pages. The kernel ceilings are 1024 private/1536 reader pages with 48
reader rights, and its heap is 512 KiB. Kernel memory objects remain bounded at
512 KiB; the storage content limit remains 256 KiB. Profile funding is explicit.

Recursive storage scopes are folded at each matrix boundary; the original fixed
three-stamp limit no longer bounds tree depth. A shared deadline bounds the exchange.
Collection before every write scans only metadata and the
key-only chunk index. The older 50/100/150-ms collection/commit/save claims were
not reliable unit conversions and are withdrawn. A fresh traced desktop run is
recorded below. The matrix contract's base storage deadline is 1200 **ticks**,
nominally twelve seconds; the runtime's exchange bound is twice that. Values
were not retuned in the recursive-close change. All are lab bounds.

`--smoke` uses the default profile's `user_programs`, not its desktop startup
graph; the desktop is tested by `desktop_test.py`.

## Next work, in recommended order

The accepted [matrix ownership contract](wiki/spec/execution/components_and_matrices.md)
now resolves the startup/persisted-policy owner question. A matrix is an ordinary
program whose parent grants its ceiling; the matrix controls its own children
within it, using code, configuration, or both. The privileged startup environment
need not be one matrix executable. The
[storage contract](wiki/spec/storage/realms_and_objects.md) requires independent
copies and unchanged path references resolved within the supplied view. Stable
persisted IDs and aliases are deferred. Names and queryable media types belong
to hierarchy metadata; extensions are conventions, not authoritative types.

- [x] **Hierarchy names and queryable media types.** Metadata and indexes commit
  atomically with revisions; lookup and exact-type filtering obey subtree scopes.
  The chooser uses names/types and resolves its hidden template/launcher by path.
  Existing stock users migrate without renumbering or replacing their saved data.
- [x] **Make the user matrix own its startup arrangement.** The stock matrix
  loads `Matrix` in its realm and resolves its saved launcher path. Startup
  requires empty slots; missing paths return to the picker. Parent grants remain
  the ceiling. Custom matrix programs need not use this bounded schema.
- [x] **Make saved layout editable through the desktop.** The launcher edits
  strip width and outer spacing using its existing user-realm grant. Expected
  revisions prevent stale overwrites; lost replies require reconciliation.
  Initial defaults now live in distribution/matrix.conf. Budgets and launcher
  selection are not exposed in the UI.
- [x] **Explicit live layout application.** Stock configuration grants its
  selected launcher geometry authority tied to that endpoint incarnation.
  The child request validates both rectangles, updates future app placement,
  and never changes budgets, services or key masks. Apps recover stale drawing
  handles while retaining documents; tests force two rendering races.
- [x] **Recover refused/crashed app launches and failed user startup.** The
  launcher observes only its latest launched task, reports CANNOT OPEN or APP
  STOPPED, and stays usable. User CONFIGURE failure returns to the picker even
  if collection has already won the race. Placement failure rolls admission back.
  Broken control channels alone cannot authorize close or prove process death;
  task outcomes distinguish those cases. Tests exercise both control directions,
  owner-stop races and exit during a loss survey.
- [x] **Recover the configured launcher locally.** One automatic retry preserves
  the running app, its draft and current layout. Fresh endpoints/bindings carry
  fresh authority; ancestor changes, owner revocation and STOP cancel recovery.
  A repeated failure stops the loop, and a clean exit stays closed.
- [x] **Make login and storage-startup failure visible and retryable.** Missing
  Login/Users paths, picker failure and failed storage readiness show a bundled
  recovery screen. Storage opens behind live input/rendering; a fresh user action
  authorizes each retry. Retries neither repair media nor replenish the automatic
  provider replacement budget.
- [x] **Bound and unwind early bootstrap failures.** Initial display/input/root
  admission gets one retry after task/endpoint cleanup and conservation checks.
  Exhausted root/input recovery shows a static restart screen; failed display
  leaves an explicit launch-terminal diagnostic. Storage has not started yet.
- [x] **Explicitly end an unresponsive app.** The configured launcher receives
  management authority bound to its endpoint and the app slot. END APP warns
  about unsaved work, defaults to Cancel and names the exact observed task;
  stale/revoked tickets cannot kill replacements. A recovered launcher receives
  fresh authority over the existing app. Apps and storage-only endpoints deny it.
- [x] **Offer explicit recovery after launcher retry exhaustion.** Owner-supplied
  fallback code occupies only the retired launcher view. Its current connection
  may request one new attempt; no automatic budget is restored. App drafts survive.
- [x] **Exercise live storage loss in the F5 desktop.** Replacement readiness is
  asynchronous; fresh grants follow readiness, without restarting Notes or login.
  Precommit/lost-reply faults, exhausted retries and blocked replacement readiness
  exercise draft retention, reconciliation and offline close protection.
- [ ] **Continue lifecycle recovery where it is still incomplete.** Explicit
  live-session storage retry, display/input/root loss and clearer focus feedback
  remain. A live storage retry needs delegated authority and a discoverable
  control without replacing the user's app or expanding an app's service grants.
  Existing app-owned close protects dirty work, and layout invalidation keeps drafts.
  Carry the accepted [session-lock boundary](wiki/spec/human_surface/compositor_and_seat.md#session-locking)
  and [general subtree suspension](wiki/spec/resources/execution_budgets.md#owner-controlled-subtree-suspension)
  through an ordinary desktop lock/unlock and a non-Matrix parent/child witness.
  Prove interactive gating independently of execution policy, no descendant
  execution after suspension completes, independent child gates surviving parent
  resume, preserved budget accounting/grants, and an external provider remaining
  eligible. Reject unauthorized control and stale handles; ordinary parent parking
  must leave children eligible. Do not require realm encryption, key destruction
  or storage-grant redistribution for lock/unlock. Existing budget-gate evidence
  does not close this acceptance.
- [x] **Remove desktop requirements from shared matrix machinery.** The stock
  executable/configuration/recovery/control router belong to distribution. Platform
  supplies local authority state with caller-selected child capacity and geometry.
  Alternative visual/headless arrangements have host tests without distro imports.
  An independent three-peer executable now exercises real IPC, source authority,
  revocation and replacement with the existing runtime server and shared local
  commands. Runtime child admission is now shared with the stock matrix, including
  channel rollback and retained endpoint custody; the alternative owns its subtree.
  Upstream/rendering orchestration still needs a second consumer before extraction.
- [x] **Give ordinary apps scoped service acquisition.** One broker connection
  at startup replaces their positional interaction/storage bundle. The sealed
  manifest bounds the offer; endpoint/incarnation selects caller authority.
  Notes and Colors use it in F5. Repeat requests allocate nothing; interrupted
  handoffs reclaim retained rights. Optional/undeclared/incompatible imports and
  cached bindings have host, real-IPC and disposable desktop coverage. This does
  not yet start providers lazily or grant new services after launch.
- [ ] **Bring storage write latency down.** Collection scans metadata before
  each write and durable commit writes through ATA PIO. Use explicit tick-to-time
  conversion and separate CPU time from elapsed time. Measure with
  `--trace-storage` before changing either; the small sample below is not a
  benchmark or a latency guarantee.
- [ ] **Let the lab desktops run from the store too.** The probe desktop and
  the notes fixtures still lend the boot catalogue, which keeps the admission
  and funding proofs on their original path. When those proofs have store
  equivalents, retire RUN by catalogue image from the matrix.
- [ ] **Choose the next kernel/device milestone deliberately.** A second
  userspace MMIO/IRQ device proof is the candidate. DMA needs a separate buffer
  ownership/isolation design.

An intermittent ATA timeout is recorded in the implementation map and remains
undiagnosed. Retain logs and distinguish a reproduced transport failure from a
guessed cause if it recurs. Once in nine runs, `matrix_database_test.py
--notes-recovery` saw the provider's quiet post-commit gate let the first save
through instead of stalling it; the provider now traces `gate-armed` and
`gate-stall` under `--trace-storage` so the next occurrence shows whether the
gate was armed on the instance that served the save.

## Validation and last evidence

Run Cargo commands **inside `source-rs/`** so its toolchain, aliases and linker
configuration apply:

```powershell
cargo test --locked
cargo test --locked -p cathedral-database-objects --features host
cargo check-userspace
cargo check-uefi
cargo clippy --locked --package cathedral-desktop --target x86_64-unknown-none --features shell -- -D warnings
```

From the repository root, select checks appropriate to the change:

```powershell
python tools/boot-harness-rs/desktop_test.py
python tools/boot-harness-rs/run.py --matrix-desktop-test --release --timeout 180
python tools/boot-harness-rs/run.py --matrix-desktop-test --pressure --release --timeout 240
python tools/boot-harness-rs/run.py --matrix-storage-test --release --timeout 120
python tools/boot-harness-rs/matrix_database_test.py --release
python tools/boot-harness-rs/matrix_database_test.py --release --recovery
python tools/boot-harness-rs/matrix_database_test.py --release --notes
python tools/boot-harness-rs/matrix_database_test.py --release --notes-recovery
python tools/boot-harness-rs/objects_test.py --release
python tools/boot-harness-rs/objects_test.py --retention --release
python tools/boot-harness-rs/resource_budgets_test.py
python tools/source-layout/check.py
python tools/source-layout/check_test.py
python tools/source-layout/format-rust.py --check
```

`desktop_test.py` builds the **same default composition as F5**, installs the
same programs into a disposable image, copies a user from the template with
physical mouse input, launches its programs from the chrome, compares exact
note glyphs, status and note bar, refuses dirty closes, saves, duplicates,
edits the copy, switches back, reloads, repeatedly replaces apps, checks Colors
keeps its color, cold-boots the same disposable disk, and has the host lab
verify the retained records on a copy of the image. `--trace-storage` adds
provider phase timestamps to its serial log. Kernel/arch changes need the
smoke and ARM64 suites documented in their guides:

```powershell
python tools/boot-harness-rs/run.py --smoke --release --timeout 120
python tools/boot-harness-rs/arm64/run.py --release
```

Passed before this handoff, on the chooser commit: the five Cargo commands
above plus clippy of every desktop, notes, matrix and storage feature set the
profiles use, both layout checks and formatting, every harness command listed
above (the notes recovery fixture eight times in a row after the one failure
noted above), the x86 release smoke suite, the ARM64 release proof,
`--matrix-test`, `--admission-test` and `--ipc-test`. Oracles verified the F5 image (six
installed objects, a user copied from the template, two notes sharing one
manifest under its Notes, the color under its Colors, empty second root), the
probe desktop images, the notes images, and both objects proofs.

When handing off again, update the baseline commit, next concrete task, known
failures and verification scope here. Keep durable subsystem detail in its owner
docs, and avoid leaving two conflicting descriptions of the default OS boot.

## Recursive-close handoff evidence

The matrix's [loss survey](source-rs/distribution/matrix/service/children/loss.rs)
asks both children without blocking their upstream IPC. It replies only after
all requested answers settle and returns only a boolean aggregate, never child
identities or draft contents. Dirty, missing, busy and failed answers keep the
matrix open. Admission/topology commands refuse while a survey is active.
LOSS remains an advisory observation of cooperative children, not a distributed
freeze that prevents later edits. General crash recovery and persisted matrix
configuration remain separate work.

The headless `--matrix-test --release` now checks dirty descendants, a deliberately
delayed clean reply followed by dirty replies, subtree survival after refusal,
exhaustion and restoration of the eight-unanswered-probe allowance without
closing the live child's endpoint, and clean subtree close/recreation while an
outside sibling survives. At that baseline, the actual
F5 test checked the outer CLOSE against a dirty note, repeated whole-user close and
reopen with clean Notes still running, closed a user containing only chrome, and
verified persistent Notes/Colors plus the cold-boot host oracle. The phase change
below removes those outer controls from the ordinary desktop.

Fresh traced F5 evidence: `build/boot-harness-rs/desktop/20261017T231701527969Z/`.
The first boot had four measured publication spans and two note saves:

| Span | Count | Min / median / max ticks | Nominal milliseconds (10 ms/tick) |
| --- | --- | --- | --- |
| Collection | 4 | 14 / 57 / 73 | 140 / 570 / 730 |
| Commit | 4 | 7 / 21 / 122 | 70 / 210 / 1220 |
| Whole note save | 2 | 80 / 129 / 178 | 800 / 1290 / 1780 |

These spans include normal traced publication work, not injected stalls. They
come from one small TCG run on a host also doing validation work, and do not
establish a stable performance distribution. The deadline-unit correction
changes documentation and the constant's name, not the timeout values.

Validation for this recursive-close change: `cargo test --locked`,
`cargo check-userspace`, shell/matrix target clippy, layout checks, formatting,
`--matrix-test --release` (including exhausted-probe recovery),
`--matrix-storage-test --release`, `matrix_database_test.py --release
--notes-recovery`, and the F5 desktop test with and without storage tracing passed.
The final ordinary desktop run is
`build/boot-harness-rs/desktop/20261017T232223657195Z/`; its independent oracle
verified all 189 retained records after cold boot. Notes recovery evidence is
`build/boot-harness-rs/matrix-database/20261017T232129731671Z/` (16 records).
The earlier intermittent gate miss did not occur in this run; it remains open.
No kernel code or timeout value changed, and no new smoke/ARM64 run is claimed.


## Desktop phases after the recursive-close baseline

The ordinary desktop now centers the user picker, then revokes that chooser's
surface/input grant when a user is furnished. The boot matrix still hosts and
supervises the user, but its picker and global help no longer occupy the screen.
The user receives the full display: a launcher down the left and one app in the
remaining area. No logout control is exposed. Restart QEMU to select another
user; the headless matrix fixture remains the whole-user LOSS/teardown proof.

Follow `shell/shell.rs` for admission and `shell/layout.rs` for phase placement.
The chooser renders its own heading (entry argument 1: CHOOSE USER; otherwise
PROGRAMS), so it retires with its ordinary surfaces. Its two images now require
40 private pages; the user receives 80 to fund that chooser, Notes and spare
capacity within the existing root budget. No kernel or platform protocol changed.

The F5 test now checks picker removal, clicks the former boot CLOSE position
with both a dirty and a clean note open, checks the app reaches the expanded
viewport, replaces apps repeatedly, and cold-boots the same disk to verify
saved text/color and the independent host oracle. Its previous interactive
whole-user close/reopen steps are intentionally superseded by this phase flow.

Validation: `cargo test --locked`, `cargo check-userspace`, shell/chooser target
clippy with warnings denied, layout and formatting checks, and the updated
`desktop_test.py` passed. Final desktop evidence:
`build/boot-harness-rs/desktop/20261018T033411184475Z/` (189 retained records
verified after cold boot). `user-picker.png` and `user-workspace.png` show the
phase transition. No new kernel smoke, ARM64 or headless matrix run is claimed
for this distribution-only change.

## Declared imports and optional persistence handoff

Store programs now carry a bounded required/optional service declaration in
their sealed ELF. Start at [the import contract](source-rs/platform/contracts/imports/README.md)
for parsing, version selection and the admission/runtime reading path. The
matrix resolves it against the owner's offer before reserving or spawning.
Children may withhold offered services but cannot widen that offer. Missing or
incompatible required imports refuse launch; optional storage becomes an actual
`Option` in the runtime view, with no storage endpoints or subtree grant.

F5's user launcher now toggles STORAGE ON/OFF for subsequent launches. Notes
accepts absence and labels its viewport TEMP NO SAVE. Save and reload refuse
without discarding the draft, and dirty-close protection still applies. Colors
requires storage and refuses with NEEDS SERVICE. Re-enable storage and relaunch
Notes to recover saved data. Use NEW at the user picker for current copied app
binaries; existing user copies retain their installed code. Undeclared legacy
images explicitly require both services, so they still run with storage and
refuse when it is withheld.

This proves two wire contracts and the existing three/five-endpoint startup
shape. It does not implement arbitrary endpoint tables, global discovery,
first-request activation or a final persisted manifest. Boot/catalogue fixtures
retain explicit wiring. The startup architecture notes now distinguish
privileged service composition from the matrix executable and hosted login UI;
persisted policy ownership and object kinds/names remain owner questions.

Validation: `cargo test --locked`, the contract tests, `cargo check-userspace`,
shell/chooser/Notes/matrix target clippy with warnings denied, layout and format
checks passed. The updated `--matrix-storage-test --release` passed owner-offer
attenuation, refused widening, refusal/retry, confinement and revocation checks.
The F5 desktop test passed temporary-mode rendering, save/reload refusal, dirty
close, storage restoration, repeated replacement and cold boot; its independent
oracle verified 189 retained records with no temporary writes. Final evidence:
`build/boot-harness-rs/desktop/20261018T052214126134Z/`, including
`temporary-notes.png`.

`matrix_database_test.py --release --notes-recovery` also passed, with 16 records
verified after each cold boot. Evidence:
`build/boot-harness-rs/matrix-database/20261018T052026510579Z/`. The earlier gate
miss did not recur and remains open. During implementation, Rust's `#[used]`
emitted GNU-retain ELF metadata that the kernel correctly refused; the final
declaration uses an exported symbol plus linker KEEP, preserving the accepted
ELF ABI without relaxing kernel validation. No kernel code changed; no new
kernel smoke or ARM64 run is claimed for this milestone.

## Readable chooser labels

The picker now shows USER TEMPLATE, USER 1, USER 2, etc., and NEW USER. The
program launcher shows LAUNCHER, NOTES and COLORS. Profile `display_title`
values are installed as a small footer in program objects, retained by subtree
copies and read through the chooser's own scope. No kernel, platform protocol
or database layout changed. These are display labels; user ordinals are not
editable account names. General object naming/kinds remain open.

At this milestone existing users still retained their old launcher; the upgrade
below corrects that omission. Missing or invalid program titles use PROGRAM 1,
PROGRAM 2, etc.
See the [chooser guide](source-rs/distribution/applications/chooser/README.md)
for the bounded format and reading path.

Verification: chooser target clippy with warnings denied, layout/format and
diff checks passed. The F5 desktop test checked exact labels before and after
cold boot, optional storage, draft protection and persistent state; the host
oracle verified all 189 retained records. Evidence:
`build/boot-harness-rs/desktop/20261018T053452234078Z/`, including
`user-picker.png` and `user-workspace.png`. No new kernel/ARM64 run was needed.

## Existing-user label upgrade

The first label fix only refreshed installed templates, leaving existing users
with OBJECT rows. The [distribution migration](source-rs/distribution/migrations/README.md)
now identifies old stock copies by retained template manifests or the verified
legacy allowlist. It replaces their launcher and appends labels to old app code
in place, preserving object identities, saved data and customized programs.
F5 runs it after seeding and records its fingerprint in the install sidecar.
The general policy for pinned copies versus managed program updates remains
undecided; this is an explicit distribution compatibility upgrade.

The interactive disk had two users with old stock code. Six program objects
were upgraded, with a backup at
`build/boot-harness-rs/cathedral-storage.raw.before-readable-labels.raw`. No
new user is required. Old Notes/Colors retain their previous features.

Verification: host migration tests and storage-lab clippy with warnings denied
passed. Compatibility replay of the pre-label desktop fixture entered its
existing user, checked the exact labels and read both saved notes:
`build/boot-harness-rs/labels-upgrade/20261018T163807055999Z/`.
The normal F5 desktop and cold-boot proof also passed, with all 189 records
verified by the host oracle:
`build/boot-harness-rs/desktop/20261018T163829248414Z/`.
No guest code, kernel protocol or database schema changed in this correction.
Layout/format checks and the persistent-image host tests passed too. The latter
exposed an existing eager host-tool build before the no-op/invalid-image checks;
image preparation now builds only when formatting, seeding or upgrading is needed.

A booted copy of the actual interactive disk confirmed USER 2's corrected labels:
`build/boot-harness-rs/labels-upgrade/interactive-check-20261018T164544/`.
The preceding two attempts hit the already tracked ATA completion timeout before
UI startup: op=30, LBA=24, status=d0, 50 ticks, with 4825 and 5347 polls. Their
logs remain under `interactive-copy-20261018T164207/` and
`interactive-retry-20261018T164335/` in the same labels-upgrade directory. The
next attempt passed without changing the driver or deadline; the cause remains
unresolved. The interactive source image was never attached to those guest tests.


## App-owned close controls

Notes and Colors now render an X in their own local viewport. Closing returns
from the app; the existing matrix child-disconnection path reclaims its grants,
surfaces, input and launch slot. There is no new close syscall or matrix opcode.
Notes presents SAVE / DISCARD / CANCEL when work is unconfirmed; temporary
Notes disables save. Cancel preserves the draft and modal keyboard input cannot
edit it. An uncertain save reconciles first; failure or a differing confirmed
value keeps the dialog open for an explicit next action. Colors requires CLOSE
ANYWAY after a storage error, and its LOSS reply now reports that error too.

Follow Notes' session into `closing.rs` for pointer actions and `view.rs` for
presentation; Colors' app entrance similarly descends into `closing.rs`.
Notes uses its existing text bitmap for the dialog and its sixth allowed surface
for the X. An initial separate dialog bitmap exceeded the 32-page allowance
when replacing text; reusing the text bitmap fixed that without raising budgets.

The profile explicitly opts into `app_close_migration`. Host installation
refreshes only recognized stock Notes/Colors and launchers, including the old
apps previously given title-only footers. Recognition checks content digests;
a title alone cannot authorize replacement. User matrix code, object identities,
child data and customized programs remain unchanged. The actual interactive
image's two users received four app updates, backed up at
`build/boot-harness-rs/cathedral-storage.raw.before-app-close.raw`.

Verification for this change:

- Notes host test, shell Notes/desktop and notes-probes target clippy with
  warnings denied, four host migration tests, storage-lab clippy, two persistent
  image tests, layout/format and diff checks passed.
- Default F5 desktop proof covered clean exit/relaunch, dirty save/discard/cancel,
  modal draft protection, temporary refusal/discard, launcher LOSS behavior,
  saved colors and cold reboot. Host oracle verified 193 records. Artifacts:
  `build/boot-harness-rs/desktop/20261018T172431571623Z/`.
- Existing-user compatibility replay upgraded the pre-label fixture, checked
  both saved notes, and used the X to close/relaunch inside its old matrix:
  `build/boot-harness-rs/labels-upgrade/20261018T172626738714Z/`.
- Notes recovery/focus/glyph checks and cold restoration passed, with exact
  16-record host oracles:
  `build/boot-harness-rs/matrix-database/20261018T172850295341Z/`.

This does not supply owner force-close for an unresponsive app, a global escape
shortcut, or new focus indicators. Colors' uncertain-close confirmation was
compiled but not separately fault-injected through the shell UI. Kernel and
ARM64 suites were not rerun for this application/installer change. The existing
intermittent ATA timeout remains unresolved; none appeared in these runs.


## User tiles and absolute pointer

The ordinary desktop now opens to horizontal user/avatar tiles across a
full-screen picker, with the template hidden. NEW USER makes a copy; at the
existing three-user listing limit the add tile disappears. No scrolling or
account-management framework was added. Inside a user the same chooser shows
Notes/Colors icon tiles. The self-launcher entry, STORAGE ON/OFF, launcher CLOSE
and normal status rows are gone. Failed actions give brief text; apps retain
their X buttons and close decisions. Required/optional imports and matrix
LOSS/CLOSE remain platform mechanisms, without default GUI test controls.

Chooser sequencing stays in `session/session.rs`; `layout.rs` shares geometry
with pointer hit testing, `view.rs` publishes surfaces and `tiles.rs` owns raster
icons. Five chooser surfaces plus six app surfaces fit the user matrix's eleven;
the root's five picker surfaces and eleven user surfaces fit its sixteen.
The chooser receives 48 private pages. Each tile uses 12; old failure text is
released before replacements to keep three-user repaint within that budget.

The default input profile enables `absolute-pointer`. The input service's
`absolute.rs` negotiates QEMU VMMouse and scales absolute axes; the matrix API
still gets ordinary local coordinates. A restricted INPUT_VIRTUAL_POINTER syscall
requires the exclusive bootstrap input grant, validates the complete reply
buffer before any device effect, and delegates whitelisted operations to the
PC board adapter. No arbitrary ports or general hypervisor calls are exposed.
The owner retirement path disables virtual input; unavailable/failed absolute
input falls back to PS/2. Other lab profiles keep relative input. This is QEMU
integration, not a USB/HID driver. See `platform/services/input/README.md`.

The existing stock-program migration refreshed both launchers and Colors in the
actual interactive disk (four copied objects). Notes data and user matrix code
remain intact. A separate pre-upgrade backup is
`build/boot-harness-rs/cathedral-storage.raw.before-user-tiles.raw`.
A booted copy showed both user avatars, entered USER 2, and opened its Notes:
`build/boot-harness-rs/labels-upgrade/interactive-tiles-20261018T174902/`.

Verification:

- Full Rust host suite, chooser/desktop/input and kernel target clippy,
  layout/format/diff checks passed. The new board test rejects non-pointer
  commands and oversized/truncated operands. Colors checks that an ordinary
  app cannot access the physical virtual-pointer transport.
- Default F5 desktop passed absolute-coordinate clicks, user creation, app
  launching, dirty save/discard/cancel, repeated exit/relaunch and cold reboot;
  host oracle verified 199 records:
  `build/boot-harness-rs/desktop/20261018T174221890406Z/`.
- The same desktop flow with VM port disabled passed PS/2 fallback:
  `build/boot-harness-rs/desktop/20261018T174441789342Z/`.
  Reproduce with `desktop_test.py --relative-pointer`.
- Existing-user upgrade, saved notes and app close passed on the pre-label
  fixture: `build/boot-harness-rs/labels-upgrade/20261018T174406660935Z/`.
  Three-user creation, full-row keyboard repaint, existing notes and close also
  passed on a newer fixture at `labels-upgrade/20261018T174733693334Z/`.
  Use `labels_upgrade_test.py IMAGE --fill-users` for that additional limit check.
- Release x86 smoke and ARM64 portability boots passed. No new ARM device was
  introduced; the shared syscall routing still compiles/runs there.

**Earlier storage finding (retest below now passes):** With the older
`desktop/20261017T232223657195Z/guest.raw` fixture upgraded, adding USER 2 twice
hit a storage deadline and provider replacement; the picker displayed TRY AGAIN.
Existing-user selection/readback passed before that extra stress. Artifacts:
`labels-upgrade/20261018T174631398112Z/` and `20261018T174825636437Z/`.
The newer fixture's equivalent two extra copies passed. At that point the
cause was unproven. After bounding foreground collection, the exact older
fixture now creates both extra users and preserves its notes:
`labels-upgrade/20261018T181109147782Z/`. This is consistent with the traced
Colors cleanup stall below; the earlier copy failure itself was not traced,
so do not relabel it as a proven ATA fault or claim every uncertain-copy case
is fixed.
No retry of an uncertain copy was added. The original chooser already lacks a
reconcile/reload action after uncertain creation; investigate that alongside the
storage stall before adding broader account controls. The new capacity check
is opt-in so its distinct purpose is explicit.

The old F5 test's storage-toggle/temporary-Notes steps were removed with that
UI. Earlier evidence remains above; service import policy host tests still run,
but this milestone does not claim new end-to-end optional-storage UI coverage.


## Transient picker handoff

The login design already says the broker exits after root transfers the seat.
The Rust picker was instead kept alive with its display/input revoked only
**after** loading the user's launcher. Slow storage therefore left the old
picker visible during launcher startup. The chooser guide also misleadingly
said the same program "becomes" the app strip; those were always separate
process instances, now documented with the process tree in distribution/README.

`shell::furnish` now starts by STOPping and collecting root slot 0 through its
existing owner interface. That path flushes retired surfaces, removes input and
storage connections, waits for process death and returns funding. The owner
checks the old control route is PEER_CLOSED, then reads/starts the user launcher.
No kernel/matrix protocol change. The selected user remains its sibling; picker
teardown cannot kill that user's tree. If the user later disappears, the owner
runs a fresh picker instead of trying to reveal the collected one. Provider
recovery grants only the currently live side of this phase boundary.

This implements lifecycle only. Warden verification, secure credential entry,
realm unsealing and seat-bound session leases are still not implemented. The
existing design is the reference; there is no new owner question about whether
the broker should stay in the user's process ancestry.

Validation: shell target clippy, layout/format/diff checks and the full default
F5/cold-boot test passed at `desktop/20261018T175557759835Z/` (199 host-oracle
records). The test now checks the whole former avatar band, clicks the old tile
location and checks it stays clear, and checks the picker-collected milestone
precedes user-launcher readiness. The former single-pixel check sampled a gap
between tiles and could miss stale avatars. User-matrix crash recovery's fresh
picker path was updated but not fault-injected in this pass. No kernel or ARM
changes; those suites were not rerun.

Existing-user compatibility replay also passed with the stronger handoff checks:
`labels-upgrade/20261018T175748538255Z/`. The interactive image's recognized
stock Colors copies were refreshed because Colors shares the owner executable;
the actual fix runs in the booted owner, so old user matrix/launcher binaries
need no changes. Saved data is preserved; the pre-update image is at
`cathedral-storage.raw.before-picker-handoff.raw`.


## Colors on retained desktop disks

Reproduced the owner's actual failure on a copy of the two-user interactive
image. Colors' initial six-byte state publication entered collection; the
provider tried up to 64 passes (each rescanning liveness and committing eight
removals) before doing the save. Accumulated program-upgrade history exceeded
the 600-tick/six-second IPC bound. Root revoked the user's storage binding;
the owner interpreted that as user death, attempted to restart the picker,
and panicked on BUSY. The last frame then looked frozen. Before-fix evidence:
`colors-check/20261018T180419Z/`, traced `colors-check/20261018T180654Z/`.

The provider now performs one bounded pass per publication/copy. It preserves
atomic publication and exact accounting; additional maintenance proceeds on
later mutations or explicit COLLECT. It can return capacity before all garbage
is drained. Background collection and quota-pressure scheduling remain future
work. The owner separately confirms a closed control route before forgetting
a remembered realm; a storage error, BUSY or timeout cannot restart login.

`colors_test.py IMAGE --users N --user I` upgrades a disposable copy and checks
Colors **first**, save, clean close, reopen, and cold-boot saved color, rejecting
panics or provider replacement. Both actual saved users passed:
`colors/20261018T180957340154Z/` (USER 2, first launch 2.52s, cold 0.38s),
`colors/20261018T181122360912Z/` (USER 1, 3.30s / 0.73s). Times include scanout
observation overhead and are TCG samples, not performance guarantees. The same
USER 2 disk without the upgrade step opened in 1.36s after the click at
`colors-check/20261018T180812Z/`.

Full default desktop/cold boot and independent 201-record oracle passed at
`desktop/20261018T180905741067Z/`. Object retention, bounded collection, exact
accounting and three cold boots passed at `objects/20261018T181138302783Z/`.
Host `cargo test --locked`, shell/storage target clippy, formatting and layout
checks passed. Kernel/ISA behavior is unchanged; no new ARM boot claimed.


The explicit lost-reply variant (`colors_test.py IMAGE --users 2 --user 2
--recovery`) also passed, including reconciliation and cold boot, at
`colors/20261018T181550212623Z/`. It adds the existing quiet post-commit gate to
the F5 composition and checks that exactly one provider replacement occurs
without restarting the picker. This exposed a second recovery bound: opening
an unclean retained volume exceeded the old 500-tick startup wait. Replacement
startup now has a separate 3000-tick/30-second bound; individual storage IPC
operations retain their six-second bound. Replacement failures now log the
actual error code. Earlier injected failures and the successful exploratory
run are retained at `colors-check/20261018T181226Z/`, `20261018T181317Z/` and
`20261018T181411Z/`. The longer startup bound is not a recovery-speed claim.

Final ordinary (fault gate absent) replay passed at
`colors/20261018T181641027782Z/`. The normal F5 EFI was rebuilt and the actual
interactive disk's recognized Colors copies upgraded with saved state intact;
`cathedral-storage.raw.before-colors-launch.raw` preserves the disk before
this update. The existing app-close migration backup is also retained.

## Matrix policy ownership decision (2026-10-18)

The owner question on startup composition and persisted matrix policy is closed
into [components and matrices](wiki/spec/execution/components_and_matrices.md).
The parent grants the ceiling; the matrix program controls and may persist its
own internal arrangement, with explicitly delegated editing and explicit live
application. Code-defined/custom attenuation is equally valid. Service imports
are requests, never grants; secrets are supplied as operations, never credential
bytes in a child's configuration. A Warden is a provider role, not necessarily
one process or hardware backing.

The decision does not choose a universal matrix schema, mandate a program's
subtree as its realm, settle object naming/copy references, or implement Warden,
authentication or encryption. The initial owner still furnishes user matrices;
removing that special knowledge is the next implementation milestone. Existing
Rust import/launch contracts document their bounded evidence; no ABI or runtime
behavior changed in this documentation pass.

## Independent-copy decision (2026-10-18)

The owner accepted [independent copy semantics](wiki/spec/storage/realms_and_objects.md):
hierarchy and backing are distinct, CoW is the default sharing model, and edits
to either original or clone preserve the other. Start with immutable replacement
writes; exclusive-use reuse can follow later without changing semantics.
Retained views and readers count when deciding whether backing is reclaimable.

The Rust store already shares immutable content with independent edits, but
subtree copies still allocate numbered metadata per descendant. Shared hierarchy
roots are not implemented. Logical IDs, copied configuration references and
aliases remain the narrowed owner question; view-relative IDs are a proposal,
not an accepted answer. This pass changes documentation and protocol comments
only, with no runtime or wire changes. Markdown links and diff checks were
verified; runtime tests were not rerun for this documentation change.

## Persisted path-reference decision (2026-10-18)

The owner accepted [paths within the supplied view](wiki/spec/storage/realms_and_objects.md)
for ordinary persisted references. Copies preserve path strings; missing targets
fail lookup, with no source-realm fallback or dependency repair. Rename/move may
break a path. Stable persisted IDs and aliases are deferred, so the copied-reference
owner question is closed rather than retained as a blocker.

The guest still uses numbered objects and has no named path lookup. Name/kind
representation is the remaining owner decision for matrix startup paths; internal
number allocation is not itself a public stable-reference contract. No runtime or
wire behavior changed in this documentation/comment pass. Local Markdown links
and diff checks passed; runtime tests were not rerun.

## Hierarchy name and media-type decision (2026-10-18)

The owner confirmed explicit [hierarchy metadata](wiki/spec/storage/realms_and_objects.md)
for names and media types, enabling indexed database filtering without filename
searches or opening every body. Extensions remain program conventions; importers
and explorers may infer and set type metadata when authorized. A declared type
is not proof of valid contents or permission to execute them. Content and
children may coexist, and the same backing can have different metadata through
independent entries.

The names/kinds owner question is closed. The next storage work is implementation:
metadata records, named lookup and scoped type filtering, followed by matrix-owned
startup paths. The guest still has numbered objects without these features.
No wire or runtime behavior changed in this documentation/comment pass. Local
Markdown links and diff checks passed; runtime tests were not rerun.

## Named realms and matrix-owned startup implemented (2026-10-18)

This completes the first implementation of the preceding decisions. F5 still
boots the ordinary desktop. The outer owner resolves `Login` and `Users`, grants
the selected user its ceiling, collects the picker, then requests `CONFIGURE`.
The user matrix loads its own `Matrix` object, resolves the saved launcher path,
and installs launcher/app placement and budgets. It no longer relies on the
outer owner knowing internal object numbers or observing its current app.

Follow [hierarchy.rs](source-rs/platform/libraries/database-objects/hierarchy.rs)
for edge metadata, exact name and type indexes, and atomic revision-checked
updates; [provider dispatch](source-rs/platform/services/storage/hierarchy.rs)
and [Scope](source-rs/platform/libraries/user-runtime/storage.rs) connect those
operations across IPC. Names/types are bounded to 32 UTF-8 bytes each; relative
lookup is bounded to 256 bytes, permits `.`, and rejects parent traversal and
absolute paths. Named siblings are unique with exact comparison. FILTER returns
up to six direct children of an exact media type. These lab bounds do not freeze
the eventual schema or type vocabulary. Pins still preserve content revisions,
not complete historical hierarchy metadata.

The chooser uses the executable-type index and hierarchy names. Its hidden
template is resolved by path; its hidden launcher comes from saved configuration.
NEW copies the template as `User N`, preserving descendant names/types and path
strings while sharing content. A misleading extension does not produce an app
tile. Unknown legacy metadata remains representable; there is no explorer or
metadata-editing UI yet. Name collisions refuse rather than overwrite.

[Configuration](source-rs/distribution/matrix/contracts/config.rs) is the stock
matrix's bounded 104-byte `CTHMAT01` record: launcher path, page/reader budgets,
margin, launcher width, gap, app surfaces and key mask. The
[loader](source-rs/distribution/matrix/service/children/configuration/configuration.rs)
requires empty child slots, checks geometry/key limits and uses ordinary
admission. Failed startup returns to the picker. No settings editor or live
apply is implemented. Parent grants remain authoritative; saving policy never
expands them. Current initial defaults are part of the explicit host migration;
move their selection into distribution composition before supporting alternate
stock arrangements or a settings workflow.

Each matrix remembers the source and scope of its own live child grants.
`RESTORE_STORAGE` restores those after provider replacement; successful explicit
revocation and child death clear remembered policy. Matching live grants are
left alone. Failed local storage exchanges retire their stream so a late reply
cannot be consumed as the next request's reply.

[Stock migration](source-rs/distribution/migrations/README.md) first refreshes
exactly recognized program copies, including old user matrices, then adds names,
indexes and configuration without renumbering or rewriting saved application
data. Existing metadata/configurations survive repeats. The exact old matrix
manifests added to the allowlist were checked against retained successful boot
fixtures; arbitrary matrix-shaped bytes are not accepted. All guest migration
tests used image copies; the live interactive disk was left untouched and F5
will run the normal upgrade on its next launch.

Evidence for this change (paths relative to `build/boot-harness-rs/`):

- `cargo test --locked` passed, along with 12 database-object host tests and
  30 storage-lab engine tests. Contract tests include invalid configuration
  geometry/key masks. Database tests cover stale/duplicate metadata updates,
  copied indexes, deletion, quota and reopening; migration tests preserve other
  roots and saved state and verify idempotence.
- Userspace and UEFI clippy, desktop shell/notes-probes/trace-storage and provider
  recovery/trace feature checks, host database/storage-lab clippy with warnings
  denied, source-layout checks (35 Omega/87 Rust edges), five layout tests,
  formatting, Python compilation and diff checks passed. No new ARM64 boot or
  full kernel smoke claim is made for this userspace/storage change.
- `desktop/20261018T211244773335Z`: default F5 creation, Notes edit/duplicate,
  app close/switch, cold boot and exact host oracle (241 records) passed.
- `hierarchy/20261018T211015023781Z`: updated launcher path after rename,
  extension-looking text exclusion, migration preserving owner edits, and
  missing launcher returning visibly to the picker passed.
- `objects/20261018T210147888941Z`: guest metadata/lookup/filter scope refusals,
  valid in-scope query, existing object operations and both cold-boot oracles passed.
- `colors/20261018T210311121562Z`: copy of the retained two-user interactive
  disk upgraded, launched/saved/reopened Colors and cold-booted successfully.
- `colors/20261018T211700524443Z`: final recovery code on a fresh one-user
  fixture survived the injected lost committed-save reply, provider replacement,
  explicit child grant restoration, reconciliation and cold boot.
- `matrix-storage-test`: final recursive scope/admission/replacement suite passed,
  including restoration preserving revocation, the outside child's source, and
  no inheritance of a dead child's remembered grant.

One limit found: recovery-profile installation on the older retained two-user
disk copy exhausted the 4 MiB engine extent before QEMU started
(`colors/20261018T210353517960Z`, requested 4,214,784 bytes, capacity 4,186,112).
Normal migration/boot on that retained disk copy passed; recovery passed on the
fresh fixture. This accumulated-upgrade capacity issue is not fixed by raising
logical quota and remains work for the installer/storage lifecycle. Never
silently replace or reformat a user's image to hide it. The prior intermittent
ATA timeout and quiet fault-gate occurrence remain recorded above, unresolved.

## Saved layout editor and reliable profile switching (2026-10-18)

The normal user launcher now has SETTINGS, also reachable with F2 when it has
focus. It edits strip width (192ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¾Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¾ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¾Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¾Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦ÃƒÂ¢Ã¢â€šÂ¬Ã…â€œ320 pixels) and outer spacing (8ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¾Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¾ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¾Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¾Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â¦ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦ÃƒÂ¢Ã¢â€šÂ¬Ã…â€œ40 pixels).
Save writes that user's `Matrix` configuration; Cancel discards local edits and
returns to app tiles. The current app keeps running in the same position.
Restart QEMU and select the user to apply the saved layout. This is a bounded
saved-layout editor, not live reconfiguration or a budget/launcher editor.

Start at the chooser's [session](source-rs/distribution/applications/chooser/session/session.rs):
route local input to [selection](source-rs/distribution/applications/chooser/session/selection.rs)
or [settings](source-rs/distribution/applications/chooser/session/settings/settings.rs),
then publish the selected view. Settings owns its draft, storage decisions and
small panel renderer. The launcher already has an explicit grant over the user
realm; apps still receive their own subtrees. No new kernel or platform authority
was added. Its zero reader-page allowance exposed an early bulk-read refusal;
the final code reads this 104-byte record through inline replies checked against
one revision, and retains the existing 48-page rendering allowance.

Saving checks the path still names the same object and refuses an intervening
different revision/content. Publication uses the expected revision. A lost
acknowledgement remains uncertain; clicking Save again reads back the desired
content before deciding whether a write is needed. Unsaved/uncertain work reports
LOSS. Cancel does not undo a write that already committed. A provider outage can
make opening the panel say CANNOT OPEN; retry after recovery. There is no editor
for resolving concurrent conflicting drafts yet.

Initial values moved from host-tool constants to
[distribution/matrix.conf](source-rs/distribution/matrix.conf). The host encodes
those defaults, preserving existing template/user configurations during upgrades.
Changing the file does not silently reset saved arrangements.

Testing revealed a separate build issue: after a recovery profile, a normal
desktop run reused a kernel bundle containing the quiet fault gate. The failed
default proof is `desktop/20261018T213314127325Z`; its log shows gate arming and
stalling during Notes save. Cargo's package output paths were reused across
feature sets, and cached outputs could restore older timestamps. The harness
now [captures each built role](tools/boot-harness-rs/artifacts.py) at a path keyed
by content digest, immediately after building it. The kernel build already
watches those environment paths, so changing feature bytes invalidates the
bundle independently of timestamps. Store-program trimming also uses each
captured role, avoiding later same-package builds replacing its input. Keep
different profile builds sequential; this is not a concurrent-build coordinator.

Verification (artifacts under `build/boot-harness-rs/`):

- `desktop/20261018T213552026966Z`: final normal desktop after recovery-profile
  use passed user creation, Notes editing/duplication, close/switch, cold boot
  and exact retained-record oracle (247 records).
- `settings/20261018T213712506523Z`: final normal three-boot proof passed Cancel,
  Save, unchanged live app placement, persisted dimensions after restart, and
  a newly created second user retaining independent template defaults. Installer
  reruns left the edited image byte-identical.
- `settings/20261018T213417759204Z`: two-boot recovery proof passed a deliberately
  lost committed settings-save reply, provider replacement, reconciliation and
  cold layout restoration. Its trace contains exactly one publication despite
  the retry. The third-boot new-user check belongs to the normal proof; running
  it with the gate rearmed also stalls Colors' initial save and can transiently
  refuse opening Settings, which is a separate fault from the settings save.
- `cargo test --locked`, all 30 storage-lab engine tests, userspace clippy and
  storage-lab clippy with warnings denied passed. The artifact regression test
  checks distinct feature bytes with identical timestamps, preserved captured
  inputs, and returning to the earlier content identity. Layout checks (35/87
  package edges), five layout tests, formatting, Python compilation and diff
  checks passed. Kernel smoke and ARM64 boots were not rerun.

At this milestone the prior fixed-volume upgrade capacity limit remained. These
GUI/fault proofs used disposable copies; the interactive disk was left untouched.
The follow-up below repairs the actual F5 installation path.

## F5 upgrade capacity repair (2026-10-18)

The owner's normal F5 launch failed before QEMU started: seeding updated apps
requested 4,214,784 bytes from the 4,186,112-byte engine capacity. Reproduced on
a copy of the actual two-user disk. Physical compaction alone allowed the
same installation to finish, without collecting history or enlarging the volume.

[database_image.py](tools/boot-harness-rs/database_image.py) now invokes offline
[compaction](tools/storage-lab/engines/volume/compact.rs) before updating existing
media whose installation receipt changed. Unchanged launches never open the
database; newly formatted media skips maintenance. The host locks the image,
saves and syncs a full `.before-compact-<BLAKE3>.raw` backup, compacts, cold-opens
and checks all table records and volume identity. A conflicting backup is
refused. Maintenance failure stops the install and does not write its receipt.
No kernel, guest storage format, quota, hierarchy or saved-data semantics changed.

Verification:

- All 31 storage-lab tests and clippy with warnings denied passed. The new
  integration test checks exact records across cold reopen/repeated maintenance,
  retained history, reclaimed engine length, exact backup bytes, backup-conflict
  refusal and unchanged image size/outside-region guards.
- Five Python installer/artifact tests passed, including maintenance-before-seed,
  unchanged receipt bypass and refusal without seeding or recording success.
- `colors/20261018T220311689389Z`: a copy of the actual retained two-user disk
  compacted from 2,109,440 to 1,425,408 logical engine bytes, preserving all 394
  records. App installation/migration, login, Colors save/close/reopen and cold
  boot passed. First and cold app launches took about 0.45 and 0.39 seconds.
- The exact F5 command then upgraded the actual interactive disk successfully.
  Its pre-maintenance backup is beside the disk, digest
  `552ed73fd21d7be7b1abc1fe1b4e5a8e0279fba6870058006294befd26f57201`.
  QEMU displayed both existing user tiles; the actual window capture is
  `f5-diagnosis/actual-window.png`. The GUI was left open for the owner.
- Layout and diff checks passed. Guest code was unchanged; kernel/ARM64 and the
  full fault suite were not rerun.

This fixes the observed reclaimable-space failure, not every capacity limit.
The extent is still 4 MiB; truly full media or compaction without scratch space
can still refuse. Volume growth and installer-wide atomicity remain future work.
Backups accumulate only on changed installations and are never silently deleted.
The prior ATA timeout and quiet fault-gate caveats remain unresolved.

## Authorized live layout and rendering invalidation (2026-10-18)

SETTINGS / SAVE now persists the user's arrangement and requests immediate
geometry application. Colors stays running, Notes keeps dirty drafts and close
protection, and later launches use the updated app rectangle. Cancel still
abandons only local edits. Saving and applying are separate operations: APPLIED
means both succeeded; SAVED; RESTART means the durable save succeeded but live
application was refused. Save can be retried without repeating an acknowledged
write. Budgets, launcher selection and key-mask changes still require startup;
editing the configuration object alone does not automatically change live policy.

The stock matrix explicitly grants its configured launcher's interaction endpoint
layout authority, bound to its incarnation and checked against its current view
epoch. Ordinary apps, the picker, storage endpoints and stale incarnations cannot
use it. Authority is checked before reporting busy owner operations. The new
`APPLY_LAYOUT` request accepts only margin/strip width/gap, validates both
rectangles before changing either, preserves current key masks and allowances,
and does not revive a revoked sibling view. Controller revocation/death or
parent-binding replacement clears the grant. Equal geometry is idempotent.
Start at [configuration.rs](source-rs/distribution/matrix/service/children/configuration/configuration.rs)
for the grant and standing-placement update, then
[layout.rs](source-rs/platform/libraries/matrix/world/layout.rs) for authority and
geometry. The protocol is owned by platform/contracts/matrix.rs; no kernel
change or owner-control endpoint was exposed to children.

Actual resizes can invalidate a draw halfway through a frame. Chooser, Notes and
Colors now propagate rendering refusals to their session, verify the view really
changed or was revoked, and rebuild surfaces while keeping application state.
Unrelated refusals under the same live epoch remain errors. Colors' ordinary
surface setup no longer executes its adversarial lab probes; those remain in the
lab build. Existing focus/held-key suppression handles changed extents; app-local
button tracking resets for a new epoch.

Verification (artifacts under build/boot-harness-rs):

- Workspace host tests passed. The new policy test covers absent/sibling/stale
  authority, malformed/overflow/no-room requests without partial placement,
  stale drawing epochs, idempotence, held-key suppression, preserved revocation
  and controller teardown. Userspace checks and clippy with warnings denied
  passed for matrix, chooser, Notes including render probes, and desktop shell
  and lab variants. Layout checks, five layout tests and formatting passed.
- `settings/20261018T221332506921Z`: normal three-boot proof passed Cancel,
  immediate Colors layout, repeated resizing with a dirty Notes draft, continued
  editing, app-owned close protection, save/close, later app placement, persisted
  geometry and independent new-user defaults. The earlier run's exact bitmap
  comparison failed because the pointer covered the note; the corrected test
  moves the pointer outside the compared bitmap before checking.
- `settings/20261018T221519348512Z`: test-only Notes gates forced a resize after
  the first surface allocation and another during bitmap rendering. Both stale
  frame paths executed, Notes loaded its document exactly once and typing worked
  afterward. These gates compile only with `render-probes`, never normal F5.
- `settings/20261018T221620237698Z`: lost committed-save reply, provider recovery,
  explicit storage restoration, readback reconciliation, live application and
  cold boot passed. The trace still shows exactly one settings publication.
- The matrix desktop suite passed nested clipping, focus, subtree replacement
  and cold colors using the lab build's surface probes.
- `desktop/20261018T221823899504Z`: the final default F5 composition passed
  user creation, app selection, Notes editing/duplication, close choices,
  repeated replacement and cold boot. The independent host oracle verified
  all 243 retained records, including exact hierarchy/history/quota and the
  empty ungranted root. This final build restored the normal F5 artifacts after
  the test-only profiles.

The interactive QEMU session and saved disk were left alone throughout this
milestone. Tests used disposable copies. F5 installs updated recognized programs
on the next launch; close the old QEMU window before launching another instance
against the same disk. No kernel or ARM64 behavior changed; those boot suites
were not rerun. Live child replacement, resource-budget reconfiguration and
startup/crash UI recovery remain separate next work. No new owner decision was
needed for this bounded geometry request.


## F5 retained-history quota repair (2026-10-18)

A subsequent F5 failure was reproduced from a copy of the actual two-user disk:
the Rust build passed, compaction preserved 614 records and reclaimed physical
space, seeding updated five stock objects, but `upgrade-app-close` aborted with
`label upgrade exceeds root quota`. Retained record bytes and physical engine
space are separate limits; physical compaction alone cannot clear this failure.

Stock refresh now plans recognized copies before removing any template history,
stages all replacement heads, and, only under quota pressure, runs the same
head/pin reachability collector as the guest inside its transaction. It commits
the replacements and collection together. Current files, customized programs,
hierarchy, identifiers and pinned revisions survive; unreferenced history may be
collected under the existing retention semantics. Live/pinned content that still
cannot fit refuses and rolls back the entire refresh. Pre-upgrade backups remain.
The 1 MiB root quota and 4 MiB interactive extent are unchanged. Seeding and
hierarchy migration remain separate transactions; installer-wide atomicity and
true live-capacity exhaustion remain limitations.

The reusable transaction pass is `collect_records` in
[collection.rs](source-rs/platform/libraries/database-objects/collection.rs).
Guest `Store::collect` still commits one bounded pass with identical policy.
The host [stock refresh](tools/storage-lab/engines/volume/labels.rs) calls it
only after its replacement plan has been staged, preserving stock recognition
from history. The host tool now depends on this shared library instead of
implementing another collector; redb's unified experimental API needs explicit
ReadableTable imports in the existing adapters.

Verification:

- All 32 storage-lab tests and its ordinary clippy check passed. New pressure
  coverage checks cold records, current data/custom code, durable pins and their
  chunks, exact quota, backups, image guards, repeat idempotence, and complete
  logical rollback when live content exceeds quota. The synthetic quota fixture
  has 8 MiB physical space to isolate the 1 MiB logical admission limit.
- Workspace host tests and all 12 database-object host tests passed, including
  interrupted collection and fixed-extent reuse. Database-object clippy, source
  layout, formatting and diff checks passed. A broader storage-lab `--all-targets`
  clippy run found the existing `hierarchy.rs` test module before `inspect`; the
  ordinary clippy target passes. No kernel/ARM64 checks were rerun.
- `colors/20261018T223112972028Z` upgraded a copy of the actual retained disk:
  charged bytes fell from 1,310,925 to 1,041,964 while refreshing eight stock
  copies. Login, Colors launch/save/close/reopen, and cold persistence passed.
  Picker startup took 3.80 s and 10.95 s; app launches about 0.48 s and 0.59 s.
- The old interactive QEMU remains running against `cathedral-storage.raw`;
  the real image was not upgraded underneath it. Close that window before F5;
  the next launch applies this repair automatically without a reset.

At the time of this repair, launch-outcome/startup-failure work remained
uncommitted. The following milestone completes that work and its verification.


## Recoverable child launch and scoped outcomes (2026-10-18)

The ordinary launcher now displays CANNOT OPEN on refused admission and APP
STOPPED when its admitted program faults or returns nonzero. It remains available
for another program; a clean exit is not labeled a crash. User CONFIGURE failure
returns to a fresh picker with CANNOT OPEN. Startup cleanup accepts a child that
has already been collected, fixing the race where failed startup panicked the
outer desktop. Root Login/Users lookup and initial provider admission still assert.

A successful child LAUNCH returns an observation receipt. LAUNCH_STATUS reports
only that requesting endpoint's latest successful launch, tied to its incarnation
and the exact kernel task handle. Counters are per observer. No PID, fault details,
other caller's launch count or later occupant of a reused slot is disclosed.
Requester teardown clears observation authority. Terminal task collection wakes
the observer. Start at [children.rs](source-rs/distribution/matrix/service/children/children.rs)
for admission/placement, [lifecycle.rs](source-rs/distribution/matrix/service/children/lifecycle.rs)
for outcome/cleanup, and [launches.rs](source-rs/platform/libraries/matrix/world/launches.rs)
for the bounded receipt policy. Rust mod.rs remains plumbing.

Surface-allocation or placement refusal now stops and refunds the just-admitted
child instead of reporting an invisible success. Deliberately headless policy
remains valid. A failed LOSS send returns a refusal rather than killing the matrix.
More generally, a broken control direction is not proof of process death: owner
probes and close handling check a nonblocking kernel task wait and preserve a live
process with BUSY. Terminal outcomes are retained until cleanup so they are consumed
only once. Explicit owner STOP remains distinct authority. Pending close can lose
a race to owner STOP, and a task can exit during a recursive loss survey without
leaking its slot; deferred disconnects remain remembered until cleanup.

Verification:

- Workspace host tests passed earlier in this milestone; all 23 matrix policy
  tests passed after review, including receipt isolation, endpoint reuse, exact
  task matching and counter exhaustion. Userspace clippy and matrix startup-fault,
  Notes startup-fault/admitted-fault, desktop shell and storage-lab variants passed
  with warnings denied. All 32 storage-lab tests and ordinary clippy passed.
  Source layout, Rust formatting, Python compilation and diff checks passed.
- `lifecycle/20261018T223552633573Z` and
  `lifecycle/20261018T224234066969Z` passed four copied-image scenarios: malformed
  program, Notes death racing admission, Notes fault after a completed
  interaction round trip, and user-matrix death inside CONFIGURE. Each fails three
  times, then launches/saves/closes a healthy Colors. The user-fault case creates
  and enters a healthy second user from the unchanged template. Normal exit is
  checked against a delayed false crash notice. Source fixture bytes stay intact.
- Earlier lifecycle failures were checker assumptions: panic location and message
  span separate lines; the avatar color was wrong; and an early task fault can
  legitimately return admission refusal if it wins storage binding. Separate early
  and post-admission fault gates now verify both behaviors. Fault-only features
  are absent from ordinary F5; the copied fixture alone receives injected programs.
- `--matrix-storage-test --release` passed repeated placement rollback, denied
  foreign/stale receipts, monotonic own receipts, reuse by an owner-launched task,
  closed control-request and control-reply directions with a surviving task,
  uncertain owner health probes, owner STOP during pending CLOSE, and exit during
  LOSS followed by successful replacement. Existing storage scope/recovery checks
  also pass. `--matrix-test --release` passed recursive attenuation, dirty/delayed
  loss protection, clean subtree replacement and recovery.
- `desktop/20261018T224335105991Z` passed the normal F5 composition: user creation,
  Notes editing/duplication, save/discard/cancel, app selection and cold persistence.
  The independent host oracle verified all 247 records, exact hierarchy/history/
  quota and the empty other root. No interactive disk or running QEMU was changed.
- `colors/20261018T224602610580Z` rechecked the final build on a copy of the
  retained two-user interactive disk. Upgrade/collection, login, Colors save,
  close/reopen and cold boot passed; both app launches took about 0.58 s. This
  keeps the F5 quota repair covered as the stock executables change.

No kernel ABI or authority boundary changed, and no owner decision was required.
Kernel/ARM64 suites were not rerun. The broader lifecycle queue remains open:
restart a failed configured launcher within its own matrix, recover root startup
failures, and offer explicit recovery for an unresponsive app without silently
throwing away work. Previous storage capacity, ATA timeout and quiet fault-gate
limitations remain as recorded above.

## F5 already-running diagnosis (2026-10-18)

Another reported F5 failure had an existing interactive QEMU (PID 66056 at
diagnosis) still holding `cathedral-storage.raw`. A read-only exclusive Windows
open returned sharing violation 32. The default harness now checks availability
before compilation or replacing boot files, and tells the user to save/close
the previous QEMU and retry. It leaves that guest running. The probe is not a
reservation; installer/QEMU locks remain authoritative. Build-only and disposable
tests remain available while the interactive disk is busy.

Six database-image harness tests passed, including a real Windows sharing
conflict that refuses before any build and succeeds after the handle closes.
The exact F5 command now exits immediately with the actionable diagnostic.
Python compilation, layout and diff checks passed. No Rust code changed.
`f5-diagnosis/20261018T225227024191Z` reused the last verified default build on
a disposable copy of the current two-user disk: upgrade, picker, login, Colors
save/close/reopen and cold boot passed. Picker readiness was 3.62 s and 9.33 s.
The source disk hash was unchanged. The existing interactive QEMU and its boot
files were untouched; closing it is still necessary before the next F5 launch.

## Local configured-launcher recovery (2026-10-18)

Successful CONFIGURE now arms one local retry of its launcher after a fault or
nonzero return. The child task's actual outcome decides eligibility; a clean
return, explicit STOP or owner policy change does not restart it. Collection
releases the old launcher before replacement. The same asynchronous HEAD/BULK
path used by normal launches reloads its executable without blocking interaction
on the separate parent stream or retaining an extra code reader indefinitely.

Start at [recovery.rs](source-rs/distribution/matrix/service/children/recovery/recovery.rs).
It snapshots current placement, page policy and storage identity, waits for old
deferred events/control replies/publication to retire, then admits fresh endpoints
and issues fresh storage/layout grants. The app's task, scene, geometry epoch,
focus and document remain intact. No launch receipt transfers to the replacement.
It does not reload Matrix configuration or reinterpret an object number in a new
realm. Parent display generation, storage connection/realm incarnation, explicit
owner changes and new slot occupants prevent stale policy from reviving grants.
A loss survey also reports uncertainty if recovery becomes pending during it.

The retry is consumed on failure too. There is no timed retry reset, manual
launcher restart UI or recovery of the dead launcher's own unsaved Settings
draft. Reboot/configure can start a new session; live apps can still save and
close after retry exhaustion. Initial root Login/Users lookup and provider
startup remain the next lifecycle frontier, followed by explicit operator
recovery for unresponsive apps and an exhausted launcher.

Verification:

- Workspace host tests, all 25 matrix policy tests, all 32 storage-lab tests and
  ordinary storage clippy passed. New policy tests preserve app scene/epoch/focus/
  queued input while replacing the launcher and reject ancestor revoke/regrant
  or capacity changes. Userspace clippy and trace/startup/fault/owner-probe feature
  sets passed with warnings denied. Layout, formatting, Python and diff checks pass.
- `launcher-recovery/20261018T230048782616Z` passed five copied-image scenarios:
  dirty Notes survives launcher fault/replacement, replacement changes live layout,
  second fault stays stopped, Notes still accepts input/save/app-owned close,
  and its exact rendered text survives cold boot. Separate cases verify clean
  launcher exit, two immediate startup faults bounded to one retry, real routed
  owner STOP, and owner storage revocation before a fault. Notes loads once per
  boot; source fixture bytes are unchanged. An earlier three-case run also passed.
  The chooser fault keys and desktop owner probes are absent from ordinary F5.
- `desktop/20261018T230320916031Z` passed normal user creation, app selection,
  Notes duplication, save/discard/cancel, replacement and cold persistence. The
  independent host oracle verified 249 records and exact hierarchy/history/quota.
- `--matrix-storage-test --release` and `--matrix-test --release` passed after
  the loss-survey review, covering ordinary async launch, receipt scopes,
  control-channel failures, cleanup, recursive loss protection and delegation.
- `colors/20261018T230536053670Z` passed upgrade/login/Colors save/reopen/cold
  boot on a copy of the actual two-user disk. Upgrade collected unreachable
  history and charged 1,046,168 of 1,048,576 allowed bytes. Collection stops once
  the transaction fits, so this charge does not establish the live-content floor:
  it can still include unreachable history. Launches took about 0.58/0.59 s.
  Growing live content may reach genuine capacity; collection is not unlimited.
- Desktop, Colors and the new recovery fixture now build identical compositions
  into separate ESPs, avoiding replacement of an active F5 guest's boot files.
  The interactive disk and old QEMU were left untouched. No kernel ABI changed;
  kernel boot/ARM64 suites were not rerun for this userspace milestone.

## Independent Colors application (2026-10-18)

The installed Colors program now starts at
[applications/colors/main.rs](source-rs/distribution/applications/colors/main.rs),
then its named session entrance. Previously it was another role of the desktop
startup executable, so every owner change also enlarged the stored app. Its
trimmed release image is now 45,288 bytes versus 78,104 before extraction.
The profile installs it at the same template position, and normal stock refresh
preserves existing user object identities, metadata and saved colors.

The recursive desktop lab links the same Colors library, selecting its probe
loop at compile time and supplying the lab-owned command handler. Ordinary
Colors selects the app loop with its close button; it contains no provider
startup. Shared state/surface behavior remains in the application, with probe
composition and assertions under the desktop lab. No kernel ABI changed.

Verification:

- Workspace host tests, userspace clippy, desktop database-probe and shell trace
  feature checks passed. Layout (35 Omega/92 Rust package edges), formatting and
  diff checks passed.
- `colors/20261018T231229384277Z` upgraded a copy of the retained two-user disk,
  selected user 2, saved/reopened Colors and verified its color after cold boot.
  Launches took 0.407/0.438 s. The source interactive disk was untouched.
- `--matrix-desktop-test --release --timeout 180` passed the shared app's nested
  rendering, physical input/focus, quotas, storage/provider/subtree recovery and
  persisted colors after a fresh QEMU process. The repeat captured an explicit
  successful harness exit after the earlier process handle was unavailable.

Next: root startup still asserts on missing Login/Users or rejected picker
admission. Give that failure a visible, manually retryable distribution screen
that does not depend on loading another program from the broken store. Provider
startup failure and recovery controls for unresponsive apps remain separate
unfinished work. No ARM64 boot was rerun for this userspace extraction.

## Visible root login recovery (2026-10-18)

F5 now shows **SIGN-IN UNAVAILABLE / TRY AGAIN** if root Login/Users lookup
fails, the stored picker cannot be admitted, or the picker dies. Start at
[session/session.rs](source-rs/distribution/desktop/session/session.rs), then
[shell/login.rs](source-rs/distribution/desktop/session/shell/login.rs) for the
Picker/Recovery/Stopped phases. The separate
[recovery executable](source-rs/distribution/desktop/recovery/main.rs) is bundled
in the boot catalogue, so showing a failure does not require another successful
program read from storage. Ordinary picker and app code still come from storage.

The recovery child has local matrix drawing/input, no bound storage realm,
no devices and no descendants. The catalogue path currently supplies an unbound
storage endpoint which it ignores. It coalesces a button click or focused Enter
into one request on its owner-only control link; only the startup owner retries.
The owner keeps the screen through path reads and retires it before admission.
Repeated failure returns to the screen, with no automatic retry loop, disk
formatting or names/program repair. A dead recovery child does not itself trigger
admission retries. The path and program-read client deadlines now outlast the
matrix's storage verdict, rather than panicking at the old three-second limit.

The selected user continues to own the whole desktop, and the login child is
stopped before user CONFIGURE. Storage replacement restores a picker realm only
in the Picker phase; it never grants storage authority to the recovery screen.
This does not handle failure of initial display, input, root or storage provider
startup, which still precedes this phase and is the next lifecycle frontier.
Unresponsive apps and exhausted configured-launcher recovery still lack manual
operator controls. No owner semantic/trust decision was needed for this change.

Verification:

- `startup/20261018T232825283690Z` passed missing Login, missing Users, invalid
  Login ELF, picker startup panic and a one-time injected admission refusal.
  Each persistent failure accepts three mouse/keyboard retries and stays alive;
  the transient case returns to the real picker, enters a user and saves,
  closes/reopens Colors. Fault features are absent from the default composition.
- Each failed guest is cold-compared against its prepared snapshot: all tables,
  records, volume identity and guards are unchanged. Engine bookkeeping changed
  55 physical bytes. The new preservation oracle also rejected deliberately
  changed hierarchy records and guard bytes in negative controls. Source fixture
  bytes remain unchanged. Early checker failures were a miscounted title width,
  cursor occlusion and assuming the whole engine image stayed byte-identical;
  none required relaxing the record-preservation check.
- `desktop/20261018T232935589889Z` passed the normal F5 flow, app-owned close,
  save/discard/cancel and cold persistence. Its independent host oracle verified
  all 233 records and the exact hierarchy/history/quota. An earlier run passed too.
- `colors/20261018T233104082301Z` passed a deliberately lost save reply, provider
  replacement, explicit realm restoration, save reconciliation, reopen and cold
  persistence. The live user was retained and the retired picker was not revived.
- Workspace host tests, all 32 storage-lab tests, userspace clippy, shell/probe/
  trace feature checks and host-engine clippy passed. Layout (35 Omega/96 Rust
  edges), source formatting, Python compilation and diff checks passed. The two
  changed host helper leaves were formatted; the host lab's broader formatting
  check reports pre-existing drift in unrelated include files.

Interactive QEMU and its disk/ESP were untouched; all GUI checks used disposable
copies and separate ESPs. No kernel ABI changed. ARM64 boot was not rerun.

## Interactive storage-provider startup (2026-10-18)

The default owner now admits display, the root matrix and physical input before
opening storage. The boot-bundled screen shows **STARTING...** while
[persistence/startup.rs](source-rs/distribution/desktop/session/persistence/startup.rs)
polls the private readiness reply. The exact bytes must be `ready`; a matching
length is insufficient. Process death, a malformed reply or the 3,000-tick
readiness timeout collects that attempt and enables **TRY AGAIN**. Each click
authorizes one attempt; initial failure does not start an automatic retry loop.
There is still no formatting or repair fallback.

[provider.rs](source-rs/distribution/desktop/session/persistence/provider/provider.rs)
owns one task, its control endpoints and pending readiness deadline. Both startup
and runtime replacement use it, so partial connection/readiness failure closes
the old stream and collects the old task before reusing its launch grant. The
session explicitly has no provider while offline. Lab setup/runtime replacement
retain their bounded synchronous readiness wait; the normal startup path polls.
Successful manual retry does not replenish the three automatic replacements
allowed later in the session. Live users are not eligible for startup recovery.

The recovery program's distribution-private STATUS command changes its card.
The owner revokes queued input and restores a fresh lease around that transition;
the program preserves observed button state, so holding over Retry through a
timeout does not request another attempt, and a later fresh click needs one press.
The recovery child retains no storage realm. A 64-page allowance accommodates its
32-page image and replacement; it is collected before the picker is admitted.

Verification:

- `provider-startup/20261018T234651309751Z` passed all five final cases: crash
  before opening, malformed five-byte readiness, a live pre-open stall through
  the normal timeout, repeated crashes, and two invalid volume headers. The stall
  kept physical pointer rendering responsive while a button stayed held over
  Retry; release/fresh click recovered into the real picker and a usable Colors
  that saved, closed and reopened. Repeated failures each required a fresh click
  with the cursor left in place. Crash/corrupt-header attempts left the complete
  disk byte-identical, and the source fixture stayed untouched. Fault features
  are absent from ordinary F5. Earlier checker failures had incorrect timeout
  and error-text expectations; the guest had correctly reached recovery.
- `startup/20261018T234529797906Z` passed all existing missing-path, malformed
  executable, picker-crash and successful manual login-retry cases. Its host
  oracle confirmed every record, volume identity and guard unchanged for failed
  attempts. This preceded the final button-state refinement, which the provider
  suite specifically exercises.
- `desktop/20261018T233828976820Z` passed the normal F5 flow and cold persistence
  after the startup reordering, with all 233 records checked by the host oracle.
- `colors/20261018T235149916300Z` passed a deliberately lost save reply, provider
  cancellation/collection, replacement and explicit grant restoration. The live
  user reconciled its save, reopened the app and retained its color across cold
  boot. This exercises the shared provider lifecycle after the startup refactor.
- `run.py --matrix-desktop-test --release --timeout 180` passed the recursive
  desktop's storage stalls, provider replacement, tree recovery, input confinement
  and cold persistence. The earlier completed run had no recoverable process
  result, so this final run supplied an explicit successful exit and both PASS
  reports. Artifacts are under `matrix-desktop-test/`.
- Workspace host tests, userspace and affected shell/lab/provider clippy feature
  checks passed. The two restart-budget tests run through the new
  `recovery_policy_test.py`, which supplies actual contract constants and an
  isolated host unwind build; the previous documented standalone command was
  stale and lacked the contract dependency. Layout remains 35 Omega/96 Rust edges.

Initial display/input/root admission failures remain uncovered, as do explicit
controls for a live unresponsive app or exhausted launcher recovery. No kernel
ABI or driver authority changed; no owner-level semantic decision was required.
The active interactive QEMU and its disk/ESP remain untouched. ARM64 boot was
not rerun for this distribution lifecycle change.


## Bounded early bootstrap recovery (2026-10-18)

[Admission](source-rs/distribution/desktop/session/admission/admission.rs) now has
an explicit partial-attempt lifetime and named display, root/funding and input
steps. Its F5 startup boundary permits two attempts total before storage or any stored user is admitted. A failed
stage collects input/root/display in that order, closes endpoints still held by
the owner and restores root funding. Successful endpoint moves are removed from
the partial-attempt inventory immediately. Input readiness now has a 300-tick
bound and checks exactly `ready`; root readiness must decode to an all-zero HEALTH
reply, and display INFO must decode successfully with supported nonzero extents.

Every rollback compares the owner's available process slots, physical reservation
and charge, page/channel limits, private/reader charges and dynamic-handle charge
with its pre-attempt snapshot. Root page/channel allowances remain in that idle
account: combining the full allowance with the owner's would exceed its individual
cap. Root process slots and physical reservations are reclaimed, then explicitly
re-funded for the next attempt. These checks use existing syscalls; no kernel ABI
or device authority changed.

Exhaustion logs the stage (display 0, input 1, root 2) and error. For root/input
failures, the owner re-admits display alone and renders **COULD NOT START / PLEASE
RESTART**, then blocks. This requires no matrix, input, storage or raw framebuffer
access by the owner. Display failure itself cannot provide graphical recovery;
the terminal reports it and the initial program returns. No user session is torn
down by this path, and no storage is opened, formatted or repaired.

The fault suite injects pre-initialization task exit, a blocked readiness reply,
malformed readiness, and admission refusal after staging but before endpoint
moves. Features are absent from the stock profile. These are not exhaustive
hardware-initialization or live-driver-replacement tests.

Verification:

- All 24 bootstrap cases passed across the recorded runs. Under `bootstrap/`,
  `20261019T000429090327Z` passed all eight display cases and transient input
  death, then caught the restart message exceeding the display protocol's
  16-character text limit. The corrected **PLEASE RESTART** screen passed repeated
  input/root death in `20261019T000742913873Z`; `20261019T000822525080Z` passed the
  remaining thirteen cases. Every transient failure reached a real user and a
  Colors app that saved, closed and reopened. Repeated failures stopped after
  two attempts, returned the measured budgets, and left the entire guest storage
  image byte-identical. Source fixtures were unchanged. Root/input terminal
  screens were checked from scanout; one was also inspected visually.
- `desktop/20261019T000008384217Z` passed the ordinary F5 user/app flow, app-owned
  close and save/discard/cancel, cold persistence and the independent oracle's
  exact 233-record check. This preceded the final diagnostic text correction and
  function relocation; the later fault cases exercised the final startup path.
- `run.py --matrix-desktop-test --release --timeout 180` passed with the final
  admission layout: recursive isolation, provider/tree recovery, physical focus
  and both saved colors after cold boot. This covers the catalogue labs sharing
  the new admission, input readiness and cleanup code.
- Workspace host tests, userspace clippy, shell/probe/trace and catalogue/database
  feature checks passed. Layout (35 Omega/96 Rust edges), source formatting,
  Python compilation and diff checks passed. No kernel ABI changed; ARM64 boot
  was not rerun for this distribution-level change.

Next: explicit operator controls for an unresponsive app or exhausted local
launcher recovery. Initial recovery is bounded now; live display/input/root loss
still needs a separate policy that accounts for unsaved user work. The active
interactive QEMU and its disk/ESP remain untouched; all tests use separate media.

## Explicit app termination, 2026-10-18

The F5 launcher now offers END APP above Settings while its delegated app slot
is occupied. F3 with launcher focus opens the same warning. Cancel is the default
keyboard choice; a fresh confirmation cancels the observed app/subtree without
requiring cooperation from a broken app. Ordinary app X and sibling CLOSE retain
their loss-protection rules. The existing three tile surfaces render the warning
and choices; the optional footer stays within the launcher's current page grant.

The matrix owner can grant/revoke management independently of launch receipts.
The ledger binds authority to the authenticated endpoint incarnation; queries
issue endpoint-local tickets for exact kernel child handles. There is no PID or
caller-supplied slot selector. Regrant, revocation and task retirement invalidate
tickets; counters reset only when the manager connection dies. CONFIGURE and its
bounded launcher restart explicitly grant the selected launcher management of
the app slot. An explicit owner policy change cancels automatic recovery, so a
restart cannot restore revoked authority. Teardown uses the existing task,
surface, endpoint and funding cleanup. No kernel ABI or stored schema changed.

Verification:

- `management/20261019T002724060034Z` passed original-launcher, recovered-launcher
  and owner-revocation scenarios. Physical input exercised both Cancel actions,
  forced termination of a non-yielding dirty Notes task, replacement Notes with
  only saved content, Colors save, and two cold reboots. Real IPC denied app
  self-grants, management on app/storage endpoints, retained tickets after target
  replacement, and retained tickets after owner revocation. Revocation also
  cancelled launcher auto-recovery while Notes kept its draft and could save/close.
  The full confirmation was checked in scanout and visually inspected. The source
  fixture was byte-identical afterward. Earlier two-mode evidence is under
  `management/20261019T002509885704Z`.
- `desktop/20261019T002848038329Z` passed the unchanged default F5 composition,
  user creation, app-owned save/discard/cancel, app replacement, cold persistence
  and the host oracle's exact 235-record check.
- `run.py --matrix-storage-test --release --timeout 180` passed existing recursive
  delegation, scoped launches/receipts, failed control links, owner-stop/close
  races, loss surveys and replacement. These exercise the shared cleanup path;
  the new force-stop-versus-pending-control interleavings are not exhaustively
  injected. Active storage/publication and loss survey gates remain bounded by
  existing deadlines; BUSY needs another explicit confirmation.
- Workspace host tests, userspace clippy and changed fixture-feature clippy passed.
  Layout (35 Omega/96 Rust edges), format, Python compilation and diff checks pass.
  No kernel or architecture changes; ARM64 was not rerun.

This is ordinary manager authority with a distribution confirmation, not proof
of human consent or an unspoofable system escape. The owner question about minimal
trusted platform chrome remains open; its outdated description of the Rust
status-app experiment has been corrected. Next: explicit recovery after launcher
retry exhaustion, then live provider-loss behavior without discarding app drafts.
The user's existing interactive QEMU, saved disk and ESP were left untouched.

## Explicit launcher recovery after exhaustion, 2026-10-18

The user matrix now replaces an exhausted launcher with the distribution's
LAUNCHER UNAVAILABLE / TRY AGAIN screen. The screen occupies the old launcher's
view; the app keeps its task, surfaces, current arrangement and unsaved work.
Clicking Try Again or pressing Enter with that view focused requests one new
stored-program read and admission. The acknowledgement precedes retirement of
the requesting screen. A successful replacement gets fresh layout and app
management authority, but no replenished automatic retry. Repeated failure
returns to the screen; its own exit or fault stays stopped.

The platform does not select distribution code. LAUNCH_IMAGES explicitly lends
existing executable grants to future store-based admissions, with none by default.
The startup owner lends its recovery executable to the user matrix and supplies
LAUNCHER_FALLBACK before CONFIGURE. The fallback has local interaction and a
connection-bound RETRY_LAUNCHER request, without storage, layout, task management
or descendant capacity. Kernel template admission retains its own code grant;
it does not grant capacity to spawn anything. Ordinary stored apps still receive
no executable grants. No kernel ABI, resource ceiling or stored schema changed.

Changing the owner's fallback policy cancels configured recovery, including a
queued request. Existing owner placement/storage/management revocation rules
also cancel recovery. This is an ordinary delegated recovery UI, not physical
human attestation or a protected global escape. It requires a live matrix and
its upstream services; live provider loss remains separate work.

Verification:

- Launcher cases passed across `launcher-recovery/` runs:
  `20261019T004036309505Z` completed recover (including cold boot) and clean-exit
  cases before a startup screenshot assertion failed. That assertion used the
  wrong glyph coordinate; corrected startup passed in `20261019T004252229040Z`.
  Recover exercised both pointer and keyboard retries, retained live layout and
  a dirty Notes draft, and saved/closed without restarting Notes. Startup faults
  proved each click permits exactly one additional attempt, without an automatic
  loop. `20261019T004341827568Z` completed owner-stop and owner-revocation cases
  before a fallback screenshot check caught the cursor covering its text.
  Moving the cursor away fixed the check; fallback clean-exit and fault cases
  passed in `20261019T004519997743Z`, with Notes still editable/saveable/closable.
  Successful subset runs confirmed the source fixture was unchanged.
- The initial fallback assertion incorrectly expected no code grant at all;
  kernel template admission intentionally includes the template's own code.
  The corrected runtime assertions verify no extra code and zero child capacity,
  plus denied storage/layout/management operations. No kernel change was needed.
  Read/admission refusal paths lead to the same fallback but were not separately
  fault-injected in this milestone.
- `management/20261019T004803880775Z` passed all three existing scenarios,
  including force-ending a hung app, both Cancel actions, stale/revoked authority,
  recovered launcher management, replacement apps and cold persistence. Notes
  now also asserts denial of fallback retry/policy and executable delegation on
  its interaction/storage endpoints, and absence of inherited executable grants.
- `desktop/20261019T004939308363Z` passed the normal F5 user/app flow and cold
  persistence, with the independent oracle checking all 237 records. The picker
  was ready 2.48 seconds after QMP connection on the first boot and 4.30 seconds
  on cold reboot. These timings exclude compilation/installation; they do not
  diagnose the owner's separate delayed-window report.
- `provider-startup/20261019T005026283061Z` passed `crash-once`: the shared
  distribution screen still handles failed provider readiness and explicit
  retry through to a usable session. `run.py --matrix-storage-test --release
  --timeout 180` passed recursive delegation, scoped launches, loss handling,
  owner stop/revocation and replacement through the shared admission/cleanup path.
- Workspace host tests, userspace and changed feature-set clippy passed. Layout
  (35 Omega/96 Rust edges), source formatting, Python compilation and diff checks
  passed. No kernel/architecture changes; ARM64 was not rerun.

Next: live provider-loss behavior that preserves app drafts. The trusted-chrome
owner question remains open. The user's interactive QEMU, disk and ESP remain
untouched; fault tests use disposable copies and separate boot media.

## Live storage loss with app drafts, 2026-10-18

F5 now polls replacement-provider readiness through
[replacement.rs](source-rs/distribution/desktop/session/persistence/replacement.rs).
It no longer blocks the startup owner for the provider's 30-second recovery
deadline. Initial login readiness and live replacement are distinct phases;
replacement success restores grants to the current user and its remembered
child realms without restarting the picker, launcher or app. The old provider
and its control endpoints are collected before a fresh attempt. Failure leaves
storage offline and the existing user processes alive. The three-replacement
session budget and six-second operation deadline are unchanged; the older lab
desktop keeps its synchronous replacement contract.

The new [live-provider fixture](tools/boot-harness-rs/provider_live_test.py) copies
a one-user default-layout image and uses the actual F5 composition. Opt-in owner
probes arm the storage service's existing fault gate only after a dirty note is
visible. No fault commands or probe features ship in the ordinary profile.

Verification:

- `provider-live/20261019T005500307323Z` passed precommit and postcommit stalls,
  including cold boot. Notes retained its draft and task, then F1 reconciliation
  correctly distinguished an uncommitted write from a committed lost reply.
  Subsequent editing/saving/closing worked and exact note pixels survived cold
  boot. Neither case restarted login; the source image was unchanged.
- `provider-live/20261019T005609024283Z` passed exhausted retries and blocked
  replacement readiness, both with cold boot and unchanged source media. Four
  precommit failures consumed exactly three replacement attempts. A provider
  that never answered readiness was collected at the existing deadline. Notes
  stayed editable while waiting and offline; Cancel retained its draft and a
  failed Save in the close dialog kept the dialog/process alive. Cold boot read
  the original saved note, proving the failed precommit writes were not published.
- `desktop/20261019T005819433872Z` passed the default F5 flow and cold persistence;
  the independent host oracle verified all 237 records, including the copied
  user, duplicate notes and saved color.
- `matrix-database/20261019T005930291094Z` passed the older recursive recovery
  lab: stalled-provider watchdog, uncertain CREATE reconciliation, precommit
  and lost-reply replacement, live sibling progress and cold persistence. The
  host oracle verified exact revisions `[4, 2]` and all 16 records on both boots.
- Workspace host tests, userspace clippy, shell and combined live-probe feature
  clippy, restart-budget policy tests, layout (35 Omega/96 Rust edges), formatting,
  Python compilation and diff checks passed. This changes only distribution
  supervision and its fixtures; no kernel, ABI or stored schema changes.

Remaining limits: this proves storage-provider failure, not display/input/root
replacement. Offline Notes keeps failed saves and close attempts visibly failed;
there is still no delegated live-session control to retry storage once automatic
recovery is exhausted. The existing protected-chrome owner question remains
separate from ordinary delegated recovery. Failure while restoring grants and
simultaneous provider/launcher death are not exhaustively injected here.

## Stock matrix and reusable platform boundary, 2026-10-18

The stock matrix is now [a distribution executable](source-rs/distribution/matrix/main.rs),
`cathedral-stock-matrix`. Its launcher/app roles, two-child owner router, sibling
launch/close requests, saved strip arrangement and configured-launcher recovery
are optional stock policy. Their [extension contract](source-rs/distribution/matrix/contracts/matrix.rs)
and unchanged configuration codec live beside that program. The normal profile,
focused profiles, installer and fault builds use the new package. Wire operation
numbers and saved configuration bytes are unchanged; there is no data migration.

[The platform library](source-rs/platform/libraries/matrix/README.md), `cathedral-matrix`,
retains reusable authority and geometry. `World<N>`, storage scopes, launch
observations and management ledgers take independently selected child capacities.
Layout accepts owner-supplied rectangles; no slot is intrinsically a launcher.
Surface ranges may be explicitly selected or allocated from free space, cannot
overlap, and cannot move while live. Pointer hit testing follows those ranges.
Replacement snapshots retain their original range without changing siblings.
The stock executable explicitly chooses its previous bottom/top sibling ordering.

This is not yet a general-purpose matrix server runtime: the existing IPC loop,
admission orchestration and forwarding adapters remain in the stock executable.
Another distribution can use the platform mechanisms and contracts with its own
program; it does not yet get a separately packaged configurable server loop.
The new host proofs exercise three visual children (last slot controlling layout),
arbitrary owner stacking, four-child headless ledgers, and zero visual slots.
These prove removal of the two-slot/desktop assumptions from shared state, not
that a phone or embedded distribution has been built. Fixed protocol resource
bounds remain. Kernel implementation and ABI are unchanged.

Architecture checks now reject platform/kernel imports of the stock contract,
and reject implementation dependencies from that contract. Reading paths and
package maps were updated across executable/IPC seams; `mod.rs` stays plumbing.

Verification and findings:

- Workspace host tests passed, including 30 shared matrix tests and the moved
  configuration codec test. Userspace clippy and stock matrix fault/trace feature
  clippy passed, as did the shell/live-probe feature set and restart-budget tests.
- `desktop/20261019T011839687704Z` passed the default F5 flow and cold persistence;
  the independent oracle checked all 235 records. This preceded the explicit
  stacking correction described below.
- `launcher-recovery/20261019T012031252420Z` passed recover, startup and
  owner-revocation cases, including live layout, automatic and explicit retries,
  retained app draft, save/close and cold persistence. Source media was unchanged.
- The recursive desktop test caught an introduced stacking regression: first-fit
  range allocation made launch order determine overlap order. The fix makes the
  owner's stacking choice explicit, tests arbitrary order and replacement on the
  host, and keeps the stock bottom/top policy in the distribution. The recursive
  desktop then passed nested pixels, focus, lifecycle recovery and cold colors.
- The management fixture initially exhausted the 4 MiB engine before boot while
  upgrading old code and installing two probes. Its preparation now compacts its
  disposable copy before each injection; it never changes source/interactive media.
  Compaction alone was insufficient: the host injector rewrote every unchanged
  record too. It now inserts only changed records, preserving unrelated pages.
- `management/20261019T013555980484Z` then passed all three scenarios on the
  final code: hung app termination, both Cancel actions, stale/ungranted authority,
  recovered launcher management, replacement apps and cold persistence. The
  source fixture was unchanged. Its successful setup also exercises the corrected
  host injector on an upgraded image, with both fault programs installed.
- Two recursive storage runs reached an ATA FLUSH timeout (`op=e7`, status `c0`,
  50 ticks) and the provider's existing engine-failure panic. The second ran after
  host tests finished. This remains the tracked ATA completion issue, not a pass;
  no storage driver/deadline changes are part of this boundary cleanup. Logs are
  `build/boot-harness-rs/matrix-storage-timeout-20261019.log` and
  `build/boot-harness-rs/matrix-storage-test/serial.log`.
- The later default-desktop run `desktop/20261019T013226419025Z` hit the existing
  ATA WRITE completion timeout before login (`op=30`, LBA 24, status `d0`, 50
  ticks). The stock recovery screen appeared; this is not a passing desktop run.
- Layout checks passed (35 Omega/106 Rust package edges), including six checker
  unit tests. Named-body formatting, changed Python compilation, Markdown links
  and diff checks passed. The host installer checks with its locked engine feature
  set. ARM64 was not rerun: this changes no kernel/architecture implementation.

Follow-up architecture proof: compose a small independently owned headless or
alternative-layout matrix program, then extract the shared IPC orchestration it
actually needs. Do not move stock config/recovery policy back into platform or
invent a universal desktop framework. The outstanding delegated live-storage
retry and protected-chrome owner question remain separate work.

## Independent matrix over real IPC, 2026-10-18

The [alternative matrix](source-rs/distribution/lab/matrix-composition/matrix/main.rs)
is a separately compiled program with three equal virtual input regions. It has
no stock matrix, desktop contract, saved configuration, storage or device grant.
It composes `World<3>` with the existing runtime `Server`; the latter already
handles more than two peers and orders disconnection before reconnection.
The [proof](source-rs/distribution/lab/matrix-composition/README.md) admits this
program and three isolated peer processes through explicit links. It checks
local-only views, malformed packets, denied owner/source requests, input to the
third region, revocation, and replacement requiring a fresh grant while siblings
keep their state. The first bring-up used the supervisor role for the relays;
setting their explicit profile argument corrected that fixture wiring mistake.

[Local command dispatch](source-rs/platform/libraries/matrix/world/commands.rs)
now owns common placement/revocation and trusted-source input. Both programs use
it; F5 also uses it on the physical input connection. Endpoint authentication,
parent binding, stacking, configuration and launch policy stay with each program.
Children enter the separate restricted dispatcher. No protocol numbers, persisted
bytes, device deadlines or kernel implementation changed.

This closes the lack of a real independent IPC consumer for the local-world API.
It does not yet extract the stock admission, upstream, storage or rendering loops:
the test supervisor admits these peers as siblings. There is no need to invent
another generic server abstraction for the subset that the runtime already serves.
The source-layout check now rejects all distribution dependencies from this proof's
packages, so adding a stock contract cannot quietly invalidate the evidence.

Verification:

- `python tools/boot-harness-rs/matrix_composition_test.py` passed in
  `matrix-composition/20261019T021047276260Z`, using private firmware/ESP and no disk.
- `run.py --matrix-test --release --timeout 120` passed the stock recursive
  matrix's locality, input, teardown and recovery checks after shared dispatch.
- `desktop/20261019T021239109327Z` passed the default F5 user/app flow and cold
  persistence. The independent host oracle verified all 235 records. This run
  had no ATA timeout; it does not resolve the previously recorded occurrences.
- Workspace host tests and `cargo check-userspace` passed. Layout checks passed
  (35 Omega/113 Rust edges) with seven checker tests; named-body formatting and
  diff checks passed.

The prior ATA completion failures and live-storage retry gap remain unresolved.
The next matrix extraction should target independent child admission or upstream
adaptation, with both actual consumers preserving their distinct policy.

## Shared child admission and independent subtree, 2026-10-18

[Runtime admission](source-rs/platform/libraries/user-runtime/matrix/admission/README.md)
now owns channel construction, startup-endpoint transfer, child spawn and control
connection acceptance. It closes all endpoints on pre-spawn failure, or cancels
and collects the child before closing retained parent ends on control-connect
failure. Successful admission returns the task and retained connections; callers
choose installation, authority grants, teardown and reservation release. There is
no implicit Drop cancellation. The stock matrix uses it for template, catalogue
and stored-object admission, preserving its budgets and recovery policy. The
runtime matrix folder has a named entrance and plumbing-only `mod.rs` files.

The independent matrix now receives three launch loans and admits its own
children. Its supervisor has neither their launch authority nor their control
ports while the matrix is alive. Children receive the normal three-endpoint
matrix environment, no storage pair, no static peer links, and no launch/device
authority. The matrix's small asynchronous lab probe loop keeps serving children
while it waits for their control replies. Probe failure ends this test program;
that deliberately limited behavior is not a shared application recovery policy.

The QEMU proof checks, per matrix startup, sixteen busy-spawn failures after
channel construction and sixteen channel-budget failures after allocating two of
three required channels. Every failure preserves the original resource charges.
It also checks sixteen middle-child replacements, no inherited local grant,
unaffected sibling views, whole-subtree cancellation and a fresh matrix using the
returned launch loans. These prove actual custody/rollback, not just pure state.
The control-connect failure branch was preserved but not separately injected.

Verification:

- `matrix-composition/20261019T022726897765Z` passed owned admission, both rollback
  paths, local input, denied authority, replacement and subtree reclamation.
- `run.py --matrix-test --release --timeout 120` passed stock recursive admission,
  locality, input, teardown and recovery with the shared helper.
- `desktop/20261019T022923861143Z` passed the default F5 desktop and cold-reboot
  persistence; its host oracle verified all 237 records. No ATA timeout occurred
  in this run; the previously recorded failures remain unresolved.
- Workspace host tests, `cargo check-userspace`, layout (35 Omega/113 Rust edges),
  seven layout-checker tests, named-body formatting and diff checks passed.

No kernel ABI, stored schema, channel budget or device deadline changed for the
ordinary desktop. The independent profile funds eleven channels explicitly; this
leaves two spare with three children live and makes partial-allocation rollback
observable. Remaining matrix extraction targets are upstream adaptation and
rendering/storage forwarding; alternative dynamic-reservation policy is not yet
proved. The recorded ATA issue and live-storage retry gap remain separate work.

## Scoped application bootstrap, 2026-10-18

The owner-approved [bootstrap model](source-rs/platform/contracts/services/README.md)
separates manifest requests, launch-approved authority, and acquired connections.
Applications enter with `Services` and request interaction/storage before their
ordinary loops. The platform owns the protocol, client, endpoint custody and
admission mechanics; the stock matrix authenticates its child connection and
selects its approved offer. No PID, sibling number or requested realm in a packet
can choose another offer. There is no new kernel syscall or global service locator.

`CTHIMP02` opts sealed stored programs into broker startup. Older import records
and undeclared legacy code keep their existing bundles. The runtime compatibility
path also supports explicitly wired catalogue fixtures. The new environment has
two endpoints for one logical broker; existing kernel owner control is separate.
The current interaction service shares the broker RPC pair and acquires its wake
receiver. Storage acquires a separate pair in two endpoint-transfer replies.
Service and bootstrap versions are separate. Required imports still refuse launch
when missing/incompatible; unavailable optional ones remain absent.

Channels are allocated before spawn from existing budgets. Acquisition only moves
preapproved rights, each at most once. Repeat application calls cache bindings;
repeat wire requests cannot duplicate them. Unsent rights remain owned by the
broker and are closed on task teardown, including interrupted pair acquisition.
Ambiguous client transport failure closes the shared broker/interaction stream so
a late capability reply cannot contaminate a subsequent ordinary RPC. Acquisition
must be serialized with interaction; current apps acquire during startup.

The first F5 run (`desktop/20261019T033724803621Z`) found a real ordering bug:
notification preceded wake-receiver transfer, so the kernel refused the nonempty
queue. The stock owner now preserves pending notification state until that receiver
has transferred. Expected transfer capacity/busy errors return refusals rather
than ending the matrix. The later default desktop passed.

Verification:

- `matrix-composition/20261019T034140088220Z` passed acquisition, unknown/ungranted
  and version refusals, forged identity fields, repeated raw/cache requests,
  rollback, replacement and whole-subtree reclamation. Sixteen cycles interrupt
  acquisition before any service or after one storage endpoint. The third peer's
  storage endpoints test custody only; this headless fixture has no filesystem.
  Its thirteen channels fund eleven live channels plus two for partial rollback.
- `desktop/20261019T033908741405Z` passed ordinary F5 Notes/Colors behavior and cold
  persistence; the independent host oracle verified 239 records.
- `desktop_test.py --service-imports undeclared` passed in
  `desktop/20261019T034306507571Z`: a disposable Notes declaration omits storage,
  which stays absent even though the owner offers it. Editing, dirty-close and
  independent Colors persistence pass across cold reboot.
- `desktop_test.py --service-imports incompatible` passed in
  `desktop/20261019T034431375347Z` with optional storage version 2. Notes degrades
  to temporary storage; Colors retains its separate usable version-1 binding.
  These variants use visual/serial checks; the default test alone runs the full
  independent retained-record oracle. Interactive saved media was untouched.
- `run.py --matrix-test --release --timeout 120` passed legacy recursive matrix
  startup/locality/recovery. `--matrix-storage-test` passed existing scoped roots,
  recursive delegation, revocation, realm launches, siblings and replacement.
- Workspace host tests and `cargo check-userspace` passed. Notes clippy also passed
  startup-fault/admitted-fault/management/render/trace feature composition.
  Layout passed (35 Omega/113 Rust edges), with seven checker tests, and named-body
  formatting passed.

This is a bounded bootstrap, not arbitrary service enumeration, dynamic grant
addition or lazy provider startup. All service channels remain preallocated.
The stock broker path currently requires interaction and separate storage streams;
legacy headless multiplexing retains its prior protocol. Upstream/rendering
extraction, the recorded ATA completion issue and live-session storage retry
remain separate work. No disk schema, kernel ABI or default desktop budget changed.

## Visible alternative matrix, 2026-10-18

[Split](source-rs/distribution/applications/split/README.md) is installed under the
stock template and recognized existing users, with its Notes code beneath it.
The normal executable-only chooser discovers it by path/metadata. No demo-specific
launcher dispatch or boot catalogue grant is involved. It enters through Services,
uses its own storage binding to load code, validates sealed imports, and admits two
independent Notes processes with interaction-only offers. Both apps prove storage
is absent and cannot be reached through interaction. Their temporary drafts never
enter the object store.

The shared mechanisms now include `World::inherit_poll`, synchronous upstream
exchanges and retained image custody, alongside the existing admission, service
broker, geometry, surface naming and input rules. The stock parent consumes the
same snapshot/custody code, including its asynchronous presentation commits.
Split imports no stock implementation or desktop contracts; the layout checker
rejects such dependencies. Its named session entrance leads to children, requests,
rendering and loss leaves; mod.rs only wires them.

The enclosing matrix controls Split's placement; Split computes its children's
two columns from its local extent. It shares sealed image readers, never maps or
copies descendant pixels. An asynchronous LOSS survey continues serving children,
and silence/malformed replies prevent close. After a timed-out child control
exchange that stream stays uncertain until child death. Individual closes retire
surfaces before returning reservations; whole-program exit lets the kernel reclaim
its subtree. The visible proof repeatedly starts the same program after teardown.

### Lab funding and limits

The larger live subtree exposed several old ceilings. Funding is explicit:

- Root: 256 private pages, 384 reader pages, 28 channels and six dynamic slots.
  Display/input reserve no unused dynamic channels. Initial owner plus root still
  reserve only the existing 32-channel global pool.
- User: 160 private pages, 192 readers, 16 channels and four descendant slots.
  Its launcher receives 64 private pages; its app receives 80 private/64 reader
  pages, six channels and two descendant slots. This is stock distribution policy.
- Split lends each Notes 32 private/four reader pages, no channels or descendants.
  It retains bounded header images within its remaining sixteen private pages.
- Single objects remain capped at 128 pages; the separate private-budget ceiling
  is now 256. Per-task reader slots are sixteen, with an independently bounded
  global copied-reader pool of 128 (the IPC retirement bitmap remains u128).
- Compositor surface capacity remains sixteen. Login overlaps user admission with
  five picker/eleven user surfaces; after stopping the picker, the owner grants
  fourteen to the user. The launcher uses five, Split nine (four per temporary
  Notes plus its header). Temporary Notes no longer allocates two unused surfaces.

Named additions allocate fresh object IDs; original six-object bootstrap identities
remain valid. The installer receipt authorizes exact previous addition manifests,
not arbitrary programs with matching names. Custom entries block installation into
that path. Migration preserves saved content, identities, launcher choice and
geometry; only known old resource defaults change. It has a separate
`.before-realm-programs.raw` backup and unchanged installations do not reopen the
image. The independent desktop oracle now models both added programs and their
nested subtree copy.

### Remaining boundaries

Split is a bounded alternative implementation, not a general replacement desktop.
It does not forward child storage, accept presentation transactions, dynamically
choose children or restart them. Parent snapshot handling and bitmap custody are
shared; full transaction scheduling and root-display adaptation remain in the stock
matrix. Loss of upstream transport ends this subtree rather than recovering it.
Parent revocation/regrant is covered by the pure World test; live outer resize and
whole-subtree close are covered visually. No owner semantic decision is needed for
these implementation gaps.

Next useful work: finish extracting transactional presentation and scoped storage
forwarding when a second consumer needs them; diagnose the recorded ATA WRITE/FLUSH
completion timeout; then address live-session storage retry delegation. Keep the
F5 desktop as the ordinary way to use new capabilities. No authenticated login,
Warden key handling or encrypted realm unlock is implemented by this milestone.

### Verification

- Workspace `cargo test --locked`, `cargo check-userspace` and `cargo check-uefi`
  passed. After adding the final revocation/budget regressions, core's 108 and
  matrix's 31 tests passed again. Storage host tests passed all 34 cases, including
  named installation, exact receipt updates and preservation of custom programs.
- Harness Python tests (13), layout tests (7), layout check (35 Omega/118 Rust
  edges), named-body formatting and changed Markdown links passed.
- `desktop/20261019T064749690692Z`: ordinary F5 Notes/Colors and cold persistence;
  the independent host oracle verified all 300 records.
- `desktop/20261019T064900399844Z`: `desktop_test.py --nested` passed two cold boots,
  independent child focus/drafts, denied persistence, dirty-close, outer resize
  and three complete subtree launches/teardowns per boot.
- `labels-upgrade/20261019T065815917067Z`: `labels_upgrade_test.py --nested` copied
  the retained six-program desktop disk, preserved both saved notes and identities,
  installed Split, and passed the same nested interaction/reclamation checks inside
  the migrated user. An unchanged second installation left the disk byte-identical.
- x86 release smoke, ARM64 release portability, stock `--matrix-test` and
  `--matrix-storage-test` passed. Independent matrix composition passed in
  `matrix-composition/20261019T065431646805Z`, including interrupted acquisition,
  rollback and subtree reclamation.

All disk tests used disposable images or copies. The interactive saved disk was
not opened or modified. The previously recorded intermittent ATA completion issue
and live-session storage retry gap remain outstanding; these passing runs do not
claim to resolve them.


## F5 installation capacity repair, 2026-10-19

The actual saved desktop failed before QEMU started. In-place redb compaction
requested 4,296,704 bytes inside the 4,186,112-byte engine extent. Rebuilding alone
was insufficient: subsequent seeding also needed temporary allocation, and adding
Split crossed the logical quota while old unpinned history remained.

The host [install entrance](tools/storage-lab/engines/volume/install.rs) now
stages seed, stock-copy refresh and hierarchy/named additions together on a private
host scratch volume, with the original image locked. A content-named full backup
is synced first. Rebuilding uses separate scratch capacity, then enforces the
original extent and cold-compares every prepared record before publishing. A failed
preparation leaves the original disk and sidecar unchanged. Named additions now
collect unreachable history under quota pressure after staging their new heads;
current data and pins survive. Actual live/pinned excess still refuses.

There is no kernel, guest-volume-size or syscall change. This remains offline
image maintenance: publication errors attempt restoration, but process/power loss
during the final copy requires restoring the synced backup. Individual migration
backups printed inside the temporary directory are not durable recovery points;
the `.before-compact-<digest>.raw` beside the saved disk is the recovery point.

Verification:

- Storage host tests: 36 passed; storage host clippy passed with warnings denied.
  The new near-capacity fixture demonstrates that
  the old in-place compactor fails, the rebuild preserves all records, and failed
  edits/oversized results leave the original bytes unchanged. A named-addition
  quota test preserves current user content while collecting old revisions.
- Harness Python tests: 13 passed. Changed installation invokes one staged host
  operation; unchanged installation never opens the image; failed preparation
  never records success.
- `desktop/20261019T071938153217Z`: ordinary Notes/Colors interaction and cold
  persistence passed; independent host oracle verified all 300 records.
- `launch-repair/fixed-copy`: a copy of the exact failing saved disk completed
  installation and booted to the user picker (3.16 seconds after QMP connection).
  The normal `--release --window` command then upgraded the actual saved disk and
  opened QEMU. It retains the same extent; its full original backup is
  `cathedral-storage.raw.before-compact-872824a45ab20d65f4d9dfd3a1085b2523044e6c740aba827a93159319575b6c.raw`.
- `labels-upgrade/20261019T072105675488Z`: existing-user saved-notes compatibility
  passed. Its subsequent `--nested` pass failed at live layout Save: the provider
  stalled and was collected, leaving SAVE FAILED and the old placement. This is
  not a passing nested regression. The trace includes ATA FLUSH completion work;
  the precise cause was not diagnosed here. The recorded runtime storage/retry
  issues remain open, separately from this pre-QEMU installation fix.


## Storage growth diagnosis and bounded PIO (2026-10-19)

The retained-disk nested Split layout-save stall is reproducible and diagnosed.
Trace `labels-upgrade/20261019T170130555475Z` enters extent growth from
1,503,232 to 3,002,368 bytes at tick 1525 and the root retires its stream at
tick 1962 (600 ticks after the request). Zeroing the new extent through
single-sector PIO commands exceeds the request deadline. This evidence does
not explain the older intermittent ATA busy timeout.

The block-device boundary now admits contiguous sector slices, with checked
fallbacks. BlockRegion batches aligned interiors while preserving partial-sector
read/modify/write and whole-range confinement. ATA uses at most 32 sectors per
command; kernel register validation admits exactly that bounded count, and each
kernel data transfer still checks readiness and transfers only 512 bytes. No DMA,
write retry, enlarged deadline or flush reordering was introduced. Engine growth
zeros through an immutable 16 KiB buffer in read-only image memory. A development
attempt using stack storage killed the provider during commit; moving the buffer
off its 32 KiB stack fixed that regression.

`labels_upgrade_test.py --nested --trace-storage` now passes on the same retained
source disk: `labels-upgrade/20261019T170820481411Z`. Growth takes 111 ticks,
commit 122 ticks, and both resized drafts, three subtree launches/teardowns and
original saved notes pass. Ordinary desktop with tracing also passes cold reboot
and the exact 302-record host oracle: `desktop/20261019T171005498588Z`.
The interactive saved disk was not used or changed. Full default host tests,
engine/volume failure-injection tests, both default target clippy aliases, format
and layout checks pass. New host checks cover contiguous range rejection before
IO, actual aligned batch dispatch, and retiring after an uncertain batch write.

Tracing now records extent begin/end and stream retirement with deadline and
error; timing summaries write retirements separately. ATA read/write counters
count requested sectors; flushes remain command counts.

Still in progress: explicit storage retry after automatic recovery exhaustion;
shared asynchronous scoped forwarding and persistent isolated Split children;
resource/admission diagnostics for that nested composition. Split still ships
with temporary children at this checkpoint.


## Nested persistence and explicit save recovery (2026-10-19)

The ATA growth fix above is pushed as **`581d6175`**. It was rebased over the
independent Omega UEFI baseline update `e298c25a`; those changes are preserved.
This follow-up makes the same F5 desktop exercise persistent nested applications.

### Current behavior and boundaries

- Split's two Notes instances receive separate storage channels rooted at named
  `Left`/`Right` containers beneath Split. Saved text survives subtree teardown
  and cold reboot. Split still has no stock-matrix contract or implementation
  dependency. Its named [storage entrance](source-rs/distribution/applications/split/session/storage/storage.rs)
  sequences refresh, authorization and asynchronous forwarding; realm creation
  lives in its sibling `realms.rs`.
- Platform `Scopes::prepare` now supplies connection/revision validation and
  append-only scope stamping to both implementations. The shared runtime
  `matrix/upstream/exchange.rs` polls one bounded packet/attachment reply without
  blocking interaction. A changed upstream binding invalidates child revisions;
  Split restores only its standing child grants within the same newly authorized
  realm. Transport uncertainty retires the stream rather than reusing late replies.
- Notes uses four surfaces regardless of storage availability; its tab bar is one
  small bitmap. Split retains nine surfaces total, so the surface pool stays 16.
- The stock launcher displays **RETRY SAVES** when the owner reports recovery
  exhaustion/failure. Click it, or F4 with strip focus, for one explicit attempt.
  The action passes through grants bound to live interaction connections (launcher
  to user matrix to root); ordinary Notes is denied both retry and availability.
  Only the desktop owner replaces the provider. Queued attempts are deduplicated,
  revoked on grant changes, and checked again against the live direct child before
  consumption. Success/failure never replenishes automatic restart authority.
  Live apps and drafts remain. The root dispatcher and its small authority model
  are in `distribution/matrix/service/parent/recovery/`; provider ownership stays
  in `distribution/desktop/session/persistence/`.
- The IPC backing pool is 36 channels; an account can hold 32 channel reservations
  and 24 dynamic handles. F5 funds root/user/app channel ceilings at 32/20/10;
  ordinary leaf apps still create none. Two persistent child admissions need ten
  channels and up to 23 handles while their service offers remain pending. These
  are explicit bounds, not global resource discovery. Admission failures now log
  the failed reserve/start stage, requested funding and local limits/charges.
  The initial prototype forgot the desktop's runtime funding override and then
  hit the old handle cap; both are corrected and recorded by the diagnostics.

### Evidence

- Default host tests and 13 harness unit tests pass. Recovery policy tests cover
  consumed restart authority plus stale, revoked, replaced, duplicate and dead
  requesters. Shared scope tests cover stale revision, connection substitution
  and refusing a full stamp chain without dropping an ancestor scope.
- Default userspace/UEFI clippy and the changed shell/live-recovery/trace,
  Notes management, chooser recovery and storage recovery feature set pass.
  Layout still enforces 35 Omega/118 Rust edges; no dependency edges were added.
- `desktop_test.py --nested --trace-storage`:
  `desktop/20261019T172240376279Z` passes separate saves, resize with two drafts,
  dirty close, three complete admissions/teardowns and a cold reboot.
- `provider_live_test.py` on a copied ordinary desktop:
  `provider-live/20261019T172352620786Z` passes precommit and postcommit loss,
  automatic restart exhaustion followed by visible explicit retry and durable
  save of the retained draft, and a failed explicit readiness retry that remains
  bounded while preserving the draft. Every case cold-opens the expected data.
- New `provider_live_test.py --case nested` on a copied Split desktop:
  `provider-live/20261019T172735251575Z` passes failure/replacement/regrant through
  root/user/Split, preserves the live child draft and unchanged sibling bitmap,
  reconciles, saves and cold-opens the result. Both Notes tasks remain alive during
  recovery. The provider probe feature only changes where it observes a dirty
  draft; production supervision and forwarding handle the failure.
- Normal `desktop_test.py --trace-storage`:
  `desktop/20261019T172830077591Z` passes the full F5 UI, cold reboot and exact
  308-record host oracle. `--matrix-storage-test` passes scoped roots, recursive
  delegation, revocation, copies, realm-loaded programs and provider replacement.
- x86-64 smoke, dynamic IPC, admission suites and the ARM64 portability proof
  pass after the changed channel/handle ceilings.
- Retained-disk upgrade plus nested saves passes on the original older fixture:
  `labels-upgrade/20261019T173130306794Z`. Installer idempotence and both existing
  saved notes pass, followed by separate Split saves and complete teardown.

### Remaining limits and sensible follow-up

The old intermittent ATA busy timeout is still not explained by the diagnosed
large extent-growth stall. Keep the traces; do not claim that all stalls are fixed.
The current protocol admits three nonzero scope stamps. Root/user/Split forwarding
uses 600/700/800-tick bounds under the client's 1200-tick ceiling; arbitrary depth
needs a protocol-level treatment instead of repeating the stock one-hop constant.

First-time creation and naming of Left/Right use separate durable transactions.
An interruption between them can leave an unnamed empty container in Split's own
realm. Existing named data is never overwritten; atomic named creation is the
next useful storage operation to design and test. Split has no child restart UI,
dynamic child chooser or presentation batches yet. A permanently failed parent
transport still requires subtree relaunch; ordinary provider replacement through
an intact parent connection is covered above. No new owner semantic decision
was needed for this milestone, and the interactive saved disk was not altered.

## Atomic creation and another real matrix level (2026-10-19)

This section supersedes the previous checkpoint's separate create/name operation,
three-stamp depth cap, per-depth deadlines and resource ceilings.

### Current behavior and reading path

- F5 -> user -> Split -> **DEEPER** now runs the same Split executable beneath
  another saved realm, with independent Notes children. It checks both current
  children's loss reports before replacing them. Reopening Split returns to its
  normal two children; DEEPER revisits the saved nested workspace. A further
  unfunded level says **NO ROOM** without discarding either draft.
- `CREATE_NAMED` commits initial contents, hierarchy metadata, name/type indexes,
  revision/event and quota together. Split uses it for Left/Right. An existing
  name returns a conflict without changing anything; uncertain callers reconcile
  by lookup/head. The sealed request includes 64 metadata bytes within the
  existing 256 KiB payload ceiling. Later publication still permits 256 KiB.
- Named shallow `COPY` shares the source's content and atomically installs its
  new hierarchy name. DEEPER uses it for its executable, then named subtree copies
  for Notes/Left/Right. `COPY_TREE` still refuses copying a subtree into itself.
  Setup is several individually atomic operations, repaired by name on retry;
  the new child is not admitted until preparation finishes. Existing nested
  contents are never overwritten.
- The shared platform scope policy uses two packet scope slots and a read-only
  containment proof when another restriction will not fit. Only successful proof
  plus renewed caller-revision validation permits the folded original request.
  Attachments remain held locally during proof. This works because hierarchy
  parents are immutable and object IDs are never reused; moves or aliases would
  require revisiting it. Tests simulate eight levels and reject escaped scopes.
- Word seven carries one absolute deadline through refresh, containment checks
  and the actual request. Each hop may shorten it, never extend it. Expired
  requests are refused before touching upstream IPC; the next request remains
  usable. The root retains its 600-tick provider bound under the client's
  1200-tick exchange bound. Active durable commits can still have uncertain
  outcomes when a response is lost.
- Start at Split's `session/session.rs`, then `children/children.rs` for admission
  and `children/nesting.rs` for the deeper transition. `children/image.rs` holds
  executable reading/validation. Its `mod.rs` is only Rust plumbing. Platform
  policy remains in `matrix/world/storage/storage.rs`; stock and Split transport
  orchestration remain in their own service/session implementations.

### Funding and kernel correction

The global dynamic channel pool is 40, with up to 36 reservations and the existing
24 dynamic handles per account. F5 root/user/app channel funding is 36/25/15;
private pages are 256/208/128 and reader pages 384/256/128. Root funds seven dynamic
slots, user five and app three. Inner Split receives 80 private pages, 64 reader
pages, ten channels and two descendants. App presentation funding is ten surfaces;
inner Split uses nine and the global surface pool remains sixteen. Picker-to-user
overlap is included. These are explicit admission bounds, not promised unlimited
recursion. The installer upgrades only recognized stock funding configurations.

Reusing a collected task slot under a different parent exposed a kernel control
pair assertion that assumed the old owner would return. `ipc/bootstrap.rs` now
requires the old child's side closed and a fresh epoch, then clears old grants,
queued messages and escrow before installing the new authorized pair. Live service
pair replacement keeps its stricter same-peer checks. A regression proves the old
parent cannot use its handles and the new parent cannot receive an old reply.

### Verification

- Workspace host tests, 14 database-object host tests, 36 storage-lab engine
  tests and 13 harness tests pass. Named creation tests include collision/invalid
  inputs, transaction abort and cold opening a snapshot after every backend write:
  each image exposes either the old state or the complete named object and indexes.
- Default userspace/UEFI and changed deep-recovery/trace clippy checks pass.
  Layout still enforces 35 Omega/118 Rust edges; no implementation dependency
  between Split and the stock matrix was introduced.
- `desktop_test.py --deep --trace-storage` passes repeated admission/collection,
  independent durable edits, dirty close, resource refusal and cold reboot:
  `desktop/20261019T185329329096Z`.
- The complete live provider suite passes precommit/postcommit loss, exhausted
  automatic recovery and explicit retry, readiness-stall refusal, nested and
  deeper replacement: `provider-live/20261019T185352551039Z`. Drafts and the
  unchanged sibling survive; live children remain running, regain storage and
  save durably across cold reboot, including root/user/Split/Split/Notes.
- `--matrix-storage-test --release` passes scoped delegation/revocation, copies,
  realm-loaded programs and replacement, plus full-packet folding, denied foreign
  scopes and an expired request followed by a successful fresh request.
- Normal desktop plus exact 312-record host oracle passes:
  `desktop/20261019T185007998065Z`. Retained-disk upgrade, installer idempotence,
  preservation of existing notes and independent Split saves pass:
  `labels-upgrade/20261019T185217790387Z`.
- x86-64 smoke, dynamic IPC and admission suites and the ARM64 portability proof
  pass after the kernel reparenting correction and funding changes.

### Remaining limits and follow-up

Containment proofs add reads as depth grows; no proof cache or arbitrary-depth
performance claim is made. Nested executables/data are snapshots, not managed
program updates. The experimental wire changed, so arbitrary old compiled nested
snapshots are not promised compatibility. Whole workspace preparation is not one
transaction, though each named operation is atomic and retryable.

Split still lacks a child restart UI, dynamic chooser and presentation batches.
A permanently failed parent transport requires subtree relaunch. Provider
replacement through an intact parent link restores standing grants without
restarting live children. The old intermittent ATA busy timeout remains unexplained.
No new owner decision was needed. Tests used disposable/copied disks; the
interactive saved disk was not altered.

## 2026-10-19 ? Multiple instances and replaceable visual roles

F5's stock matrix now has five local child slots: chrome, three app instances and
background. No kernel or platform code knows these desktop roles. The background
is resolved at `Background`; chrome still comes from the persisted launcher path.
Custom executable bytes remain protected by the existing installer rules. The
background has one bottom surface, no keys/storage/descendants/channel creation.
It sees pointer events only where no higher child is hit. There is no role-selection
UI or automatic background restart yet.

Start at `distribution/matrix/service/children/children.rs` for owner dispatch;
`configuration.rs` establishes roles; `launch.rs` admits scoped code; `workspace.rs`
selects and lists local instances; `lifecycle.rs` handles close/death. Chrome's
`session/tasks.rs` paints running-app rows. Its instance tickets name only this
matrix's live apps. Ordinary Notes is checked to receive DENIED for enumeration
and selection. The platform's new owner-only visibility operation retains geometry
and local scenes, cancels input, and clips hidden publications to zero upstream.
It does not suspend processes or give them information about siblings.

Resource changes: 15 kernel task accounts, 96 dynamic channels (92 reservable),
32-handle waits, 48 reader rights per task, 48 dynamic handles and 512 KiB kernel heaps on both ISAs. Per-task
private/reader budget ceilings are 1024/1536 pages; individual objects remain 128
pages. The stock scene has 40 surface slots. These are bounded lab capacities,
not a scalable production allocator. Runtime wait serialization now sizes its
handle array from the contract; the old 64-byte message buffer only held eight.
`CTHIMP03` adds channel/descendant request ceilings to scoped service declarations;
leaf apps request zero, Split requests its subtree funding. Owner offers still cap
actual grants. Older declarations preserve their former funding behavior.

The compositor now builds immutable image views once per redraw, rather than
once per 256-pixel scanline fragment. The desktop test measures QMP key injection
to screenshot-observed text (including capture/polling overhead); this is an upper
bound, not a renderer CPU benchmark. It does not establish a frame-rate guarantee.

Outstanding: full nested desktop/login, arbitrary dynamic app counts, richer
swappable background grants, background recovery, keyboard task selection and
further measured chrome/compositor work. Selection currently replays surfaces
individually rather than publishing a whole switch atomically. Multiple instances
share their executable's realm; no implicit per-instance copy is made. Split's
subtree can hit funding limits before all three app slots are occupied. Existing
storage revision/conflict semantics remain in force.

Verification passed:

- `cargo test --locked` in `source-rs/`; storage-lab host tests with `--features engines`.
- `cargo check-userspace`, `cargo check-uefi`, source-layout (35 Omega / 121 Rust edges),
  Rust formatting and the 13 discovered harness unit tests.
- Normal `desktop_test.py --trace-storage`: three independent drafts, fourth-launch
  refusal, protected close, slot reuse, Colors over dirty Notes, cold reopen and
  independent whole-store oracle. Artifacts: `desktop/20261019T225656787818Z`.
  QMP-to-visible-text upper-bound samples: 105.5, 113.7, 111.4 ms.
- `desktop_test.py --deep --trace-storage`: Split + Notes + Colors coexist, nested
  drafts survive switching, repeated close/reclamation, live layout, deeper Split,
  funding refusal and cold reopen. Artifacts: `desktop/20261019T230236259836Z`.
- Launcher recovery `--mode recover` on a disposable copy (automatic retry, fallback,
  explicit retries, draft retention and cold save); other recovery modes were not rerun.
  Artifacts: `launcher-recovery/20261019T230133569587Z`.
- x86 smoke, ARM64 portability, `--ipc-test`, `--matrix-storage-test`, `--session-test`.
  Session also checks the full 32-handle wait and sustained drag (120 reports,
  no lost motion; 31 ms to final observed cursor in that run).

The mixed test exposed the old sixteen-reader metadata allowance despite sufficient
page funding. Forty scene surfaces now have 48 local reader slots, covering normal
replacement/ingress headroom; the independent global read pool remains bounded at
128. Exhaustion probes now use contract capacities rather than stale literals.
Automated images are disposable; the interactive `cathedral-storage.raw` was untouched.


## 2026-10-19 ? Nested desktop and atomic app selection

The ordinary Nested tile is the stock matrix compiled with `nested-desktop`,
not a new platform service. Start at `distribution/matrix/main.rs`, then
`service/children/startup/startup.rs`. It copies finite installed seeds inside its
own realm, prepares missing `Users/Template` names, and runs the same chooser in
picker mode. Its selected user is another stock matrix using the usual CONFIGURE
path. The picker is stopped before that user presents. Inner Notes and Colors
keep saved data under their own program subtrees; there is no ancestor storage
binding, physical input or display endpoint supplied to this program.

The template carries another Nested executable with finite seeds. Preparing a
world is lazy, not infinite eager copying. Existing names and data are never
overwritten; each creation/copy is atomic and retryable, while preparation as a
whole is not one transaction. This is selection of a realm, not Warden auth,
encryption, guest-kernel execution or hardware virtualization.

Bounded funding: Nested requests 320 private pages, 512 reader pages, 40 channels,
five descendants and 24 surfaces. It gives its selected user 256/384 pages,
24 channels and three descendants; that supports chrome, background and one leaf.
Close other outer apps before launching Nested. Another inner Nested level is
present in the chooser but refused with NO ROOM by the current resource ceiling.
No task-count or kernel memory-cap increase was made for this milestone.
Atomic switching exposed the framebuffer aperture overlapping shared mapping
slots above the old sixteen-reader range. Its virtual address is now
`0x0000_0080_0400_0000`; a compile-time assertion keeps the entire guarded object
mapping arena below it. Display clients use DISPLAY_INFO, not a baked-in address.
The normal outer desktop still supports three ordinary leaves. Fixed surface
ranges may refuse fragmented capacity rather than moving live siblings.

CTHIMP04 extends the sealed declaration with page/surface demand and requires all
five requested resource amounts to fit the owner offer before spawn. It uses the
stock matrix's ordered environment entry. Legacy declarations retain their prior
rules. Stock config ceilings now admit larger matrix programs while ordinary
leaves retain 128/128 pages and ten surfaces; old recognized stock configurations
are upgraded without replacing custom layout. The fourth tile puts task rows at
local y=504, and chrome has seven surfaces. The storage deadline is 1200 ticks
(12 seconds at 100 Hz), with client and watchdog waits derived from it. The larger
template copy exceeded the previous 600-tick bound during extent growth in
disposable runs. The boot owner's health wait now covers the matrix storage
deadline, avoiding a root-connection panic during a larger template copy.

App selection stages old/new app visibility in the existing presentation protocol;
its batch capacity now covers the 40-surface scene. Commit publishes the entire
switch; refusal aborts and restores local selection. Startup, teardown, settings
and individual app repaint are still separate operations; this is not vblank sync.

Remaining follow-ups: scalable resource negotiation (including multiple inner
apps/deeper desktop nesting), responsive narrow-screen chrome, nested background
and picker recovery, explicit handling of user-matrix startup failure, automatic
nested layout reflow, and stock-code upgrades inside already-created inner users.
Current installation upgrades outer stock realms and Nested's installed seeds;
existing inner snapshots deliberately retain their code. Reopen after a failed
inner startup by closing Nested from its outer task row. Multiple instances of
Nested share its saved Users subtree, just as multiple Notes instances share their
program data. No implicit per-instance realm clone is made.

Verification:

- Rust workspace host tests and 36 storage-lab engine tests pass; userspace/UEFI
  checks, Nested + trace clippy, formatting, layout (35 Omega / 121 Rust edges)
  and 13 harness unit tests pass.
- `desktop_test.py --nested-desktop --trace-storage` passes inner user creation,
  Notes/Colors persistence across cold reboot, explicit UNSAVED WORK and NO ROOM
  refusals, subtree teardown and outer/inner Notes isolation. Artifacts:
  `desktop/20261019T234725230182Z`.
- Normal desktop passes three Notes instances, switching, refusal, protected close,
  slot reuse, cold reopen and the independent whole-store oracle (532 records):
  `desktop/20261019T234900506491Z`.
- Deep Split passes coexistence, resizing, recursive close, funding refusal and
  cold reopen: `desktop/20261019T235047434065Z`.
- ARM64 portability proof and x86-64 smoke pass after the framebuffer relocation,
  including display restart, aperture guards and mapping rollback/reclamation.
- Matrix desktop passes recursive atomic publication, admission rollback,
  stalled-provider isolation, expired-request refusal, lost-ack reconciliation,
  provider replacement, physical input and cold reopen. Matrix storage passes
  scoped delegation/revocation, copies, realm-loaded programs and replacement.

The fault-injection regression caught a client deadline still fixed at 1200 ticks
after the provider bound was lengthened. `storage::EXCHANGE_TICKS` now derives
from twice that bound, leaving time for the provider verdict to return without
retiring the client connection. The stall fixture now distinguishes an expired
pre-execution request (no write) from a committed request whose reply was lost;
its former expectation that both committed contradicted provider deadline checks.

One earlier disposable normal-desktop run (`desktop/20261019T234336003280Z`)
reported NO_MEMORY during login and recovery-screen publication. It did not
reproduce in subsequent normal, nested or deep runs; cause remains unresolved.
The previous intermittent ATA busy timeout remains open too. Tests use disposable
images; the interactive saved disk was not altered.

## 2026-10-19 - Desktop measurement and collection I/O

The next performance step uses the ordinary F5 composition. Start at
`tools/boot-harness-rs/desktop_test.py`, whose steps feed `desktop_timing.py`;
`storage_timing.py` parses the serialized provider's phases. Each boot records
host boot/login/launch/typing/switch/save observations. Optional `--trace-storage
--trace-input` adds provider CPU, sector counts, collection/commit/extent spans
and compositor redraw CPU. Reports and interpretation are in the harness README.
Host observations include QMP, deliberate input spacing and screenshot polling;
guest ticks, per-process CPU and host wall time are distinct clocks. Tracing
perturbs execution. These are diagnostic observations, not performance promises.

The main measured save cost was reachability collection: repeated per-chunk
index lookups across shared program manifests thrashed the small engine cache.
`platform/libraries/database-objects/collection.rs` now marks distinct referenced
index keys, validates them in one ordered metadata scan, then performs the same
bounded removals. Missing indexes still refuse collection or pinning before any
mutation. Pin validation also uses this scan, trading individual lookups for a
root-wide ordered index walk when the pinned object has chunks. Metadata work
still scales with the live set; at most eight removals does not bound total
collection CPU to eight records. No storage format, durability, retention or
kernel authority changes were made.

The provider cache is 64 KiB instead of 32 KiB, inside its existing 56-page heap.
A 128 KiB trial exhausted that heap while copying a user and was discarded.
Ordered validation with 32 KiB also reduced reads, but 64 KiB further improved
the measured working set. This is a bounded tuning choice, not general memory
negotiation or proof that every future workload fits.

Comparable `--trace-storage` runs (artifacts under `build/boot-harness-rs/desktop`):

| Observation | Baseline `20261020T002203005756Z` | Optimized `20261020T002630278955Z` |
| --- | --- | --- |
| Notes save host upper bound | 1506.6 ms | 358.4 ms |
| Notes provider sectors read | 2785 | 201 |
| Colors save host upper bound | 3053.4 ms | 525.9 ms |
| Colors provider sectors read | 5410 | 194 |
| Create user host upper bound | 3117.6 ms | 2045.1 ms |

Host load varied. Login and launch did not show a corresponding improvement;
do not generalize these save results to all desktop latency. The final run with
both trace features (`20261020T002959149665Z`) measured about 2-3 ms compositor
CPU per typed character, within 132-136 ms host observations. This does not
attribute all remaining latency to scheduling: app rendering, IPC and fixture
overhead are not separately resolved by these spans.

Verification:

- Rust workspace host tests; 16 database-object tests with `--features host`,
  including shared chunks/pins, corrupt-index refusal without mutation, existing
  cold-write recovery tests and a cold 64 KiB-cache collection I/O regression;
  all 36 storage-lab engine tests pass.
- Userspace clippy, display input tracing and storage trace/recovery feature
  clippy, formatting, layout (35 Omega / 121 Rust edges), and 17 harness unit
  tests pass. No kernel/ISA code changed; ARM64 was not rerun for this milestone.
- Ordinary desktop with both traces passes three app instances, input, switching,
  saves, protected close, cold reopen and the independent whole-store oracle
  (536 records): `desktop/20261020T002959149665Z`.
- Nested desktop passes both boots, inner Notes/Colors persistence, recursive
  dirty-close refusal, funding refusal and outer isolation:
  `desktop/20261020T003211305619Z`.
- `run.py --matrix-desktop-test --release --timeout 120` passes stalled-provider
  isolation, expired request refusal, lost acknowledgement reconciliation,
  replacement, focus/clipping and cold reopen; artifacts in `matrix-desktop-test`.

Next: resource accounting and funding negotiation, then nested background/picker
recovery and layout reflow. Keep timing evidence alongside those changes rather
than raising capacities blindly. Initial-user creation and login remain visible
costs worth separating further. Previous unresolved NO_MEMORY and intermittent
ATA reports remain open; the deliberately oversized cache trial above is a
separate, reproducible and reverted experiment. All guest runs used disposable
images; the interactive `cathedral-storage.raw` was untouched.

## 2026-10-19 - Live resource growth through matrix owners

Start at `distribution/matrix/service/children/admission.rs`: resolve imports,
check surface placement, ask for a local funding shortfall, then reserve/spawn.
`children/funding.rs` authenticates a requesting child's endpoint, applies stock
ceilings and obtains its own shortfall upstream before transferring funds. Shared
amount/ceiling checks live in `platform/libraries/matrix/funding.rs`; the platform
`FUND` packet is on the supplied interaction connection. The kernel knows neither
matrix roles nor this protocol.

`RESOURCE_GROW` (71) atomically transfers increments of private pages, reader
pages, channels, free descendant slots and physical frames along an owned launch
edge, including a live child. The syscall validates the complete input before
touching ledgers; the resource entrance validates all five families before any
commit. Runtime mapping tables already cover the fixed mapping arena. This does
not enlarge global pools, mapping/handle limits or CPU allowances. A separate
`NO_PROCESS_CAPACITY` refusal identifies unavailable transferable slots, including
the existing ordering constraint, without revealing account identities.

Current stock growth policy is explicitly bounded: programs admitted with
descendants can grow to the larger of their initial grant or
384/768 pages, 40 channels, six descendants and 1792 physical frames. Ordinary
leaves have no growth policy. Each transfer is atomic; the complete ancestor chain
is not one transaction. Capacity granted to an intermediary may remain there
after a downstream refusal and is reused or returned when that subtree closes.
There is no live shrink/recall protocol or automatic retransmission after a lost
reply; a failed transport retires the upstream connection.

The ordinary Nested program now requests 32 surfaces, still from the existing
40-surface scene. Its fresh user runs Notes and Colors concurrently. A third leaf
or another Nested still refuses. Surface ranges remain fixed, and the global
15-task ceiling remains unchanged. The initial simultaneous-app test exposed a
stranded picker account below the selected user's process-order boundary. Nested
now loads user code, retires the picker, reserves the user (reusing that account),
then publishes its full view. Admission refusal restarts the picker. A later
user-startup fault still needs the broader recovery work listed below.

Refusal cleanup also exposed an unused object-executable reference: the old path
minted it before policy checks and could retain it on reservation refusal.
`TASK_IMAGE_RELEASE` (72) now drops an unused caller-local object grant; templates
are refused and consumed grants are stale. Stock admission creates the grant only
after preflight and releases it on reservation failure. This preserves the
existing reservation rollback after a failed spawn.

Trace diagnostics distinguish page/reader/channel/process/frame shortfalls,
policy ceilings, transfer failures, surface range exhaustion and local app slots.
The desktop keeps NO ROOM. In the nested run, the inner matrix requested
`[100,0,0,1,97]`; its owner obtained `[36,0,0,0,0]` from the outer user, then granted
the full inner request. No resource inventory crossed a child-facing boundary.

Verification:

- Workspace host tests pass, including five-family atomic refusal with live
  charges and platform ceiling/overflow tests. Userspace/UEFI clippy, layout
  (35 Omega / 121 Rust edges) and 17 harness unit tests pass.
- IPC QEMU passes live page growth/use, invalid-address/handle refusal, subtree
  cancellation and budget restoration. Its executable probes drop more than one
  grant-table's worth of abandoned grants and verify stale names plus exact quota,
  process and physical baselines.
- Concurrent Nested Notes/Colors, repeated refusal, switching, reuse after close,
  isolation and both cold boots pass: `desktop/20261020T030752847109Z`.
- Normal desktop, three independent Notes drafts and whole-store oracle pass:
  `desktop/20261020T030423386464Z` (536 records). Deep Split passes coexistence,
  nested close/resize/funding refusal and cold persistence:
  `desktop/20261020T030320712540Z`.
- Matrix desktop recovery and cold reopen pass. ARM64 portability proof passes;
  the new live-growth syscall's guest exercise is currently x86-64.

Next: nested background/picker and post-admission user-startup recovery, live
layout reflow, and a deliberate upgrade route for saved inner code snapshots.
Existing inner users retain their code; create a new inner user to try live
growth. Flexible surface funding and live quota return are still separate work.
Previous intermittent ATA and unreproduced NO_MEMORY reports remain unresolved.
All tests used disposable images; the interactive saved disk was not touched.

## F5 live quota and transaction headroom repair (2026-10-19)

The owner's saved image reproduced `live/pinned content exceeds root quota`
before QEMU launch. There were two separate problems: labels checked admission
before named replacements, and the complete upgraded live/pinned set genuinely
needed 1,287,797 bytes, beyond the old 1 MiB allowance. Existing nested users keep
their older code snapshots; they cannot be silently discarded as upgrade garbage.

The coordinated installer now stages labels without collection/admission, then
finishes hierarchy and named programs before collecting unreachable history and
checking final quota. Standalone label/hierarchy migrations still enforce quota.
Hierarchy checks even when its own plan has no changes, and no longer rejects the
intermediate plan before named replacements. Final excess reports required bytes
and leaves program publication untouched.

The logical root allowance is 2 MiB. Merely raising it exposed a second failure
on the owner's fuller image: it booted, but the next Notes save ended the provider
with `Engine`. The compacted database occupied 2,609,152 bytes and the old 4 MiB
extent did not leave enough transaction allocation space. Database lab media and
the provider now use an 8 MiB assigned extent. F5 recognizes legacy guarded 4 MiB
images and invokes explicit offline `grow` before installation, even if the
program receipt already matches. Growth holds the image lock, syncs a complete
content-named backup, prepares and cold-compares every record off-image, then
grows the file and moves its trailing guard. Identity and contents survive. It
can restore the original length/bytes on ordinary publication errors. Like the
existing offline compactor, interruption during publication requires its backup;
this is not a power-loss-atomic resize. A successful growth may remain if a later
program upgrade refuses. No existing media is reformatted.

Verification:

- Workspace host tests, all 16 database-object host tests and all 38 storage-lab
  engine tests pass. Regression coverage crosses quota with mixed generations,
  admits the final replacement, preserves user/custom heads, and refuses pinned
  final excess with byte-identical input. Offline growth checks records, identity,
  guards, backup, shrink refusal and refusal before publication.
- Quota-pressure fixtures scale with the allowance. The object quota fixture now
  uses unique chunk identities rather than wrapping byte values after 251 chunks.
- All 19 harness unit tests pass, including growth with an unchanged receipt and
  refusal without replacing media/receipt. Userspace, storage-lab and object-host
  clippy, workspace formatting and layout checks pass.
- Default desktop creation, concurrent Notes drafts, close, saves, cold reopen
  and the 536-record whole-store oracle pass on 8 MiB media:
  `desktop/20261020T034543188251Z`.
- The exact saved disk was tested in `quota-upgrade-repro`: 77 heads survive;
  only nine recognized stock program heads change. It boots to the existing
  users, enters User 1, launches Notes, saves `quotacheck`, and cold-opens that
  text with healthy storage. Test text was written only to a disposable copy.
- The interactive `cathedral-storage.raw` and receipt have now been upgraded.
  Its complete tables/records, identity, guards and even physical bytes match the
  verified upgraded copy. The original content-named backup ends in
  `be036e6641b95064e652fb60da2483aaab47ec14f01cf8ceae620eee9eea8387.raw`;
  the old receipt is also kept in `quota-upgrade-repro/interactive-before.programs`.

These remain fixed lab capacities, not a general quota/volume expansion policy.
Saved inner-user code snapshots are still deliberately retained. The previous
live-growth milestone's recovery/layout follow-ups remain next; no new owner
semantic decision was needed for this launch repair.


## Compact taskbar and bounded recursive desktop (2026-10-19)

The stock launcher occupies an 80-pixel bottom bar with 48-pixel app icons,
horizontal running-instance buttons and right-side controls. Defaults use eight
pixels of margin/gap. Login remains full-screen. Matrix configuration writes
CTHMAT02; CTHMAT01 decodes to compact geometry while preserving launcher selection,
key authority and resource fields. Saved version-2 arrangements remain unchanged.

The bounded kernel arena has 31 accounts, 64 boot endpoints, 192 dynamic channels,
512 sealed reader handles and a maximum 4096 reader-page grant. Both architecture
heaps are 2 MiB. Memory backing vectors initialize directly in their heap arena;
the kernel exhaustion probe keeps bookkeeping in its allocations. The enlarged
spawn record is aligned to 512 bytes so checked single-page copy-in remains valid.
IPC escrow retirement uses a word array, tested across every 64-bit boundary.
Abandoned executable references force a return to the boot loop for reclamation
before the next caller budget query. Their pending counters cover a whole subtree;
the admission guest checks that physical and grant budgets return to baseline.

The display/matrix surface ceiling is 64. Admission may lend descendant hosts
up to 24 spare ranks above their declared minimum, within existing disjoint ranges
and standing ceilings. Funding keeps 256 reader pages at forwarding owners for
code and image replacement. These changes conserve authority; manifests do not
mint budgets. Presentation batches reset in place to avoid stack-size-dependent
teardown faults while dropping all retained readers.

Nested startup recovery is local: one automatic picker retry, then exit to the
outer taskbar; failed selected-user startup restores its picker; one background
retry, then a usable desktop without a background process. Outer drafts/siblings
survive repeated attempts. Saved inner executable/template snapshots are still
intentionally retained. Use a fresh outer user for new code at every depth; a
managed recursive upgrade policy is separate work.

Storage distinguishes retained quota refusal from known pre-commit extent refusal.
For the latter it drops/reopens the poisoned engine before returning capacity,
keeping authenticated connections. Commit failures remain uncertain and other
I/O/corruption failures still retire the provider. Opening an existing object table
uses no writer. The ATA flush deadline is 250 ticks (other phases remain 50), inside
the existing request deadline; there is no automatic retry of a failed write.

Verification completed during this milestone:

- Workspace host tests, 17 database-object host tests, 38 storage-lab engine tests,
  19 harness unit tests, userspace/UEFI clippy and layout checks pass.
- Ordinary desktop, three simultaneous Notes drafts, fourth-launch refusal,
  close protection, reuse and cold reopen: `desktop/20261020T043449182055Z`.
- Settings cancel/live geometry/cold persistence: `settings/20261020T043606799710Z`.
- Launcher crash recovery and startup fallback: `launcher-recovery/20261020T043733935425Z`.
- Full retained quota, three save refusals, failed close-save, surviving draft,
  sibling reads and two boots; independent oracle confirms every record unchanged:
  `storage-capacity/20261020T043905119323Z`. Physical extent poison/reopen and cleanup
  retry are separately tested against the real redb engine on the host.
- Nested picker, selected user and background fault injection, repeated cleanup
  and surviving outer Notes draft: `nested-failures/20261020T044335198301Z`.
- Three Nested desktops beneath the outer desktop, deepest Notes save/read,
  recursive dirty-close protection, two subtree close/re-enter cycles, isolation
  and all checks repeated after cold reboot: `desktop/20261020T044754085311Z`.
- Deep Split coexistence, drafts, resize, nested close, funding refusal and cold
  persistence: `desktop/20261020T044435628405Z`. Management original/recovered/revoked
  authority cases pass: `management/20261020T044828618328Z`.
- Provider live pre/post-commit recovery: `provider-live/20261020T044643905697Z`;
  startup crash-once/bad-volume: `provider-startup/20261020T045010333443Z`;
  all four lifecycle cases: `lifecycle/20261020T045034245325Z`. Other provider
  fixture variants were not rerun in this milestone.
- Matrix desktop, admission and IPC guest suites pass, including executable
  release and reclaimed resource baselines.
- Full current F5 installation on a copy of the owner's two-user saved disk,
  Colors launch/save/close/reopen and cold boot: `colors/20261020T045258562388Z`.
  The upgraded store contains 1188 records and fits the existing assigned extent.
- ARM64 portability proof and x86-64 smoke pass; smoke used a 90-second host timeout
  under concurrent load after the default 30-second run timed out late in the suite.

Deep startup remains a performance follow-up: the successful run observed up to
55 seconds at the third level. An earlier run exceeded the old 45-second scene
wait; nested checks now allow 120 seconds and report time at each depth. This is
an observation bound, not a guest deadline change or a latency fix. Concurrent
desktop tests also exposed shared profile artifacts overwriting each other's
installed programs before cold verification; each run now has its own profile
and program paths. The isolated three-level rerun passed both boots.

Next: profile recursive startup and steady-state rendering/IPC costs, deliberate
upgrades for saved inner code snapshots, and flexible surface funding/live quota
return. No interactive saved disk has been modified during this milestone; F5
will install the new stock code normally. A fresh outer user is needed to get
the current implementation throughout the Nested template chain.

## Saved Nested user launch ABI repair (2026-10-19)

The owner's saved Nested user returned CANNOT OPEN after the task-capacity
increase. Reproduced on an unchanged copy of the interactive disk:
`nested-saved-repro`. The old user matrix reached startup but its launcher spawn
returned INVALID_ARGUMENT. Raising MAX_LAUNCHES from 14 to 30 had also enlarged
TASK_SPAWN_TREE's implicit request from 176 to 304 bytes. The kernel read unrelated
bytes beyond the old request and rejected them as nonzero unused grant entries.
Fresh-image tests rebuilt every executable together and missed this compatibility
failure; preserved code must also be exercised against a new kernel.

MAX_SPAWN_LOANS now fixes that syscall's wire capacity at 14 independently of the
31-account kernel arena. The kernel and runtime enforce the same limit. Dynamic
descendant reservations and the three-level desktop capacity remain unchanged.
No executable snapshots, configuration or user data need replacement for this fix.
Saved desktops continue to use their saved UI; fresh users get the new taskbar.

Verification: host tests, userspace/UEFI clippy, layout and format checks; IPC guest
suite including a real recursive spawn with nonzero adjacent bytes after the old
176-byte record; admission guest suite. An unchanged saved-disk copy with the new
kernel reaches the inner desktop (`nested-saved-fixed`), where the original
vertical launcher renders. A second boot from another unchanged copy also launches
inner Notes (`nested-saved-fixed-app`). The interactive saved disk was not modified.

## Fresh lab state instead of recursive legacy upgrades (2026-10-19)

Owner direction: legacy lab users do not need compatibility or migration; making
new users is cheap. This supersedes the earlier proposed recursive snapshot
upgrade milestone. Current stock seeds already produce the bottom taskbar at
every depth. Cached inner templates on the retained disk explain the old UI.

The default launch still keeps session data. The new **Cathedral: Fresh Lab in
QEMU** launch entry adds `--fresh`. After building, it sets the disk and receipt
aside together, then uses normal first-install seeding. It refuses a busy disk;
a failed receipt move restores the disk. No kernel/platform changes or recursive
program migrations were added. Use this route when discarding obsolete lab state.

Fresh creation verified all three taskbars. Cold reopen then exposed the chooser's
hard-coded 600-tick LAUNCH receive timeout, shorter than the matrix's bounded code
load/funding work. It panicked while selecting the third nested user. LAUNCH now
uses a named stock-contract budget of eight storage intervals plus reply margin,
like multi-stage CONFIGURE. Other UI requests retain their short deadline; no
operation is automatically replayed after an uncertain timeout.

The interactive disk was reset through this command. Its prior disk and receipt
are in `build/boot-harness-rs/before-fresh-rc75aito/`; the fresh disk contains only
current stock seeds, ready for NEW USER. Validation: 21 harness tests, Rust host
tests, userspace clippy, formatting and layout checks pass. A copy of this newly
seeded interactive disk passes fresh user creation and bottom-taskbar rendering
at all three Nested levels, then repeats those checks after cold reboot:
`build/boot-harness-rs/fresh-taskbars-fixed/`. The test's users are only on that
copy. This focused check does not repeat the previous milestone's full subtree
close/exhaustion suite. Deep startup remains slow. Focus subsequent work on actual
OS behavior, not legacy lab upgrades.

## Nested storage request amplification (2026-10-19)

The latency workload now lives in `tools/boot-harness-rs/desktop_latency.py`,
entered through `desktop_test.py --latency --trace-storage --trace-input`.
It builds the ordinary F5 composition on a disposable disk, measures the outer
desktop and three Nested levels, saves at each depth, then cold-opens the same
disk. Picker launch, user creation/entry, Notes launch, pointer, typing, save
and close have separate observations in `boot-*-latency.json`. The existing
`desktop_timing.py` reports provider operations, sectors and compositor CPU.

The two-slot scope packet was causing repeated ancestry HEADs at each ancestor.
At eight forwarding hops, the host model expanded one write into 21 provider
operations. `platform/libraries/matrix/world/storage` now retains four successful
containment proofs per binding, reducing a repeated write to one provider call.
Stock and Split confirm proofs only after the matching upstream check succeeds.
Requests still authenticate the live connection/revision and reach the provider
with their innermost scope. Proofs never authorize an operation or cache its data.
Immutable parents and never-reused object numbers are essential to this rule.
Revocation, regrant and upstream changes invalidate proofs; retiring/replacing a
root provider revokes **all** its bindings, leaving other sources independent.
Eviction requires another proof. No kernel mechanism or IPC wire format changed.

The stock nested seed copier also checks whether a destination exists before
looking up its source. This removes ten unnecessary source lookups on reopening
an existing template, without replacing its programs or data.

Measured third-level observations (seconds, traced QEMU/TCG):

| Phase | Before | After | Provider calls before ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ after |
| --- | ---: | ---: | ---: |
| Fresh picker launch | 41.62 | 21.41 | 449 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ 60 |
| Fresh user entry | 27.40 | 7.91 | 312 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ 27 |
| Cold picker launch | 26.23 | 6.34 | 283 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ 28 |
| Cold user entry | 40.22 | 7.83 | 312 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ 27 |
| Cold Notes launch | 13.43 | 3.36 | 89 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ 9 |
| Cold Notes save | 5.20 | 3.00 | 21 ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ 1 |

Baseline artifacts: `desktop/20261020T054908071290Z`; revised artifacts:
`desktop/20261020T055706461711Z`, both beneath `build/boot-harness-rs/`.
The baseline cold workload was rerun against its original ESP/disk after fixing
a fixture mistake: retyping identical saved text makes Notes clean, so the cold
workload now types different text and checks each deletion. Fresh baseline data
is from the original completed first boot. Both corrected cold workloads passed.
Host observations include QMP/screenshot overhead, host load varies and tracing
adds work. These are observations, not timing promises; operation counts show
the eliminated work independently of host scheduling.

This does **not** establish an input latency improvement. Revised third-level
typing observations span 187ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Å“335 ms across both boots; root pointer observations
span 40ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Å“136 ms there. The compositor CPU for the three cold typed characters was
about 2ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Å“6 ms each, measured on its own clock. Do not subtract it from host time
and label the remainder scheduler or IPC cost.

Next measured targets: first-time seed creation still takes 21 seconds at depth
three; a single cold save still reads 938 sectors and writes 146, with about two
seconds of provider-account CPU in this run. Investigate collection/commit/read
work and serialized seed creation before blaming rasterization or adding SIMD.
Separately measure chrome polling and interaction round trips; their costs have
not yet been isolated. No new owner decision is needed for this work. The
interactive disk was not reset or modified during these measurements; Fresh Lab
remains the way to discard obsolete executable snapshots.

The subtree-reopen fixture also needed a stronger close observation. A blank
canvas appears before child collection, and all nested backgrounds use the same
color. Its old one-pixel assertion could pass with the subtree still alive;
the next launch then exhausted surface funding. Both nested/latency workloads
now wait for the owner's running-task cell to clear too. The first full-tree
run (`desktop/20261020T060137243348Z`) hit that sequence and ultimately ended in
a host Python `MemoryError` during screenshot capture, not a guest panic.

Verification completed: Rust workspace host tests (including real recursive
proof forwarding, stale bindings, forged scopes, source invalidation and bounded
eviction), 21 Python harness tests, default userspace clippy and the stock matrix's
nested/trace/fault/probe feature set, formatting and layout checks. Formatting
also normalized a few pre-existing named Rust bodies; kernel edits are cosmetic.
The latency workload passed both boots. Provider replacement, uncertain commits,
independent roots and cold verification passed in
`matrix-database/20261020T060012433920Z`. Two-level Split persistence, protected
close, funded refusal and repeated reclamation passed both boots in
`desktop/20261020T060035553703Z`. ARM64 and kernel smoke were not rerun for this
userspace change.
The corrected three-level desktop fixture passed both boots, deepest saved
Notes, protected close, two complete subtree teardown/reopen cycles per boot,
and an isolated outer note in `desktop/20261020T060627976276Z`. No guest code
change was needed for that fixture race; waiting for actual task removal fixed
the observation. These later untraced runs experienced different host load and
are functional verification, not additional before/after performance samples.

## Storage maintenance and nested interaction costs (2026-10-20)

The previous traced cold third-level save spent 1,601,597 us of provider CPU in
collection and 262,889 us in its publication commit. The provider was scanning
every live head/manifest before every small mutation, including each template
copy. `Store::collect_under_pressure` now reads the durable quota ledger first:
below 1.5 MiB it does no writer/collection scan; at or above that watermark it
runs the same eight-record bounded pass. The 512 KiB reserve is maintenance
headroom, not a new admission rule. Every mutation still checks exact quota,
and explicit COLLECT remains unconditional. There is no automatic retry of an
uncertain write. Pins, immutable content, immediate durability and the on-disk
format are unchanged. Unpinned old revisions can remain longer; they have no
retention guarantee. The independent small-workload host oracles now expect
that retained history rather than assuming every request collected it.

Nested realm preparation also resolves each named source at most once in that
preparation. Workspace and next-level template copies share those local
source/head results. Destination checks still precede lookup; nothing survives
the preparation, and each COPY still validates the expected source revision.
This removes ten provider requests from fresh third-level picker setup, from
60 to 50, without a new batch protocol, authority shortcut or partial-tree
publication rule.

Notes now retains its unchanged background, close button and tab bitmap.
Subsequent ordinary text edits publish only a replacement text bitmap. Successful
decoration updates are remembered within one scene, and invalidation discards
the scene so every surface is recreated. The `trace-input` route now includes
Notes key receipt and paint begin/end. Per-step JSON records the ordered
source/matrix/Notes observations; source and matrix releases are excluded from
the key-delivery series. Different matrix CPU counters must never be joined.
An incomplete paint span can cross a screenshot observation boundary, so do not
pair an old end with the next key's begin.

Traced QEMU/TCG observations at the third nested level:

| Measurement | Previous milestone | Revised |
| --- | ---: | ---: |
| Cold save observed time | 2998 ms | 739 ms |
| Cold save provider CPU | 1986 ms | 391 ms |
| Cold save collection/ledger CPU | 1602 ms | 18 ms |
| Cold save sectors read / written | 938 / 146 | 217 / 97 |
| Cold save durable flushes | 2 | 1 |
| Fresh picker observed time | 21410 ms | 16323 ms |
| Fresh picker provider requests | 60 | 50 |

Baseline: `desktop/20261020T055706461711Z`. A pressure-only intermediate run
with input instrumentation is `desktop/20261020T155022438947Z`; it completed both
boots before the Notes/seed changes. The completed revised workload is
`desktop/20261020T155343691135Z`. All paths are under `build/boot-harness-rs/`.
Host load and trace overhead vary; counts and eliminated work are stronger
evidence than a timing ratio. Cold user entry still took 8.3 seconds and fresh
entry 11.5 seconds in the revised run; these phases were not fixed.

Deep typing observed 209ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Å“240 ms fresh and 114ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Å“118 ms cold, so responsiveness is
still uneven. Source-to-Notes delivery crossed seven matrix processes and took
5ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Å“8 nominal 100-Hz guest ticks in the revised samples. For one cold character,
delivery took six ticks, Notes paint took five ticks including IPC, Notes charged
about 1.8 ms CPU during that paint, and compositor redraw about 0.9 ms. These
separate clocks are not a decomposition of host elapsed time. The useful next
measurement is wake/dispatch and interaction round trips across the chain,
including chrome status polling; do not assume rasterization or assign all
remaining time to kernel scheduling.

Remaining storage limits: a pressured root still pays for a whole live-metadata
scan; physical engine capacity is independent of logical quota; creation still
does multiple durable transactions. The revised deep save still reads 217
sectors for a tiny payload. Investigate scope walks, engine reads/cache and
commit allocation separately before increasing cache/heap funding or adding a
general transaction API. Kernel scheduling and ABI are unchanged in this work.
No new owner question blocks these follow-ups. The interactive disk was not
modified or reset; use **Fresh Lab** for updated programs at every nested level.

Verification:

- Workspace host tests, 18 database-object host tests (including a cheap
  cold-cache ledger check, bounded pressure collection, pin survival, exact
  accounting and unchanged independent roots), 38 storage-engine host tests and
  22 Python harness tests pass. Default userspace and changed trace/fault/probe
  feature clippy checks pass, as do layout and Rust format checks.
- The default F5 desktop passes fresh/cold operation, three Notes instances,
  draft switching, close/save/discard, tabs and the independent 541-record oracle:
  `desktop/20261020T155644324318Z`.
- The standalone object fixtures had two stale assumptions: their 4 MiB image
  no longer matched the 8 MiB provider, and reader-page exhaustion expected BUSY
  instead of the kernel's NO_MEMORY. The volume seeder now accepts a size while
  retaining its original default for the separate volume-IO lab; the object
  fixture uses the database extent and still verifies outside guards. The
  pressure assertion still requires reads to resume after readers are dropped.
  Two-boot publication/refusal/cold recovery passes at
  `objects/20261020T155909026335Z`; three-boot pins, bounded collection, bulk
  pressure, exact cold records and final heap/reader release pass at
  `objects/20261020T160023934011Z`.
- Provider replacement, ambiguous commits, independent roots and exact cold
  oracle pass at `matrix-database/20261020T160057846155Z`.
- Two forced Notes rendering invalidations during creation/publication retain
  the draft and restore input: `settings/20261020T160125588429Z`.
- Full retained quota repeatedly refuses save and close-save while preserving
  drafts, sibling reads and all persisted records across two boots:
  `storage-capacity/20261020T160241919815Z`.
- Three nested desktops, deepest saved Notes, recursive dirty-close protection,
  two subtree close/reopen cycles per boot, confinement and cold recovery pass:
  `desktop/20261020T160310046094Z`.
- No kernel behavior changed; ARM64 and kernel smoke were not rerun.

## Continuous pointer movement and nested matrix exits

The owner reported APP STOPPED followed by losing inner desktops during normal
F5 use. A fresh disposable desktop reproduced a matrix exit while creating a
nested user with pointer motion during every scene wait. The first reproduction
is `desktop/20261020T162724840470Z`; the instrumented reproduction is
`desktop/20261020T163058173648Z`. The latter identifies `POLL` (operation 5)
timing out with `-110`, followed by `matrix-service-exit`. This is a userspace
matrix failure, not an observed kernel fault or an old-disk-only failure.

The stock matrix used to drain up to eight upstream input events at the start
of a turn, then refresh again before dispatching the selected child request.
Each ancestor could do the same before answering that child's POLL. Under
continuous motion this delayed nested replies past the unchanged interaction
deadline. The parent now consumes one snapshot per service turn. MORE records
remaining work and makes the next idle wait immediately runnable, while an
already-selected child request gets its turn before another refresh. Quiet
notification-driven matrices still park; legacy two-endpoint fixtures still
refresh on demand. No scheduling grants or IPC deadline were increased.

The first fairness-only stress run completed its fresh boot but lost a deepest
Notes close click during a cold reopen (`desktop/20261020T163320913383Z`);
there was no matrix exit. Bounded child queues now coalesce trailing pointer
motion within a button state, retaining the first report's coordinates and the
latest motion. Button transitions, keys and cancellations remain ordered.
Non-coalescible overflow still cancels only that child's queue. Host regressions
cover a motion flood around press/release, key/cancel boundaries, and sibling
isolation on genuine overflow.

`desktop_test.py --nested-desktop --pointer-stress` moves the pointer throughout
startup/scene waits and for three seconds after each desktop becomes visible.
It fails immediately on any matrix service exit, and retains the three-level
Notes, dirty-close, recursive reclamation and cold-persistence checks.
Synchronous interaction receive failures now log the operation and status.
Normal F5 also saves serial output to `build/boot-harness-rs/interactive/serial.log`
while still printing it to the terminal. The next boot replaces that log.

The owner's saved interactive disk has not been reset. Use **Cathedral: Fresh
Lab in QEMU**, then create users, to get the corrected executable throughout the
tree; existing user realms intentionally retain their copied programs.

Verification:

- The complete motion stress workload passes on fresh and cold boots, with
  three nested desktops and two subtree close/reopen cycles per boot:
  `desktop/20261020T163904829764Z`.
- Workspace host tests pass, including all 39 matrix tests. The 22 Python
  harness tests, default userspace clippy, stock matrix nested/trace/probe/fault
  feature clippy, layout and Rust format checks pass.
- A disposable headless boot using the interactive stdio chardev reaches the
  user picker with the readiness marker in both terminal output and the live
  serial file: `interactive-log-check/20261020T164355066761Z`.
- Ordinary desktop fresh/cold behavior and the independent 541-record storage
  oracle pass at `desktop/20261020T164622379795Z`. An earlier attempt at
  `desktop/20261020T164407521583Z` lost its host Python process without a
  traceback or test result, leaving its QEMU child alive; that owned orphan was
  stopped before the successful rerun. No guest panic or matrix-exit marker was
  recorded in the interrupted attempt; the host termination cause is unknown.
- The existing headless matrix protocol test passes (`matrix-test`): identical
  worlds, local views, attenuated input and recovery over the legacy connection
  shape. No kernel behavior changed; kernel smoke and ARM64 were not rerun.

## Slow interaction replies preserve live matrices

The owner's next normal F5 run still lost inner desktops after the fairness
change. The captured live log is
`build/boot-harness-rs/interaction-delay-owner/serial.log`: a Background unwrap
panics with `-110`, followed by `matrix-service-exit`. Fair dispatch reduced one
cause of delay but did not fix treating a receive deadline as fatal process loss.
The desktop still uses one emulated CPU, 128 MiB and TCG; this repair does not
change those resources, the kernel scheduler or the kernel ABI.

The stock matrix now progresses one asynchronous upstream `POLL_TAGGED` through
its service loop. A connection-local, non-reused request ID associates each
reply with its pending snapshot; it grants no authority. Mismatched
replies are ignored. After the diagnostic threshold it retains its view, focus
and input ordering while waiting for that same reply and serving local child queries.
Deferred mutations stay bounded per stream. Separate storage work progresses;
completed program reads wait for interaction ownership before child admission.
Ready deferred requests precede another poll; requests blocked on independent
storage do not prevent consuming an interaction wake.

Asynchronous presentation also retains its original transaction and readers
across delay. Other serialized interaction calls, image uploads, physical input
forwarding and service acquisition keep waiting for their original response
instead of exiting or retrying the operation. None of these changes retries
ambiguous mutations. Storage transaction deadlines and reconciliation are
unchanged. Expired diagnostic timers leave the asynchronous wait set.

The first five-second-stall run survived all three delays but caught a clean
subtree close being refused during an unrelated poll
(`desktop/20261020T170942625177Z`). Simply queuing LOSS behind that poll still
refused a later clean close under load (`desktop/20261020T172247533354Z`).
Read-only loss surveys now progress on their child control links during polling,
provided no completed admission is waiting. Publication still defers them.
A close reply stays queued until teardown can publish erasures; its ready
receiver is excluded while teardown is blocked. Dirty, missing and uncertain
replies still cannot authorize discarding work. Busy surveys, uncertain child
answers and refused close now have distinct diagnostic traces.

The next run passed its fresh boot but lost a Backspace after the cold boot's
delay sequence (`desktop/20261020T171509381534Z`). Clearing focus on a diagnostic
timeout manufactured an input discontinuity despite retaining the original
ordered reply. Lateness now preserves focus; real upstream cancellation and
queue overflow still cancel input. The test-only late-poll hook wakes children
to prove local queries can complete during the held upstream request.

A later loaded run lost three letters from the initial four-key entry before
any injected delay (`desktop/20261020T172943269969Z`). A host regression confirmed
that a click plus four press/release pairs overflowed the eight-entry child
queue before draining. The queue now stores only the four event fields instead
of full eight-word reply packets, fitting sixteen events in the same 512 bytes.
Replies reconstruct their status, current epoch and padding; grant changes
already clear queued input before advancing the epoch. Motion coalescing and
bounded overflow cancellation retain their ordering and sibling-isolation tests.

`desktop_test.py --nested-desktop --pointer-stress --interaction-delay --timeout 240`
compiles test-only F8/F7/F6 gates at the root. They retain a poll, image upload
and compact publication for 500 guest ticks each. The poll gate first sends an
unrelated DENIED reply; the test requires rejection of that reply, child polling
during the delay, eventual recovery, preserved Notes contents and dirty state,
Colors publication, clean close, subtree reclamation and cold persistence.
Ordinary F5 has no gates. The interactive disk was not reset or changed; use
**Cathedral: Fresh Lab in QEMU** for corrected executables throughout copied
user realms.

Remaining scope: only polling has explicit wire request IDs. Other interaction
operations serialize one outstanding request; this is not a multiplexed RPC
transport. Startup and individual surface exchanges can still block dispatch,
and a live provider that never responds can leave them waiting indefinitely.
Terminal transport errors and malformed replies remain failures. Further work
should make those exchanges resumable and provide recoverable unavailable state
without losing drafts; SMP or larger budgets do not replace that work.

Verification:

- Workspace host tests pass; the matrix suite now has 41 passing tests, including
  tagged replies and the burst regression (observed failing before queue compaction).
  All 22 Python harness unit tests pass. Default userspace and
  matrix nested/delay/trace/probe/fault feature clippy checks pass, as do layout,
  Rust formatting and whitespace checks.
- Ordinary F5 fresh/cold behavior, three independent Notes drafts, close/save/
  discard, Colors and the independent 553-record storage oracle pass:
  `desktop/20261020T173847824501Z`.
- Three nested desktops under continuous pointer motion pass on fresh and cold
  boots with all three five-second delays, mismatched poll rejection, local
  polling during the stall, uninterrupted keyboard focus, draft protection and
  two subtree close/reopen cycles per boot:
  `desktop/20261020T173552869333Z`.
- The legacy two-endpoint matrix protocol fixture passes (`matrix-test`).
  Provider replacement, ambiguous writes, independent roots and exact cold
  storage records pass at `matrix-database/20261020T172005490015Z`.
- No kernel behavior changed; kernel smoke and ARM64 were not rerun.


## Scheduler timing preparation and periodic contention (2026-10-20)

Baseline: `e94a9ef0`. The shared scheduler now maintains active ownership
branches instead of rebuilding readiness over every reserved account. Each
selection visits active direct-child contenders along its descent and preserves
the previous slot-ordered rotation. State/gate changes update the affected
ancestor chain. Local exhaustion hides a whole subtree without rewriting every
descendant; reattachment, restart and slot reuse preserve rotation and debt.

CPU accounting charges only the running ancestor chain and accounts whose
repayment deadlines are due. Idle credit refills lazily, including before a rate
change; running refill/consumption remains one net operation. Indexed heaps hold
one deadline per sleeper/exhausted account, with explicit removal on sleep
cancellation. Ready/finished queries use maintained state. Storage is allocated
at construction, with no allocation in dispatch/accounting/wakeup. This adds
linear metadata per reserved slot in exchange for removing dormant-slot scans.

Exact existing policy tests still pass. New regressions compare 20,000 mixed
readiness/gate/reparenting operations with the old selection model and 10,000
CPU samples/rate/tree changes with independent eager accounting. Another 20,000
heap mutations compare expiry/removal with an ordered model across the u64
boundary. A three-account workload inspects the same scheduler work in a
three-slot and a 50,000-slot arena; 10,000 future sleepers require only one root
inspection per idle dispatch. These host tests do not raise the OS task limit
or establish constant-time selection for a large *active* sibling set.

The x86 smoke suite now includes a periodic userspace task beside busy and
yielding peers, then with an intentionally inadequate 1,000-us CPU ceiling.
Sixteen jobs use absolute releases eight nominal 100-Hz ticks apart and consume
at least 1,000 accounted microseconds each. Samples are buffered before serial
output. In this run both uncapped modes started and finished in the release
tick (coarse observations, not zero elapsed time); the capped mode reached 61
ticks of start lateness and 73 ticks of response, with 15 of 16 jobs finishing
past the next period. Peer progress, cap enforcement and complete reclamation
passed. This is an overload/timing baseline, not guaranteed service admission.
Serial records are in `build/boot-harness-rs/smoke/serial.log` (`CPU_TIMING`).

Read the [timing audit](source-rs/kernel/core/scheduler/timing.md) before extending
this work. Trap dispatch still scans IPC/readiness/device/task waiters and
pending work. Boot admission, whole page operations and descendant reclamation
still run with IRQs masked; the heap and polling diagnostic/device paths also
need bounded-time review. The pinned session's exclusive borrow forbids simply
enabling interrupts inside those operations. Next: event-owner waiter indexes,
explicit consistent suspension points for long operations, then a separately
admitted restricted timing profile. Ceilings still are not reservations.
Compiler requests, trusted evidence and runtime grants remain distinct; no
Omega certificate ABI, priority policy or real-time guarantee was invented.

Verification: workspace host tests; all 26 scheduler tests;
default UEFI and smoke-feature clippy, supervision-userspace clippy; Rust format,
layout and whitespace checks; x86 release smoke including the new periodic lab;
ARM64 release portability proof. Logs: `build/scheduler-host-tests.log`,
`build/scheduler-policy-tests.log`, `build/scheduler-smoke.log`,
`build/scheduler-arm64.log`.

The nested desktop pointer stress run passed its fresh boot, deepest saved
Notes, dirty-close protection and two subtree close/reopen cycles, but the cold
boot refused a clean outer close during the second cycle:
`desktop/20261020T185301025767Z`. The log records `matrix-close-refused` and
recursive `matrix-loss-uncertain` with TIMED_OUT, without kernel panic or matrix
exit. Child control LOSS/CLOSE still use `time::after(100)` per hop; an uncertain
answer correctly cannot authorize discarding the subtree. This is an outstanding
timing-sensitive close failure, not a claimed passing full two-boot stress run.

An unchanged revised-kernel cold replay on a copy of that exact saved fixture
passed all three levels, both teardown/reopen cycles and isolated outer Notes:
`desktop/scheduler-cold-recheck`. The same copied saved fixture also passed with
the preserved pre-change EFI from `desktop/20261020T173847824501Z`, at
`desktop/scheduler-baseline-cold`. Neither replay reproduced the refusal, so its
relationship to this scheduler change is not established.
Keep this regression visible; do not silently increase
control deadlines, treat a timeout as permission to discard, or weaken the
fixture to hide it. All desktop checks use disposable/copied media; the owner's
interactive disk has not been modified.

## Event-targeted wait completion (2026-10-20)

Baseline: `46cd674a`. Start at
[waits.rs](source-rs/kernel/core/user_tasks/execution/waits/waits.rs), then its
[routing guide](source-rs/kernel/core/user_tasks/execution/waits/README.md).
Syscalls and interrupts no longer probe every task's IPC, readiness and child
waits. IPC emits coalesced receiver hints for message delivery, revocation,
peer/grant closure, escrow discard, receive-right movement and boot rebinding.
Device policy targets installed owners; reclamation targets subscribers to the
changed child launch. A preallocated indexed heap selects only due wait timers.
Validated physical device routes and the keyboard owner are cached at startup.

Hints carry no authority or result. The existing syscall owners still validate
live grants and tickets, check copyout, and prefer queued data or terminal
results over expiry. Whole-set readiness validation is unchanged. Cancellation
removes subscriptions and timers before slot reuse; replacement cannot inherit
an old child wait or device hint. Registration always follows a current-state
check, avoiding lost readiness when an earlier hint was drained.

This is receiver-level indexing, not one subscription per endpoint. An unrelated
endpoint of the same task may cause a harmless bounded recheck. Bitmaps retain
the current 31-task arena limit; no OS capacity increase is claimed. Wake routing
allocates no storage, but simultaneous completions still require proportional
work. No kernel ABI, platform policy, distribution behavior, priority scheme or
reservation contract changed.

Verification:

- Workspace host tests pass; all 126 core tests pass after adding event-route,
  device replacement, cancellation/reuse, wraparound and populated-waiter
  regressions. IPC coverage includes byte/memory sends, moved receive rights,
  escrow closure and boot-pair replacement.
- Default UEFI and smoke-feature clippy pass; source layout, Rust formatting and
  whitespace checks pass. x86 release smoke and ARM64 portability proof pass.
  Logs: `build/waiter-host-tests.log`, `build/waiter-core-tests.log`,
  `build/waiter-clippy.log`, `build/waiter-smoke-clippy.log`,
  `build/waiter-smoke.log`, `build/waiter-arm64.log`.
- Three nested desktops under continuous pointer motion pass on both fresh and
  cold boots, including delayed/mismatched polling, deepest saved Notes,
  dirty-close protection, two subtree teardown/reopen cycles per boot and
  isolation: `desktop/20261020T191448080068Z`.
  This run did not reproduce the prior intermittent clean-close LOSS timeout;
  its cause remains unresolved, and the prior section's evidence still applies.
  Interactive saved media was not modified.

Next: index deferred admission/reclamation work, then introduce explicit
consistent suspension points for long IRQ-masked operations. Follow the
[timing audit](source-rs/kernel/core/scheduler/timing.md). Remaining scans,
grant lookup, heap behavior and device polling still preclude a bounded latency
claim. The exclusive session borrow still forbids simply enabling interrupts
inside ongoing operations. Owner decisions are not blocking these mechanics;
a separately admitted timing profile and its policy remain future work.

## Indexed deferred work and incremental allocation (2026-10-20)

Baseline: `6f73a85c`. Follow the new
[deferred-work entrance](source-rs/kernel/core/user_tasks/execution/deferred/deferred.rs)
and its [suspension/lifetime guide](source-rs/kernel/core/user_tasks/execution/deferred/README.md).
Trap-side pending detection no longer scans task/launch arenas or retired-memory
words. Accepted work populates fixed sets for admission, memory, cancellation,
retired tasks and loan return. Admission and memory steps rotate across pending
slots. Teardown clears queued calls/admissions before reuse, drains newly added
descendant cancellations, then returns loans only after physical reclamation.
Cached child-to-launch routes survive lending and dynamic reservation reuse.
Executable-reference and IPC escrow retirement visit marked entries rather than
every entry; their bounded bitmap-word iteration remains.

A boot step reclaims, admits at most one child and advances at most one memory
caller. After it returns, with no mutable session borrow or allocator lock held,
an explicit interrupt window and scheduler yield may run peers even while work
remains. A peer's next trap returns to boot for progress. An asynchronous boot
IRQ cannot bypass pending cleanup. This preserves deferred work on the boot
stack and avoids an artificial timer-tick delay when every caller is blocked.

Private allocation now reserves quota/funding once, acquires and zeros at most
eight pages per step, and retains unpublished progress in session state. It
returns no handle and exposes no mapping until all pages are ready. Each batch
is charged to its requester. Cancellation/failure refunds unbacked reservations
and allocated frames exactly once, before account reuse. Conservation checks
include reserved-but-unbacked pages. Final mapping and rollback remain whole
operations. The allocation batch size is a work allowance, not a latency promise.

Review caught a race newly possible across queued admissions: a consenting peer
can exit while another boot step runs. Admission now rechecks its original
consent, returning `PEER_CLOSED` before allocating/publishing a child instead of
reaching the live-peer assertion in IPC boot-pair installation. A replacement
task does not inherit that consent. Owner death removes the queued request.

Verification:

- Workspace host tests pass; all 130 core tests pass after the final peer-consent
  regression. Pending-set tests cover coalescing, rotating requeue, cancellation,
  reuse, newly queued descendants and boot/user routing.
- The real-frame rollback fixture fails maximum-size allocation at every frame
  boundary and cancels it at all 15 partial eight-page boundaries. Every case
  checks exact funding/frame recovery; partial batches expose no writable map.
  x86 release smoke passes, including IPC deadlines, cancellation/restarts,
  hierarchical CPU contention, memory faults and final heap/frame reclamation.
- ARM64 release portability passes; its private-memory peers now allocate full
  128-page regions through multiple steps, check zeroing/private data across
  preemption, and reclaim them after peer faults. ARM64 also runs the partial
  allocation rollback fixture.
- Default UEFI, smoke-feature and ARM64 portability-program clippy pass; layout,
  Rust formatting and whitespace checks pass. Logs are `build/deferred-*.log`.
- Three nested desktops under continuous pointer motion pass on fresh and cold
  boots, including deepest saved Notes, protected close, repeated teardown/reopen
  and isolation: `desktop/20261020T193010071854Z`. This desktop run preceded only
  the final queued-peer-consent check and test-only ARM64 expansion; the final
  kernel also passed x86 smoke. The earlier intermittent LOSS timeout did not
  recur and remains unresolved. Interactive saved media was not modified.

Next: make ELF construction and physical subtree reclamation resumable, with
private progress and cancellation state before further suspension boundaries.
Map/seal/unmap, allocation rollback, heap behavior and large event batches also
remain non-preemptible. The [timing audit](source-rs/kernel/core/scheduler/timing.md)
tracks these limits. This work does not change application ABI, platform policy,
distribution UI, CPU priorities or reservation semantics, and makes no measured
desktop-speedup or real-time guarantee. No owner decision blocks the next
mechanical steps.


## Resumable executable admission (2026-10-20)

Baseline: `8555aed8`. Live launch now follows
[lifecycle/spawn.rs](source-rs/kernel/core/user_tasks/lifecycle/spawn.rs): validate
current authority, advance a private draft, then prepare and commit its grants.
[Image decoding](source-rs/kernel/core/user_tasks/admission/image.rs) translates
today's ELF/probe input into segment descriptions. The resumable page engine,
funding, cancellation and publication do not depend on ELF as the future format.
No new executable or application ABI is established.

The first step creates an inactive root. Subsequent steps initialize/map at most
eight image or stack pages, then prepare runtime tables and any framebuffer in
separate phases. Launch state retains the immutable source and private draft;
no runnable context, task root or transferred endpoint appears before the final
serialized commit. Every step rechecks original peer consent and initial rights.
Delegation/grant plans and epochs are prepared after the last yield. Frame use
is charged incrementally to the child, CPU work to the requesting owner.

Failure or owner/peer death refunds all draft frames. Returning a dead owner's
launch first aborts any draft; no surviving lender means its executable-source
reference is also released. Conservation includes unfinished roots. Initial boot
admission remains synchronous. Decoding, kernel-root construction, runtime table
reservation, framebuffer mapping, final commit and rollback are still whole
operations; the batch size is a work allowance, not a latency guarantee.

Verification and findings:

- Workspace host tests pass, including 131 core tests. The new cursor regression
  covers partial pages, gaps, differing batch sizes and the stack without skips
  or duplicates.
- The real-frame admission fixture runs on x86 and ARM64. It fails at every frame
  allowance, withdraws at every stage boundary, checks BSS/permissions and proves
  that grants stay with the owner until commit. Production lifecycle cases cover
  owner/peer death, a queued message making an initial receive right unmovable,
  and reclamation of an abandoned object-backed source.
- Large by-value fixture sessions overflowed boot-stack headroom during development;
  the retained fixture session is now heap-owned and scenarios have separate stack
  frames. The fixture also avoids assuming syscall result registers match x86.
  ARM64 exception entry still saves onto the interrupted stack before switching
  stacks: audit stack-exhaustion reporting/emergency entry separately. The observed
  ARM fault report was delayed and did not identify the initial overflowing frame.
- The nested desktop harness incorrectly required dirty state after cold-open
  typing followed by deletion restored the saved document. Its screenshot showed
  the correct text and clean background; the expectation now distinguishes fresh
  unsaved content from cold-open confirmed content. Failure evidence:
  `desktop/20261020T200328028156Z`.

- Release x86 smoke passes with `--timeout 90`, including IPC, contention,
  cancellation/restart, display protection and final heap/frame reclamation.
  The first complete-suite attempt exceeded the default 30-second harness budget
  while still progressing; this does not change any guest protocol deadline.
- Release ARM64 portability and the executable-authority integration suite pass.
  Default x86 UEFI, smoke-feature x86 UEFI and smoke-feature ARM64 UEFI clippy pass,
  along with layout, formatting and whitespace checks.
- Logs: `build/admission-host-tests.log`, `build/admission-clippy.log`,
  `build/admission-smoke-clippy.log`, `build/admission-smoke.log`,
  `build/admission-arm64.log`, `build/admission-authority.log`.

- Fresh and cold desktop boots pass with three nested desktops, continuous pointer
  motion and deliberately delayed/mismatched interaction replies. Deepest Notes,
  clean/dirty close, persistence, isolation and two teardown/reopen cycles per boot
  pass: `desktop/20261020T201043556840Z`, `build/admission-desktop.log`.
  Interactive saved media was not modified.

The previously recorded intermittent clean-close LOSS timeout remains unresolved;
this milestone does not weaken its assertions or change protocol deadlines.

Next: bound subtree reclamation and mapping/rollback phases, preserving authority
withdrawal before physical reuse and ancestor funding return. Measure remaining
uninterrupted phases under competing work before making timing claims. Follow the
[timing audit](source-rs/kernel/core/scheduler/timing.md). No owner decision blocks
that mechanical work; guaranteed-service admission remains a separate design step.


## Incremental subtree reclamation (2026-10-20)

Baseline: `1428451e`. Start at
[lifecycle.rs](source-rs/kernel/core/user_tasks/lifecycle/lifecycle.rs), which
withdraws canceled authority, advances physical cleanup, then checks conservation.
[Reclamation](source-rs/kernel/core/user_tasks/lifecycle/reclamation.rs) separates
stopping tasks from collecting their slots and publishing outcomes.

Cancellation still stops the complete affected subtree and closes endpoints,
waits and device routes before the first yield. Physical retirement then rotates
among dead tasks. Alias cleanup scans at most 32 reader/region entries and cleans
at most one page object per step. After aliases disappear, each step returns at
most eight private image/stack/table frames. Abandoned admission drafts use the
same eight-frame allowance. Funding refunds track the remaining frame ledgers;
retirement CPU work is charged to the dead task, draft cleanup to its requester.
Accepted live readers retain backing after producer death.

Dead slots remain Exited until the entire cleanup wave finishes. Final metadata
collection publishes child outcomes, returns launch loans and completes blocked
cancellation callers. Admission waits behind cleanup and pending executable/IPC
reference refunds; existing users and their IPC/memory work can continue between
steps. Preventing new admissions also prevents a stream of new tasks from keeping
one cleanup wave alive indefinitely. The boot loop now drains pending cleanup
even after the last live task exits. No application ABI or distribution policy
changed.

Remaining large operations: complete authority withdrawal over the current
31-slot arena, final metadata collection, conservation assertions, individual
page-object unmap/free, abandoned private-allocation rollback, pending-reference
settlement and the previously recorded loader phases. This is a work allowance,
not a certified time bound. New launches can wait behind unrelated cleanup;
per-subtree collection dependencies are a future refinement if measurements
justify the extra state.

Verification:

- Workspace host tests pass, including 132 core tests. Pending-set coverage now
  checks that physical retirement, collection and abandoned drafts all hold the
  admission gate until their work is removed.
- The real-frame fixture cancels a committed parent/child and checks every step:
  at most eight private frames returned, endpoints withdrawn before the first
  pause, unaffected peer still Ready, both slots unavailable and child outcome
  unpublished until collection. Final frame/funding refunds are exact. Existing
  abandoned-source and interrupted-admission tests now drain multiple steps.
- Release x86 smoke and ARM64 portability pass with staged teardown, including
  live shared readers, peer faults, repeated cancellation/restarts and complete
  heap/frame reclamation. Executable-authority and IPC channel/startup-bundle
  integration suites pass. Default x86, smoke-feature x86 and smoke-feature ARM64
  UEFI clippy, layout, formatting and whitespace checks pass.
- Fresh and cold three-level desktop runs pass with continuous pointer motion,
  delayed/mismatched interaction replies, Notes persistence, clean/dirty close,
  isolation and two subtree teardown/reopen cycles per boot:
  `desktop/20261020T211451406375Z`. This preceded only the final gate holding
  admission until pending executable/IPC reference refunds settle; the final
  kernel is checked by the x86/ARM64 and executable-authority suites.
  Interactive saved media was not modified. The earlier intermittent clean-close
  LOSS timeout did not recur; its cause remains unresolved.
- Logs: `build/reclamation-host.log`, `build/reclamation-clippy.log`,
  `build/reclamation-smoke-clippy.log`, `build/reclamation-arm64-clippy.log`,
  `build/reclamation-smoke.log`, `build/reclamation-arm64.log`,
  `build/reclamation-desktop.log`, `build/reclamation-authority.log`,
  `build/reclamation-ipc.log`.

Next: measure the largest remaining masked phases under competing work, then
split single-object unmap/free and pending-reference retirement where those
measurements warrant it. Retain the ARM64 stack-exhaustion entry/reporting audit
from the previous milestone. Guaranteed-service admission remains separate from
these mechanisms; no new owner decision is needed for this cleanup work.

## Kernel-phase observations and single-pass page edits (2026-10-20)

Baseline: `09c1e774`. [Execution timing](source-rs/kernel/core/user_tasks/execution/timing/timing.rs)
buffers selected uninterrupted Rust-body spans, including nested trap/deferred
work, admission construction, reclamation and individual memory operations.
Only `memory-lab` builds collect them; normal F5 builds have no additional sample
storage or clock reads. Reports print after the session stops, outside measured
work. The [timing audit](source-rs/kernel/core/scheduler/timing.md) describes the
sampling boundaries and [report tool](tools/boot-harness-rs/timing_report.py).

The periodic probe now has two additional competitors: repeated one-page and
maximum-size allocations, freeze, unmap, remap and release. Both leave four owned
objects at exit, two unmapped, so actual task cleanup is measured too. The first
development fixture tried to keep four maximum-size objects mapped, exceeding
the separate address-space mapping ledger limit. That fixture was corrected;
no production capacity was enlarged to accommodate the test.

Measurements plus inspection identified repeated ledger searches during seal
and unmap. [Region edits](source-rs/kernel/core/user_tasks/address_space/regions.rs)
now traverse the ledger once; unmap compacts it while removing translations.
Both remain atomic. There is no new suspended mapping state, handle behavior,
authority rule or scheduling policy. The real-frame fixture checks every removed
page, every retained owner page, read-only state and surviving writable stack,
including table-boundary and last-slot aliases on both ISAs.

The complete baseline and final x86 smoke runs pass. Host load makes their elapsed
maxima and averages too variable for a reliable speedup claim (including spikes
in tiny operations). The structural improvement is removal of repeated scans;
this does not establish a desktop latency gain or a real-time bound. The combined
decode/root-creation phase repeatedly registers millisecond-scale work and is
the next useful place to narrow instrumentation before adding more state machines.
Initial boot admission and assembly entry/exit are outside the runtime samples.

Verification:

- Workspace host tests pass, including 132 core tests. Default x86 UEFI and
  smoke-feature x86/ARM64 clippy pass. Release x86 smoke and ARM64 portability
  pass, including real page tables, permissions, shared readers, failure rollback,
  subtree cleanup and full heap/frame refunds. Layout, formatting and whitespace pass.
- Fresh and cold three-level desktop runs pass with pointer stress, delayed/
  mismatched interaction replies, deepest Notes persistence, protected close,
  isolation and two teardown/reopen cycles per boot:
  `desktop/20261020T224834092056Z`, `build/phase-desktop.log`. This exercises the
  ordinary build with timing collection disabled. Interactive saved media was
  untouched. The earlier intermittent LOSS timeout did not recur.
- Logs: `build/phase-host-final.log`, `build/phase-default.log`,
  `build/phase-clippy-final.log`, `build/phase-arm64-clippy.log`,
  `build/phase-smoke-baseline.log`, `build/phase-smoke-final.log`,
  `build/phase-arm64.log`. Preserved observations:
  `build/phase-baseline-serial.log`, `build/phase-final-serial.log`.

Next: separate executable decoding from root/table construction costs, then
bound whichever phase warrants it. Pending-reference bursts and single-object
cleanup remain atomic. Keep the ARM64 exhausted-stack entry/reporting audit and
the intermittent clean-close LOSS timeout open. Guaranteed-service admission is
still separate; these mechanical changes need no owner decision.

## Resumable supervisor mappings during admission (2026-10-20)

Baseline: `fab5207d`. Separating `Decode` and `Root` observations identified x86
supervisor mapping construction as the substantial part of the old `DecodeRoot`
phase. In the baseline's 32-restart fixture, decoding totaled 1,363 us and root
construction 214,587 us. These are QEMU elapsed observations, affected by host
scheduling and calibration, not native cost bounds.

[Construction](source-rs/kernel/core/user_tasks/admission/construction.rs) now
allocates a private root, then prepares its supervisor mappings before loading
user pages. Core supplies a 64-leaf work allowance; the
[x86 backend](source-rs/kernel/arch/isa/x86_64/user/memory/creation.rs) owns the
boot-root address, kernel image/heap/stack ranges and cursor. Each step preserves
the original supervisor permissions. No mapper or mutable page-table borrow
survives the return to boot. The root accessor refuses an unfinished supervisor
tree. ARM64 retains its existing permanent supervisor-branch borrow and completes
the same preparation interface immediately. Initial boot admission runs this
engine synchronously; ordinary child launches may yield between batches.

All partial table allocations stay in the existing core frame ledger. Funding is
charged after each step and refunded on refusal or owner/peer death. Source and
initial-grant custody, revalidation and final publication remain unchanged.
This is a smaller uninterrupted work unit, not less total work: in the final
32-restart sample, bare-root allocation totaled 747 us and mapping preparation
258,996 us spread across 2,112 steps. Both binaries include lab programs in their
kernel image; normal F5 has a different mapping footprint. Admission can take
longer under contention because peers can run between those steps.

The longer startup exposed a test-interval error in the CPU-turn fixture: early
children spun while later siblings were still being built, but the assertion
compared their lifetime preemption counts against an outside peer that began
later. Workers now park on their existing IPC ports until all admissions finish
and the owner starts measurement. The fairness tolerance is unchanged. Final
counts were outside/subtree 64/64 with one spinner and 65/65 with three. The
admission rollback fixture's old fixed 64-step stall guard also had to account
for the embedded kernel image; no production capacity or deadline was enlarged.

Verification:

- Workspace host tests pass, including the new x86 cursor regression over empty
  ranges, gaps, high addresses and differing batch sizes, plus 132 core tests.
- Release x86 smoke and ARM64 portability pass. Physical admission coverage
  fails every frame allowance, cancels at every stage (now including supervisor
  batches), checks unpublished authority and exact refunds, and checks owner/
  peer loss and retained executable-source reclamation. It verifies multiple
  x86 supervisor steps and one ARM64 preparation step.
- Executable-authority integration passes: recursive funded pools, executable
  capabilities, failed loads, stale grants and surviving siblings. Default x86,
  smoke x86/ARM64 and supervision-guest clippy pass. Layout, formatting and
  whitespace checks pass.
- Fresh and cold three-level desktop runs pass with pointer stress, deepest
  Notes persistence, protected close, isolation and two teardown/reopen cycles
  per boot: `desktop/20261020T232137338052Z`, `build/root-desktop.log`. This
  exercises the ordinary build with timing collection disabled. Interactive
  saved media was untouched. The intermittent clean-close LOSS timeout did not
  recur; delayed interaction replies were not enabled in this run.
- Logs: `build/root-host.log`, `build/root-baseline.log`,
  `build/root-baseline-serial.log`, `build/root-smoke-final.log`,
  `build/root-final-serial.log`, `build/root-arm64.log`,
  `build/root-authority.log`, `build/root-default.log`,
  `build/root-clippy-final.log`, `build/root-arm64-clippy.log`,
  `build/root-guest-clippy.log`. The initial test-interval failure is retained in
  `build/root-smoke.log` and `build/root-pre-barrier-serial.log`.

Next: use the now-separated phases to choose among runtime-table preparation,
framebuffer mapping, rollback and reference-retirement bursts. Do not infer a
latency guarantee from the work allowance. The ARM64 exhausted-stack entry audit,
intermittent clean-close LOSS timeout and guaranteed-service admission remain
open; no owner decision was needed for this mechanical step.

## Staged framebuffer admission and retirement observations (2026-10-20)

Baseline: `929db9c8`. The next audit found a sampling problem: `RegionTables` and
`Framebuffer` counted admissions that requested neither mapping. Those no-op
samples no longer dilute the reports. The real-frame admission fixture now
reports a complete private construction and, on x86, receives the actual reserved
GOP aperture. Initial boot admission remains outside these runtime samples.

[Construction](source-rs/kernel/core/user_tasks/admission/construction.rs) now
advances framebuffer mapping by at most 64 device leaves per step. Core's
[framebuffer coordinator](source-rs/kernel/core/user_tasks/address_space/framebuffer.rs)
retains the byte offset and tracks table allocations; the
[x86 device backend](source-rs/kernel/arch/isa/x86_64/user/memory/device.rs)
validates each slice and preserves RW/NX, guards and uncached PAT policy. Device
backing never becomes owned RAM or accessible through ordinary checked-copy
helpers. The draft stays private and unrunnable until all mappings and the final
authority commit complete. Every step retains the existing owner/peer and initial
endpoint revalidation, CPU charging and physical funding. Partial failure or
withdrawal refunds all owned tables, never the device aperture.

The normal 1024x768 GOP aperture spans 768 pages and now takes 12 mapping steps;
the contract's maximum 16 MiB aperture takes at most 64. Initial boot uses the
same engine synchronously. ARM64 keeps its explicit unsupported framebuffer
result; no display backend, ABI, deadline or capacity changed. The framebuffer
coordinator is a named sibling of image construction and page-object mappings;
`mod.rs` remains plumbing.

A new [reference-retirement fixture](source-rs/kernel/core/user_tasks/memory/references_test.rs)
creates 70 real objects across bitmap words: 65 one-page objects and five
128-page objects, each held by two kernel references. It drops those references
through production settlement, verifies exact backing/funding refunds and keeps
one sealed owner mapping readable. The baseline settled the burst in 30 us;
the final x86 sample was 49 us and ARM64 155 us. Settlement remains atomic: these
observations do not justify prioritizing another state machine there yet, and
this is not a full-arena worst case or a competing-workload measurement.

The new complete-construction sample observed runtime-table preparation at
140 us on x86 and 134 us on ARM64. x86 framebuffer preparation totaled 828 us
across 12 batches, with a 412 us maximum. Earlier successful whole-framebuffer
samples were in the hundreds of microseconds; batching does not remove work,
and host scheduling makes these unsuitable for a speedup or latency-bound claim.
The reason for this split is the explicit cap on uninterrupted leaf edits.

Verification:

- Workspace host tests, default x86 UEFI clippy and smoke-feature x86/ARM64 clippy
  pass. Layout, formatting and whitespace checks pass.
- Release x86 smoke passes. Real-frame admission cancels at every construction
  boundary, now including each framebuffer batch, and fails every physical frame
  allowance while checking unpublished context/IPC state and exact refunds.
  Display probes still pass NX/guard faults, checked-copy rejection, provider
  restart and every device-table allocation failure. ARM64 portability passes,
  including admission rollback and the new reference burst.
- Fresh and cold three-level desktop runs pass with pointer stress, protected
  close, deepest Notes persistence, isolation and two teardown/reopen cycles per
  boot: `desktop/20261020T234449530073Z`, `build/aperture-desktop.log`. These use
  ordinary builds with timing collection disabled. The interactive saved disk
  was untouched. The intermittent LOSS timeout did not recur; delayed interaction
  replies were not enabled in this run.
- Logs: `build/burst-baseline.log`, `build/burst-baseline-serial.log`,
  `build/aperture-smoke.log`, `build/aperture-final-serial.log`,
  `build/aperture-host.log`, `build/aperture-clippy.log`,
  `build/aperture-default.log`, `build/aperture-arm64-clippy.log`,
  `build/aperture-arm64.log`.

Next: runtime-table preparation still runs atomically; ARM64 currently walks
all leaves to reserve tables where one visit per leaf-table span should suffice.
Measure that simplification before adding a cursor. Whole error rollback,
single-object cleanup and reference/escrow bursts remain bounded only by their
capacity limits. Keep the ARM64 exhausted-stack entry/reporting audit and the
intermittent clean-close LOSS timeout open. Guaranteed-service admission remains
a separate design decision; this milestone required no owner decision.

## ARM64 runtime-table reservation by span (2026-10-20)

Baseline: `601edf25`. [ARM64 user mappings](source-rs/kernel/arch/isa/aarch64/user/mappings.rs)
now reserve one address per 2-MiB leaf-table span. Advancing to the next boundary
also handles a start inside a table. For the current 52 guarded mapping slots,
this reduces 6,708 walks to 14 while allocating the same tables and installing
no user leaves. No new cursor, ABI, capacity or scheduling policy is needed;
reservation remains atomic. Core retains physical custody and rollback.

The shared [real-frame fixture](source-rs/kernel/core/user_tasks/memory/rollback.rs)
now repeats reservation and checks that no frames were added, then maps/unmaps
every reserved slot with a maximum-size object. Runtime mapping prohibits table
allocation, so an omitted table fails directly. The test checks each page's
contents and read-only state, guard rejection, removal of every alias, and
preservation of the owner mapping and stack. Existing failure injection still
covers every admission allocation and exact frame/heap refunds.

The complete-construction ARM64 sample recorded runtime-table preparation at
121 us before and 50 us afterward. Other samples varied upward as well as down;
these host-sensitive observations are not a latency bound or a reliable speedup
ratio. The defensible improvement is removal of 6,694 redundant table walks.

Verification: release ARM64 baseline and final portability suites pass,
including the expanded mapping test and all 66 admission failure boundaries.
Release x86 smoke also passes the expanded shared fixture. Workspace host tests,
x86/ARM64 smoke-feature clippy, layout, formatting and whitespace checks pass.
Production changes are ARM64-only; the nested graphical desktop test was not
repeated, and interactive saved media was untouched. Logs:
`build/reserve-arm64-baseline.log`, `build/reserve-arm64-final.log`,
`build/reserve-host.log`, `build/reserve-arm64-clippy.log`,
`build/reserve-x86-smoke.log`, `build/reserve-x86-clippy.log`.

Next: prioritize the outstanding ARM64 exhausted-stack entry/reporting audit.
Runtime table reservation is now a small fixed number of walks at current
limits. Whole error rollback, single-object mapping/cleanup, reference/escrow
bursts and event bursts remain atomic; do not claim bounded-time execution.
The intermittent clean-close LOSS timeout and guaranteed-service admission stay
open. No owner question was needed for this simplification.

## ARM64 stack-safe exception entry and terminal reporting (2026-10-20)

Baseline: `1fe5af72`. The previous entrance saved its 800-byte context onto the
interrupted EL1 stack before switching to the permanent IRQ stack. A new probe
moved SP into the boot guard and faulted on an identified store; the baseline
timed out without a fault report (`build/stack-before.log`,
`build/stack-before-serial.log`). This reproduces the earlier diagnostic gap.

[Entry assembly](source-rs/kernel/arch/isa/aarch64/context/entry.S) now saves
normal contexts directly into the reserved top of the permanent IRQ stack.
A transient `TPIDR_EL1` scratch value preserves x0 while x1 and the original SP
are saved there; the register is reserved by this backend and cleared before
Rust dispatch. No store depends on the interrupted kernel stack. Register layout,
EL0 SP handling, shared scheduling and return-root selection remain unchanged.

Fatal EL1 synchronous faults take a separate assembly entrance before any normal
context save. It captures the original ESR/FAR/ELR/SP, switches to a distinct
permanent fault stack, and calls the existing terminal reporter. FIQ/SError also
use the terminal entrance. A second fatal fault parks without recursively
restarting reporting. Returning BRK/SVC/IRQ paths retain ordinary dispatch, and
EL0 faults still go through core's process-containment path. This remains a
single-CPU EL1h profile with masked dispatch and no nested resumable callbacks.
Kernel exhaustion is terminal; emergency-stack/mapping corruption is not recovered.

The new [boot probes](source-rs/kernel/boot/uefi/smoke/stack.rs) require the exact
faulting instruction, guard address, original SP and reporting-stack range for
both boot-stack and interrupt-stack exhaustion. A positive probe gives BRK, SVC
and a real timer IRQ only 16 bytes of interrupted-stack headroom, checks x0/x1
preservation, and returns to the original caller stack. Run with the ARM64
harness's `--fault stack-exhaustion`, `--fault irq-stack-exhaustion`, or
`--fault stack-entry`; these probes are absent from normal boots.

Verification:

- All three targeted probes pass in release and debug builds. The seven existing
  ARM64 guard/code/data/NX probes pass in release. The full release portability
  suite passes, including EL1/EL0 register preservation, timer preemption, IPC,
  user-fault containment, admission rollback and full reclamation.
- Workspace host tests, ARM64 smoke/probe clippy, default x86 UEFI clippy, layout,
  formatting and whitespace checks pass. Production code changes are ARM64-only;
  no graphical desktop test or x86 QEMU rerun was needed. Saved desktop media
  was untouched.
- Full debug portability does NOT pass: after the trusted-task tests it reports
  a read translation fault in `__chkstk` before shared-memory userspace work.
  Diagnostic: vector 37, ESR `0x96000007`, PC `0x43b7d31c`, FAR `0x1000000b00`,
  original SP `0x1000006b00`. The guard access occurs during stack probing before
  SP itself moves into the guard. The original instruction was resolved against
  that build's PDB/disassembly; the exact oversized caller remains to be located.
  This is now an immediate terminal report, not a delayed secondary entry fault.
- Logs: `build/stack-exhaustion.log`, `build/stack-irq-stack-exhaustion.log`,
  `build/stack-stack-entry.log`, `build/stack-debug-*.log`,
  `build/stack-arm64-release.log`, `build/stack-arm64-debug.log`,
  `build/stack-debug-overflow-serial.log`, `build/stack-host.log`,
  `build/stack-clippy.log`, `build/stack-probes-clippy.log`,
  `build/stack-x86-default.log`. Other protection logs are `build/stack-*.log`.

Next: trace and reduce debug userspace-session setup's stack pressure (inspect
large by-value Session/result temporaries) without widening stacks or discarding
guards. The exhausted-stack entrance/reporting gap is addressed; the newly
localized debug boot failure is separate. Keep the intermittent clean-close
LOSS timeout open. Atomic cleanup/event bursts and guaranteed-service admission
remain future work; no owner decision was needed here.


## Stack-safe, fallible userspace session startup (2026-10-20)

Baseline: `7331c086`. Full ARM64 debug portability now passes. The previous
`__chkstk` guard read came from stacked startup frames carrying inline session
and IPC startup-handle tables. The old debug userspace runner reserved 28,000
bytes and preparation reserved another 38,912 bytes, before their callees and
boot callers. The stack probe faulted before entering the oversized frame.

[Preparation](source-rs/kernel/core/user_tasks/startup/startup.rs) now returns
an owned heap session. [Arena construction](source-rs/kernel/core/user_tasks/startup/arena.rs)
allocates fallibly and initializes fields directly, without a whole-session
stack temporary. All fallible work precedes raw initialization; no partially
initialized session is published. The owner remains at a stable mapped address
until execution clears the switch handler and `ACTIVE`, then ordinary drop
reclaims its storage. The admission fixture uses that same owner, removing its
extra box/copy. This preserves the existing raw-pointer suspension obligations.

[IPC startup](source-rs/kernel/core/ipc/bootstrap.rs) reserves its initial-handle
rows for the configured task slots, including dormant children, once on the
heap. Rows never grow during a trap. Its allocation refusal now reaches callers
as `OutOfHeap`, not `InvalidEndpoints`. Scheduler accounts/deadlines, waiter
storage and lab timing samples also have fallible constructors used by userspace
startup. The trusted-kernel-task session still uses the existing infallible
`Scheduler::new` wrapper; this is not a whole-kernel allocation-failure audit.

Disassembly of the final ARM64 debug build shows preparation's probed allocation
at 6,432 bytes; the userspace runner no longer needs a stack probe. Other
constructors still have sizable debug frames. This is not a global stack-use
bound. Boot/task stack capacities and guards are unchanged, and the fatal-stack
entry/reporting mechanism from the previous milestone remains intact.

The new shared [startup fixture](source-rs/kernel/boot/uefi/lab/startup.rs)
exhausts the real kernel heap and releases it in 4-KiB increments. Every refusal
must return `OutOfHeap` with exact heap/frame recovery. Once startup fits, a zero
physical-frame allowance must refuse image admission, again refunding everything.
It reaches 73 heap refusals on both ISAs before that boundary. This samples
available capacities rather than injecting a fault at every allocation call.
Subsequent ordinary sessions still boot and reclaim successfully. A host test
also verifies that impossible scheduler capacity returns an error.

Verification:

- Full ARM64 debug and release portability pass, including the new heap-pressure
  fixture, private-process isolation, shared-object lifetime and admission rollback
  (78 frame boundaries in debug, 66 in release).
- Full x86 release smoke passes, including the same heap-pressure fixture,
  supervision, device/display recovery and exact memory reclamation.
- Workspace host tests, default x86 UEFI clippy, x86/ARM64 smoke clippy, layout,
  formatting and whitespace checks pass. ARM64 kernel-only smoke excludes the
  userspace fixture because that build omits the required image metadata.
- Nested desktop regression covers fresh/cold boot, three recursive desktops,
  pointer stress, draft-protected close, confined persistence and repeated subtree
  reclamation. Disposable images only; interactive F5 saved media was untouched.
- Logs: `build/session-arm64-debug.log`, `build/session-arm64-release.log`,
  `build/session-x86-smoke.log`, `build/session-host.log`,
  `build/session-x86-clippy.log`, `build/session-x86-smoke-clippy.log`,
  `build/session-arm64-clippy.log`, `build/session-desktop.log`,
  `build/session-desktop-final.log`, `build/session-debug-stack-frames.txt`.

Next: both the exhausted-stack reporting gap and the resulting debug startup
failure are resolved. Return to the measured atomic cleanup/event bursts,
starting with rollback and single-object mapping/cleanup before adding more
cursors. Keep the intermittent clean-close LOSS timeout open; no failure in this
milestone proves that issue fixed. Guaranteed-service admission remains a
separate owner-level design choice. No new owner decision was needed here.


## Bounded failed-launch rollback (2026-10-20)

Baseline: `e5a13906`. A live process's failed child launch previously released
its entire draft synchronously, including errors after image/table construction
or final authority validation. It now retains the first refusal in the launch
account and uses later admission turns to return at most eight private frames
per visit, through the same frame-release engine as abandoned/dead-task cleanup.
[Spawn](source-rs/kernel/core/user_tasks/lifecycle/spawn.rs) shows the choice
between advancing construction and advancing rollback. No additional cleanup
cursor or ABI is introduced.

The parent remains blocked, the launch remains busy and its child stays
unpublished until all private frames are refunded. Then the original error is
returned and the launch becomes retryable. Initial rights/messages remain with
the parent throughout refusal. Each cleanup step charges the requester and
refunds the child's physical account. If the parent dies mid-rollback, close
clears its refusal, removes pending admission and transfers the partial draft
to ordinary abandonment before returning launch capacity/source custody.
Executable-object references remain pinned until the appropriate owner/loan
cleanup settles them. No partly retired root can resume or serve checked copies.

The [shared real-frame suite](source-rs/kernel/core/user_tasks/admission/rollback/rollback.rs)
now has a small sequencing entrance and named construction, refusal, authority
and fixture leaves. Existing sweeps still refuse every physical-frame allowance
and withdraw at every construction phase, including framebuffer batches on x86;
those tests assert the eight-frame cleanup allowance and exact refunds.
New production-lifecycle cases cover a failure at the last allocation, a busy
initial receive right after complete private construction, blocked-parent and
selectable-peer state between batches, no early launch reuse, successful retry,
and owner death after partial rollback. Object-backed executable tests cover
both interrupted construction and interrupted rollback with final source refund.

Verification:

- Full ARM64 debug/release portability and x86 release smoke pass the expanded
  suite, including all physical-allocation failure boundaries, authority checks,
  object lifetime, preemption and exact heap/frame reclamation.
- The release userspace admission profile passes recursive funded pools, load
  rollback, stale grants and sibling survival. The nested desktop regression
  covers three levels, pointer stress, protected close, confined persistence,
  repeated subtree reclamation and cold reopen on disposable images.
- Workspace host tests, default x86 UEFI clippy, x86/ARM64 smoke clippy, layout,
  formatting and whitespace checks pass. Interactive saved media is untouched.
- Logs: `build/refusal-arm64-debug.log`, `build/refusal-arm64-release.log`,
  `build/refusal-x86-smoke.log`, `build/refusal-admission.log`,
  `build/refusal-desktop.log`, `build/refusal-host.log`,
  `build/refusal-x86-clippy.log`, `build/refusal-x86-smoke-clippy.log`,
  `build/refusal-arm64-clippy.log`.

This bounds private-draft frame returns per visit, not elapsed time. Heap drops
and bookkeeping remain atomic; pre-draft bare-root failure and synchronous
initial-boot construction still perform their small/whole rollback internally.
Runtime page-object mapping, unmapping/free, allocation rollback, pending
reference/escrow settlement and event bursts remain separate work. More turns
can lengthen the wait for a failed launch under contention; there is no faster
launch or hard-real-time claim. The intermittent clean-close LOSS timeout remains
open. No owner question was needed for this mechanism change.

## Bounded page-object backing retirement (2026-10-20)

Baseline: `2e10113a`. Runtime page-object release, allocation refusal/cancellation,
dead-task object cleanup, deferred executable references and discarded IPC
attachments now return at most eight physical frames per path visit. The
[memory entrance](source-rs/kernel/core/user_tasks/memory/memory.rs) selects each
owner; [retirement](source-rs/kernel/core/user_tasks/memory/retirement.rs) owns the
shared physical refund operation. This extends the existing boot-loop suspension
mechanism without a new syscall or scheduler policy.

Final release removes the alias first and keeps the caller blocked until its
backing refund completes. Death inherits a partly completed release. Allocation
failure refunds the unbacked reservation exactly once, then retires initialized
pages incrementally before returning the error. A retiring object retains its
metadata slot and whole-object logical quota until completion; physical funding
returns to its original sponsor batch by batch. Other users can continue memory
operations between visits, and live readers still pin shared backing.

[Executable references](source-rs/kernel/core/user_tasks/memory/references.rs)
rotate across pending objects, consuming surrendered reference counts once.
[IPC escrow](source-rs/kernel/core/user_tasks/memory/escrow.rs) drains one finite
discard snapshot before taking the next. Review found a slot-reuse hazard in
the incremental implementation: receiver teardown could release an unpublished
reader before its pending discard bit was consumed. Teardown now preserves
slots owned by either the current snapshot or the next IPC discard batch. A
consumed slot may safely be reused while its old object's backing finishes
retiring. The regression deliberately kills both receivers with both batches
pending, reuses the first slot, and checks that the new published right survives.

The [real-frame suite](source-rs/kernel/core/user_tasks/memory/retirement_tests/retirement_tests.rs)
also checks maximum-size release, blocked caller/ready peer state, competing
allocation, death after the first release batch and exact conservation after
each cleanup visit. Existing allocation sweeps refuse every frame allowance and
cancel at each partial batch, now checking the refund allowance in their test
wrappers. The 140-reference/70-object burst takes 130 bounded settlement visits
while retaining a live owner's sealed mapping. Host policy tests check retiring
slot/quota retention, stale authority and eventual safe reuse.

Verification:

- ARM64 debug/release portability and x86 release smoke pass, including the
  expanded real-frame suite and exact heap/frame reclamation.
- Release admission passes recursive funded pools, load rollback, stale grants
  and sibling survival. Nested desktop passes three levels, pointer stress,
  protected close, confined persistence, repeated reclamation and cold reopen.
- Workspace host tests, default x86 UEFI clippy, x86/ARM64 smoke clippy, layout,
  formatting and whitespace checks pass. Interactive F5 saved media was untouched;
  automated desktop/admission runs used disposable images.
- Logs: `build/object-host.log`, `build/object-arm64-debug.log`,
  `build/object-arm64-release.log`, `build/object-x86-smoke.log`,
  `build/object-admission.log`, `build/object-desktop.log`,
  `build/object-x86-clippy.log`, `build/object-x86-smoke-clippy.log`,
  `build/object-arm64-clippy.log`.

The allowance is per cleanup path visit, not a total of eight frames across a
whole boot turn. Mapping removal still happens atomically before any backing
refund. Final map/seal/unmap, heap operations, metadata scans, collection,
authority withdrawal and event bursts remain uninterrupted. More cleanup visits
can increase completion latency under contention; these are structural work
bounds, not a desktop speedup or real-time guarantee.

Next: use the existing phase observations to choose between remaining mapping
work and bursts of event completions. Preserve mapping visibility and terminal/
deadline ordering when introducing further suspension points. The intermittent
clean-close LOSS timeout remains open; passing this regression does not diagnose
it. Guaranteed-service admission still needs its separate owner-level design.
No new owner decision was needed for this milestone.

## Batched wait completion (2026-10-20)

Baseline: `592aacd7`. [Wait completion](source-rs/kernel/core/user_tasks/execution/waits/waits.rs)
previously retried all affected receive, readiness and child waiters within one
invocation. It now selects at most eight candidates per visit and retains the
rest in the registry. A finite current wave preserves readiness-before-receive-
before-child order; later hints coalesce into the next wave so repeated low-slot
events cannot continually overtake older candidates. No allocation is introduced
on the wake path. Cancellation/completion clears both sets before slot reuse.

Pending candidates participate in the ordinary boot-work detection and interrupt
routing. The boot step continues a batch before yielding, without requiring a
fresh hardware event or runnable user. Syscall owners still validate the current
capability, destination, message/child state and deadline. Data or a terminal
error wins over timeout at the actual retry; no arrival timestamp or successful
result is cached in the registry. This preserves existing completion rules while
allowing completion to occur later under contention. No scheduling priority or
new application ABI is added.

Host tests cover a full 31-slot burst, ordering across kinds, draining without new
events, repeated low-slot hints and clearing both queues before a slot changes
wait kind. The [real-frame burst](source-rs/kernel/core/user_tasks/execution/waits/burst.rs)
parks 25 inactive roots and triggers 24 waiters. They complete over exactly three
visits, including two through the production deferred step with no new interrupt.
Assertions check queued data/terminal errors before expiry, readiness without
dequeue, a too-small receive preserving its message, empty-queue timeouts and
exact frame/heap recovery. This fixture does not execute guests; ordinary smoke
and desktop regressions cover real task suspension and execution.

Verification:

- ARM64 debug/release portability and x86 release smoke pass the expanded suite.
  The new buffered `WaitCompletion` phase records gathering plus one retry batch;
  it includes empty invocations and overlaps enclosing trap/boot observations.
- Nested desktop passes three recursive levels, pointer stress, protected close,
  confined persistence, repeated reclamation and cold reopen on disposable media.
- Workspace host tests and the final core tests pass, as do default x86 UEFI
  clippy, x86/ARM64 smoke clippy, layout, formatting and whitespace checks.
  Interactive F5 saved media was untouched.
- Logs: `build/wait-host.log`, `build/wait-core.log`,
  `build/wait-arm64-debug.log`, `build/wait-arm64-release.log`,
  `build/wait-x86-smoke.log`, `build/wait-desktop.log`,
  `build/wait-x86-clippy.log`, `build/wait-x86-smoke-clippy.log`,
  `build/wait-arm64-clippy.log`.

Remaining: eight candidates is a per-invocation allowance, not a whole-trap or
whole-boot-step limit. Subtree withdrawal can call wake repeatedly. Due waiter
timers still drain the at-most-31-entry heap atomically; scheduler sleepers,
physical keyboard and direct device wakes are separate. Grant lookup, mapping
edits and collection also retain atomic work. The timing audit names these gaps;
next work should measure aggregate closure/completion and separate ready-time
from dispatch-time observations before claiming latency improvements. The known
intermittent clean-close LOSS timeout remains undiagnosed. No owner decision was
needed for this mechanism change.

## Coalesced subtree withdrawal (2026-10-20)

Baseline: `6552c20a`. Cancellation previously called the ordinary task-close
entry for each descendant. Each close retried waiters, multiplying completion
batches inside the uninterrupted authority sweep. A descendant queued for later
cancellation could also receive/copy data before the sweep reached it.

[Lifecycle](source-rs/kernel/core/user_tasks/lifecycle/lifecycle.rs) now separates
standalone close from [task authority withdrawal](source-rs/kernel/core/user_tasks/lifecycle/withdrawal.rs).
[The cancellation sweep](source-rs/kernel/core/user_tasks/lifecycle/reclamation.rs)
stops and withdraws every affected descendant, including newly discovered
grandchildren, then runs one waiter batch for surviving observers. Slot, outcome,
loan and physical retirement rules are unchanged. A standalone close still has
its own completion batch; collection and the boot step may also retry waiters.
This reduces repeated work within the sweep without inserting a suspension point
while descendants retain canceled authority.

The shared [real-frame regression](source-rs/kernel/core/user_tasks/lifecycle/withdrawal_test.rs)
uses production admission and loan commits to construct a parent with eight
children. After the initial owner dies, it queues descendant messages and an
unrelated receive before the authority sweep. It checks that all nine descendants
are stopped and their rights/waits removed before the first pause, none consumes
or copies its queued data, and exactly one waiter-completion invocation occurs
for the whole sweep. The unrelated receiver completes successfully. Physical
cleanup then returns every frame and heap allocation while preserving that peer.

`Close` phase samples now expose standalone withdrawal plus completion. `Withdraw`
samples exclude empty cancellation visits and include the whole nonempty sweep
plus its single completion batch. Earlier averages included empty visits and
must not be compared as though this were an elapsed-time speedup. Both are
inclusive QEMU observations, with later physical retirement measured separately.

Verification:

- ARM64 debug/release portability and x86 release smoke pass, including the new
  subtree regression, existing cancellation counts, waiter burst and exact refunds.
- Nested desktop passes three levels, pointer stress, protected close, confined
  persistence, repeated subtree teardown and cold reopen on disposable media.
- Workspace host tests, default x86 UEFI clippy, x86/ARM64 smoke clippy, layout,
  formatting and whitespace checks pass. Interactive F5 media was untouched.
- Logs: `build/withdraw-host.log`, `build/withdraw-core.log`,
  `build/withdraw-arm64-debug.log`, `build/withdraw-arm64-release.log`,
  `build/withdraw-x86-smoke.log`, `build/withdraw-desktop.log`,
  `build/withdraw-x86-clippy.log`, `build/withdraw-x86-smoke-clippy.log`,
  `build/withdraw-arm64-clippy.log`.

Next: distinguish becoming runnable from actual CPU selection in the periodic
observations. The authority sweep, grant-table scans, deadline extraction and
mapping edits still run atomically; a whole trap/boot turn is not limited to one
waiter batch. The known intermittent clean-close LOSS timeout remains open.
No owner semantics or application ABI changed, and no real-time guarantee follows.

## Wait-to-selection observations (2026-10-20)

Baseline: `74ceb818`. Registered IPC receives, readiness waits and child waits
now record two separate observations in `memory-lab` builds: deadline lateness
at completion in ticks, and elapsed microseconds from completion/unblock to the
first selected return. The latter includes peer execution, boot work, CPU-ceiling
gates and host descheduling. It ends before assembly restores user context; it
does not timestamp the first user instruction or the original hardware event.

The [timing entrance](source-rs/kernel/core/user_tasks/execution/timing/timing.rs)
now lives in a named folder with separate phase definitions, wait observations
and host tests. Rust module plumbing contains no implementation. Storage is
reserved fallibly at session preparation; hooks allocate and print nothing.
Ordinary builds omit the extra storage, clocks and hooks. The ARM64 heap-pressure
fixture exercises 75 sampled startup refusals with the larger lab-only storage,
with exact reclamation; debug boot retains its existing guarded stack size.

`WAIT_DISPATCH` reports per-slot/per-kind completed, selected, canceled and still
pending populations. Cancellation clears unfinished timestamps before slot
reuse. Later returns cannot count a completion twice. Only selected observations
contribute to selection durations; only reached deadlines contribute to lateness.
Data/terminal-before-timeout rules are unchanged, so a reached deadline does not
necessarily mean the wait returned a timeout. No-deadline waits, immediate
syscall results and other wake mechanisms are not silently mixed into these
populations. The [contract and reading guide](source-rs/kernel/core/user_tasks/execution/timing/README.md)
records the full scope and units.

[The report tool](tools/boot-harness-rs/timing_report.py) displays these spans
separately from inclusive execution phases. `--verify-waits` checks periodic
observations against the actual number of registered blocked waits. Normal x86
smoke performs that check for all five periodic cases, requiring selected counts
to agree and no pending/canceled samples. It imposes no latency threshold; zero
blocked waits is valid when all deadlines are already expired at the call.
The capped workload can therefore have fewer blocked-wait samples than its 16
jobs. This is observable now rather than being mistaken for missing samples.

Verification:

- Workspace host tests pass, including a ready task held behind an actual
  scheduler CPU ceiling, first-selection accounting, cancellation/reuse, zero
  timestamps, deadline equality and wrapping ticks.
- ARM64 debug/release portability and x86 release smoke pass. All five x86
  periodic wait counts agree with selection reports. Report-tool tests reject
  missing/duplicate/inconsistent observations and preserve separate populations.
- Default x86 UEFI clippy, x86/ARM64 smoke clippy, source layout, formatting and
  whitespace checks pass. No ordinary desktop policy or saved F5 media changed;
  the nested desktop was not rerun for these lab-only hooks.
- Logs: `build/dispatch-host.log`, `build/dispatch-core.log`,
  `build/dispatch-arm64-debug.log`, `build/dispatch-arm64-release.log`,
  `build/dispatch-x86-smoke.log`, `build/dispatch-report.log`,
  `build/dispatch-x86-clippy.log`, `build/dispatch-x86-smoke-clippy.log`,
  `build/dispatch-arm64-clippy.log`. Python check:
  `python tools/boot-harness-rs/timing_report_test.py`.

Next: use these observations to select the next remaining atomic mapping,
authority or deadline operation. They do not justify changing CPU entitlement
or assigning privileged processes implicit priority. Guaranteed service remains
separate from ceilings. The intermittent clean-close LOSS timeout is still open;
these probes have not reproduced or diagnosed it. No owner decision was needed.

## Batched page-table permission/removal edits (2026-10-20)

Baseline: `d8450d09`. The page-object timing workloads exposed repeated hierarchy
walks inside atomic freeze/unmap operations. The
[core region coordinator](source-rs/kernel/core/user_tasks/address_space/regions.rs)
now sends the contiguous range to the selected ISA before updating its backing
ledger. [x86](source-rs/kernel/arch/isa/x86_64/user/memory/regions.rs) and
[ARM64](source-rs/kernel/arch/isa/aarch64/user/regions.rs) walk once per leaf table,
then edit only the requested entries. A current 128-page object needs one or two
walks instead of 128. The backends own table geometry/encoding; core retains
frame lifetime, capability publication and its single ledger pass.

There is no suspended cursor, allocation or new authority state. Both operations
remain atomic with an inactive root, pinned backing and masked interrupts. Leaf
writes still scale with object size; ledger work still scales with mapped pages.
The existing next-root synchronization remains required. This is less work in
an uninterrupted operation, not a real-time bound or an SMP design. Mapping new
leaves and its failure rollback remain unchanged.

The new x86 host test constructs tables through the x86 crate and translates the
result independently. It checks physical addresses, RO/NX permissions, complete
removal and untouched neighbors across PT, PD and PDPT boundaries. The shared
real-frame allocation fixture now seals temporary aliases at every guarded slot
alignment on both ISAs, then checks each removed page and surviving owner/stack
mapping. All backing is reclaimed. An initial fixture assertion incorrectly used
the single-page copy validator for an entire object; that assertion was corrected
to validate each page before the successful boot runs.

Verification:

- Workspace host tests and arch host clippy pass, including the new table test.
- x86 release smoke and ARM64 debug/release portability pass, including existing
  reader write-fault containment, allocation/refusal sweeps and exact reclamation.
- The report verifier accepts all five periodic wait populations. This run's
  128-page freeze/unmap means were 14.11/24.72 us; unchanged map installation was
  46.36 us. These are elapsed observations with host/probe effects, not an isolated
  before/after benchmark or a desktop speedup claim.
- Nested desktop passes three levels, pointer stress, protected close, confined
  persistence, repeated subtree teardown and cold reopen on disposable media.
  Interactive F5 media was untouched.
- Default x86 UEFI clippy, x86/ARM64 smoke clippy, layout, formatting and
  whitespace checks pass.
- Logs: `build/region-edit-host.log`, `build/region-edit-host-clippy.log`,
  `build/region-edit-arm64-debug.log`, `build/region-edit-arm64-release.log`,
  `build/region-edit-x86-smoke.log`, `build/region-edit-report.log`,
  `build/region-edit-clippy.log`, `build/region-edit-smoke-clippy.log`,
  `build/region-edit-arm64-clippy.log`, `build/region-edit-desktop.log`.

Next: runtime mapping installation still repeats hierarchy walks per page and
must preserve complete rollback on refusal. Investigate a reserved-table range
transaction before adding suspension state. The authority sweep, deadline
extraction and other audited atomic work remain; guaranteed CPU service is still
separate from ceilings. The intermittent clean-close LOSS timeout remains open.
No owner semantics or application ABI changed.

## Atomic reserved-range installation (2026-10-20)

Baseline: `2771bad8`. Runtime page-object installation now enters `map_pages` in
the [x86](source-rs/kernel/arch/isa/x86_64/user/memory/regions.rs) or
[ARM64](source-rs/kernel/arch/isa/aarch64/user/regions.rs) backend. The first pass
validates all backing addresses, table branches and vacant destination leaves;
the second installs ordinary RW/NX or RO/NX entries. Both passes visit each
leaf table once, giving a maximum-size object two or four hierarchy walks rather
than one per page. No allocation or external work separates the passes.

Refusal changes neither parent nor leaf tables. Core reserves its ledger space
first, then publishes entries only after the complete mapping succeeds. No
partial runtime installation needs rollback. x86 admission now reserves parent
tables with user/write permission using its unpublished scratch alias; runtime
installation refuses restrictive parents instead of widening them. The scratch
leaf is removed before any task runs. Obsolete `NoFrames` and ARM64 single-page
rollback helpers were removed; x86 keeps a private scratch-unmap helper.

This remains an atomic inactive-root operation under the current single-CPU,
IRQs-masked contract. Leaf validation/writes, backing validation and ledger work
still scale with bounded populations. Frame custody, allocation cancellation,
caller charging, final refunds and capability publication retain their existing
ordering. No owner decision or userspace ABI change was needed.

Verification:

- Workspace host tests pass. The expanded x86 tests independently translate RW/NX
  and RO/NX mappings across PT, PD and PDPT boundaries. Complete table snapshots
  remain unchanged for all 128 possible collisions, a missing later table,
  read-only parents, a huge-page conflict and invalid frames. Retrying after
  refusal succeeds; neighbors and table counts remain unchanged.
- x86 release smoke and ARM64 debug/release pass. The shared
  [real-frame proof](source-rs/kernel/core/user_tasks/address_space/regions_test.rs)
  injects all 128 possible conflicts across two tables, checks that core adds no
  ledger entries on refusal, removes only the conflict and successfully retries.
  Invalid final backing and all-slot mapping/sealing/removal also pass, with the
  existing exact frame/funding/heap reclamation checks.
- All five periodic wait/selection population checks pass. This run's 128-page
  mapping mean was 19.56 us (max 152 us). Host scheduling and workload mix prevent
  treating this as an isolated speedup or a real-time bound.
- Nested desktop passes three levels, pointer stress, protected close, confined
  persistence, repeated subtree teardown and cold reopen on disposable media.
  Interactive F5 media was untouched.
- Arch host clippy, default x86 UEFI clippy, x86/ARM64 smoke clippy, source layout,
  formatting and whitespace checks pass.
- Logs: `build/region-map-host.log`, `build/region-map-arch.log`,
  `build/region-map-host-clippy.log`, `build/region-map-arm64-debug.log`,
  `build/region-map-arm64-release.log`, `build/region-map-x86-smoke.log`,
  `build/region-map-report.log`, `build/region-map-all-phases.log`,
  `build/region-map-clippy.log`, `build/region-map-smoke-clippy.log`,
  `build/region-map-arm64-clippy.log`, `build/region-map-desktop.log`.

Next: inspect image-loading work behind the remaining larger admission samples
before choosing another split. The all-session report observed 895 `ImageBatch`
visits with a 179.62-us mean and 20 samples at least 1 ms; these include mixed
fixtures and host effects and do not establish a cause. Authority withdrawal,
deadline extraction and the other audited atomic paths also remain. Keep the
timing audit honest about complete trap/boot turns versus individual phases.
The intermittent clean-close LOSS timeout remains open; guaranteed CPU service
is still separate from ceilings.

## Bounded image-copy work and page initialization (2026-10-20)

Baseline: `c4351353`. Investigation of the larger `ImageBatch` samples found
redundant writes: the loader zeroed each entire page, then overwrote its file
prefix. [Page population](source-rs/kernel/core/user_tasks/address_space/image.rs)
now copies that prefix directly and zeroes only the tail. A wholly zero BSS or
stack page never reads source contents. Every byte is initialized before cache
maintenance or mapping; ownership, rollback and publication order are unchanged.

New lab-only per-page observations separate backing allocation, initialization
of file-backed pages, initialization of all-zero pages, and cache/translation
publication. They narrowed the larger x86 samples to file-backed initialization;
allocation, zero-only pages and publication were much smaller in the observed
population. Disassembly of the measured build showed word-sized `rep movsq` and
`rep stosq` in its standard copy/fill routines. That does not diagnose the remaining
emulator/host contribution or justify replacing those routines with custom ISA code.

[Runtime construction](source-rs/kernel/core/user_tasks/admission/construction.rs)
now gives [the loader](source-rs/kernel/core/user_tasks/address_space/creation.rs)
two independent allowances: eight initialized/mapped pages and 8 KiB of file bytes
per turn. A page whose prefix would exceed the remaining byte allowance stays at
the cursor until the next turn. Partial file pages spend their actual prefix size;
BSS/stack pages spend only the page allowance. The allowance must accommodate at
least one full page, so a valid next page cannot strand the cursor. Initial boot
admission remains synchronous through the same engine. More turns permit peer
execution but can lengthen total launch time under contention.

`ImageBacking`, `ImageFile`, `ImageZero` and `ImagePublish` are per-page attempts
inside the per-turn `ImageBatch`. File includes a partial page's zeroed tail;
Publish includes cache maintenance and table allocation/leaf installation, not
making a task runnable. Failed allocation is sampled before returning; failed
mapping is also sampled. Counts can therefore differ. Ordinary builds omit these
hooks and clocks, and synchronous initial construction supplies no observer.
The report tool labels the distinct populations. See the
[timing audit](source-rs/kernel/core/scheduler/timing.md) for scope and units.

Verification:

- Workspace host tests pass. All 4,097 possible prefix lengths are tested against
  poisoned destination pages and adjacent sentinels for byte-backed and frame-backed
  contents. Partial final source pages and empty-source BSS pass too. The cursor
  test crosses partial pages, BSS, gaps and stack with both allowances, preserving
  exact address order and progress on every turn.
- x86 release smoke and ARM64 debug/release pass. Every observed admission turn checks
  its copied-byte allowance. The complete real-frame synthetic construction checks
  exactly 72 backing/publication attempts, one file page and 71 zero pages; initial
  synchronous roots are excluded. Existing physical refusal, withdrawal, retained
  source, fault-containment and exact reclamation sweeps pass.
- All five periodic wait/selection checks pass. The instrumented run before the
  copy allowance recorded 903 image turns, mean 165.96 us and 24 samples >=1 ms;
  afterward it recorded 2,443 turns, mean 67.36 us and one sample >=1 ms. These are
  different subdivisions and mixed session populations, not a total-launch speedup
  or a time bound. A single file-page sample still exceeded 1 ms afterward.
- Core host clippy, default x86 UEFI and x86/ARM64 smoke clippy, Python report tests,
  layout, formatting and whitespace checks pass.
- Nested desktop passes three levels, pointer stress, protected close, confined
  persistence, repeated subtree teardown and cold reopen on disposable media after
  the copy allowance change. Interactive F5 media was untouched.
- Logs: `build/image-load-host.log`, `build/image-load-host-clippy.log`, `build/image-load-clippy.log`,
  `build/image-load-smoke-clippy.log`, `build/image-load-arm64-clippy.log`,
  `build/image-load-x86-smoke.log`, `build/image-load-arm64-release.log`,
  `build/image-load-arm64-debug.log`, `build/image-load-desktop.log`,
  `build/image-load-report.log`. The earlier instrumented run is preserved as
  `build/image-load-before-budget-report.log` and
  `build/image-load-before-budget-serial.log`; the initial copy-routine inspection
  is in `build/image-load-symbols.log` and `build/image-load-disassembly.log`.

Next: the largest copy unit is now one file page. Do not shrink it or introduce
SIMD merely from QEMU maxima; use a controlled source/copy probe if investigating
that remaining cost. Next prioritize capacity-focused authority-withdrawal
evidence: its wider grant scans still run atomically, while the existing subtree
fixture covers nine descendants. Deadline work also remains in the timing audit.
Guaranteed CPU service is still a separate design from ceilings. The intermittent clean-close LOSS
timeout remains open; no owner semantics or userspace ABI changed here.

## Scoped grant withdrawal at task capacity (2026-10-20)

Baseline: `691bc714`. IPC task closure previously scanned the entire grant arena
even though dynamic records already occupy recipient-local reserved ranges.
[Grants](source-rs/kernel/core/ipc/grants.rs) now provides the common range used
by lookup, vacancy search, handle-budget accounting and
[withdrawal](source-rs/kernel/core/ipc/lifecycle.rs). Installation asserts that
dynamic grants use their owner's vacant range and refer to dynamic endpoints.
This includes invisible incoming moves: transfer vacates the source and installs
a new recipient-local record. No ownership index or additional state was added.

At the current 31-task capacity, close scans 192 packed boot records plus the
task's 48 dynamic slots (240 candidates), instead of all 1,680 records. Escrow
discard still walks its actual affected grant chain. Sponsorship and channel
accounting remain independent of custody, with charges held until final closure.
Boot endpoint replacement clears its three fixed records, including an old
revoker. Control-pair acceptance touches six records and finds its peer-local
ordinal in the boot prefix. Local handle-budget queries scan only 48 dynamic
records. Names, visibility, permissions and the application ABI are unchanged.

[Capacity policy tests](source-rs/kernel/core/ipc/tests/withdrawal.rs) fill all
64 boot endpoints and 192 dynamic channels. One case closes the last task slot
with all 48 visible dynamic grants, checks exact discarded-memory notifications,
preserves every unrelated dynamic grant/queue, then restarts with fresh names.
Another fills all 48 recipient slots with a chain of hidden receive rights, with
memory escrow at its tail. Closing the recipient drains the chain, preserves
the surviving sender's sponsorship charge until final close and leaves peers'
queued messages intact. Both cases finish with no dynamic channels left.

The [real-frame subtree fixture](source-rs/kernel/core/user_tasks/lifecycle/withdrawal_test.rs)
now fills all 31 task slots: initial owner, unrelated survivor and 29 descendants
in two delegated branches. Each spawn stays within the existing 14-loan wire
capacity. It queues grandchild data after owner death, then proves every descendant
is stopped and stripped of authority before one survivor completion batch runs.
No canceled child consumes its data; physical cleanup retains slots/outcomes until
collection and returns every frame/funding unit. The cleanup stall guard scales
with task capacity; the full fixture needed 2,130 retirement visits in this run.

Verification:

- Workspace host tests and core host clippy pass, including both full-arena
  scenarios and existing transfer, initial-bundle, restart and sponsorship tests.
- x86 release smoke and ARM64 debug/release pass, including all 31 real roots,
  both delegated branches, survivor delivery and exact reclamation. The x86
  fixture observed its one authority sweep at 125 us; this is a QEMU observation,
  not a bound or an elapsed comparison against the previous nine-descendant case.
- All five periodic wait/selection checks pass. Default x86 UEFI and x86/ARM64
  smoke clippy, layout, formatting and whitespace checks pass.
- Nested desktop passes three levels, pointer stress, protected close, confined
  persistence, repeated subtree teardown and cold reopen on disposable media.
  Interactive F5 media was untouched.
- Logs: `build/authority-host.log`, `build/authority-core.log`,
  `build/authority-capacity.log`, `build/authority-host-clippy.log`,
  `build/authority-arm64-release.log`, `build/authority-arm64-debug.log`,
  `build/authority-x86-smoke.log`, `build/authority-report.log`,
  `build/authority-clippy.log`, `build/authority-smoke-clippy.log`,
  `build/authority-arm64-clippy.log`, `build/authority-desktop.log`.

Next: full-capacity cleanup exposes repeated conservation-accounting work:
2,131 observations totaled 19,912 us in this fixture, versus 2,678 us in retirement
bodies. Inspect repeated ledger scans before adding more scheduling machinery,
retaining both physical and funding conservation checks. Authority's remaining
link/launch scans, channel sponsorship scans, escrow chains and simultaneous
deadline extraction still run atomically. No real-time entitlement or owner
semantic was added. The intermittent clean-close LOSS timeout remains open.

## Reuse the cleanup backing count (2026-10-20)

Baseline: `2a1f4b89`. Each cleanup step's
[conservation check](source-rs/kernel/core/user_tasks/lifecycle/reclamation.rs)
counted page-object backing twice against an immutable session. It now counts
once and reuses the result in the physical-allocation and funding equations.
Both assertions still run after every step, with the same task/draft counts,
allocator baseline and unfinished allocation reservations. No maintained counter,
sampling interval or allocation/retirement behavior was introduced.

Verification: workspace host tests, x86 release smoke, ARM64 release portability,
x86 smoke clippy, formatting and whitespace checks pass. The real-frame suites
exercise partial allocation/rollback, retained backing and escrow, and the full
31-task subtree with exact refunds. All five periodic wait/selection checks pass.
Logs: `build/accounting-host.log`, `build/accounting-x86-smoke.log`,
`build/accounting-arm64-release.log`, `build/accounting-clippy.log`, and
`build/accounting-report.log`. No desktop rerun was needed for this local count
reuse; the prior milestone's desktop evidence is unchanged. F5 media was untouched.

The full-capacity x86 fixture still performs 2,131 conservation checks. This run
recorded 13,768 us total versus the previous 19,912 us; retirement itself recorded
4,861 us versus 2,678 us. Host/QEMU timing variation prevents attributing a precise
elapsed speedup. The structural result is one backing scan instead of two.

Next: conservation remains an atomic full recount. Avoid adding cached totals
without an independent audit that catches drift across allocation refusal,
partial cancellation and retained escrow. The timing inventory still identifies
simultaneous deadline extraction and aggregate authority work as remaining
boundaries. No owner choice or ABI changed; the intermittent clean-close LOSS
timeout remains open.

## Four-level desktops and retained-store heap pressure (2026-10-20)

Baseline: `29d5f02e`. The owner reported lag around four nested matrices in
normal F5 with existing users. The desktop and latency runners now accept
`--nested-depth 4`, exercising four nested desktops **beneath** the outer one.
The deepest Notes focus coordinate scales with depth; the first exploratory
four-level failure was a harness click outside that app, corrected before the
passing runs. No kernel scheduling, authority, ABI or page budget changed.

The saved F5 disk was copied, never installed into, repaired or reset in place.
On that copy, automatic collection before a nested subtree copy panicked with
`memory allocation of 8208 bytes failed`. Heap tracing placed the failure before
the copy writer: engine state, its 64 KiB cache and the transient live-record
census exhausted the provider's 56-page heap. This was a real provider crash,
not just slow framebuffer writes. The collector now stores fixed-size identities
inside separate event/manifest/chunk sets rather than separately allocated full
keys. The cache is 32 KiB, leaving more room in the same heap. Pins, authentication,
deletion order, exact quota and immediate durability are unchanged.

Compact identities alone still exhausted the heap. With both changes, the
initial saved-disk replay created and entered the third nested user without a
provider restart; observed requested-allocation peak fell from 215,004 to 172,158
bytes. These allocator samples exclude allocator overhead, stack and reader
pages. They do not establish a universal bound. The full live census still
scales with root metadata; fallible/bounded collection workspace remains open.

**Retained-program caveat:** the same disk contains older nested program
manifests than its current outer template. Two extended replays exited a matrix
with `TIMED_OUT` while a copy was underway, after getting past the original OOM.
One ran alongside the retention test; the second ran alone and also failed.
Do not call the owner's existing tree fixed. Older interaction timeout behavior
is a plausible cause, not yet attributed to an exact executable/call site.
No recursive legacy upgrade was added; the owner does not require one. Fresh
Lab recreates users with current executables, while retaining a backup of old data.

[nested_existing_test.py](tools/boot-harness-rs/nested_existing_test.py) preserves
this diagnostic path: source availability check, byte copy, current private boot
image, no object-store installer, nested login/create, deepest edit/save,
dirty-close refusal and clean teardown across cold boot. It expects one outer
User 1 and compact stock chrome; it deliberately fails on old-program timeouts.
The host hierarchy inspector now prints names beside IDs/parents/manifests.
Use a copy because engine open/close can recover or maintain records.

`trace-storage` adds recursive LOSS/CLOSE send, answer, deferred and stale-reply
events. Incarnations and slots are matrix-local, not globally unique task IDs.
The intermittent clean-close LOSS timeout was not reproduced or declared fixed;
its existing refusal policy and deadlines remain unchanged.

Evidence before the storage fix:

- Four-level current-program pointer stress and cold reboot passed, including
  dirty-close protection, deepest persistence and repeated whole-subtree teardown:
  `build/nested-four-focused.log`.
- Four-level latency/cold reopen passed: `build/nested-four-latency.log`. Fresh
  deepest typing observations were 120ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Å“133 ms and save 412 ms; user entry was
  5.2 s. Host injection/capture and tracing perturb these upper bounds. Earlier
  isolated key tracing saw 1ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Å“4 guest ticks from source to Notes. This does not
  isolate scheduler delay or claim a host-independent performance threshold.
- Five-second delayed interaction fixtures at four levels passed with pointer
  stress and cold reopen: `build/nested-four-delayed.log`.
- Saved-disk OOM, compact-only OOM and initial successful third-user replay:
  `build/nested-existing-inspect/replay-metrics.log`, `replay-fixed.log`,
  `replay-cache.log` in that directory. Extended retained-program failures:
  `build/nested-existing-final.log`, `build/nested-existing-solo.log`.

Final storage verification: workspace host tests, all 38 storage-lab host tests,
default and trace/delay x86 storage/matrix clippy, 27 Python harness tests,
layout, formatting and whitespace checks pass. The three-boot objects retention
suite passes exact host record comparisons, pins, collection and cold recovery:
`build/nested-retention.log` (artifact `objects/20261021T041418057534Z`). Host logs:
`build/nested-lag-host.log`, `build/nested-storage-host.log`,
`build/nested-default-clippy.log`, `build/nested-final-clippy.log`,
`build/nested-final-python.log`. No ARM64 or kernel smoke rerun: this changes
userspace collection/cache and diagnostics, not kernel or architecture code.

The final storage build also passes four nested desktops under pointer stress
on both fresh and cold boots, including exact deepest saved text, dirty-close
protection, isolation and two complete close/reopen cycles per boot:
`build/nested-four-final.log`, artifact `desktop/20261021T041745289469Z`.
The storage-lab host clippy check passes after the inspector change.
The saved-disk replay runner also passes both boots at depth four on a copy of
that current-program fixture (`build/nested-replay-current.log`, artifact
`nested-existing/20261021T042304676407Z`), including edits, save confirmation,
protected close and clean collection. Its source bytes remain identical. This
validates the replay path without reclassifying the older F5 disk's failure.

Next: bound collection workspace independently of record count and measure
launch/seed phases under retained-store pressure. Preserve the failing saved
copy for exact timeout attribution if needed. Current-program desktop evidence
must stay separate from legacy snapshots and from the still-open LOSS timeout.

## Fresh lab as the F5 default (2026-10-20)

The owner prefers a fresh OS on every F5 launch over retaining old nested
program snapshots. **Cathedral: Build & Run in QEMU** now passes `--fresh`.
The existing reset backs up the disk and receipt under `before-fresh-*`, then
seeds current programs; users must be created again. **Cathedral: Resume Saved
Lab in QEMU** explicitly reuses the disk for persistence work. The bare CLI
without `--fresh` still preserves state. No reset mechanism or migration changed.

Verification: launch JSON and both command arguments checked; all ten existing
database-image tests pass, including reset backup/rollback and busy-disk refusal.
Whitespace checks pass. No guest rerun was needed for selecting the already
tested fresh launch path. The current disk was not reset during this change.

## Bounded collection, one refresh per nested binding and a soak (2026-10-21)

Baseline: `7697b70e`. The owner asked for the previous handoff's three targets in
order: bound collection's memory, stress one long session, and reduce nested
entry delays. The measurements below come from the four-level latency workload
with storage traces on disposable media; F5 media was untouched.

**Collection workspace.** [collection.rs](source-rs/platform/libraries/database-objects/collection.rs)
no longer gathers every head into a vector or every live identity into one set.
A pass marks one record kind at a time inside one window of identities (events
by the low byte of their object number, manifests and chunks by their first
digest byte), reading heads in batches of 32. A window whose marks would exceed
`WORKSPACE_BYTES` (24 KiB at a conservative 64 bytes per mark) is split in two
and marked again, down to 256 windows per kind; a window that still overflows
returns the new `Error::Workspace` with nothing removed. The provider skips that
maintenance turn, records `collect-workspace-exhausted` in every build and lets
exact quota admit or refuse the write; explicit COLLECT answers NO_MEMORY. Chunk
indexes are still validated before any removal, the event pass authenticates
every head's event first, pins are validated for their one revision, and
retirement order is unchanged. `Collection` now reports the windows a pass
computed and its largest marked set, sampled as `collect-pass` under tracing.
Two host tests collect a crowded root with a workspace too small for one window
per kind and reach exactly the unbounded result, and refuse whole with a
workspace too small for one mark.

**Provider heap and cache.** [main.rs](source-rs/platform/services/storage/main.rs)
sizes the heap from the funded private pages: 64 pages stay outside for one
maximal reply plus eight of headroom, and the rest is heap, clamped between the
56 pages the engine has always run in and the 128 pages one private object may
hold. [database.rs](source-rs/platform/services/storage/database.rs) reserves
256 KiB of that heap for engine state, the writer and the collection workspace
and gives the remainder to the engine cache, never below the previous 32 KiB.
The desktop profile now funds the provider with 200 private pages, so its heap
is 512 KiB and its cache 256 KiB (64 pages) instead of 8 pages; every lab
profile keeps its previous 56-page heap and 32 KiB cache. The first attempt
funded 384 pages and asked for a 312-page heap, which the kernel refused as one
object: the provider panicked at startup and the desktop showed its sign-in
recovery screen as designed. Provider heap peaks in the traced four-level runs
were 373 KB (fresh) and 376 KB (cold) of 524,288, with recovery itself peaking
near 370 KB; the heap report line now prints `capacity=`.

**One refresh per nested binding.** The trace showed why entry cost grew faster
than depth: before forwarding any child request, a nested matrix sent
STORAGE_INFO upstream, and the parent answered that only after refreshing in
turn, so one request from the deepest of the nine matrix hops at depth four
cost about eighty transits before reaching the provider. The depth-four save
spent nine ticks on the request path and none on the reply path.
[pending.rs](source-rs/distribution/matrix/service/parent/storage/pending.rs)
now refreshes before the first exchange only, while `storage_epoch` is zero,
and once more when the parent refuses a stamped revision: a child's refusal is
answered after that refresh (its attachment already travelled), while the
matrix's own program read repeats its step. Grants, revocations and provider
replacement still clear bindings exactly as before; a stale revision is still
refused by the parent rather than accepted. In the cold-boot trace the
nested-receive count fell from 3,672 to 683 and root-receive from 1,360 to 170
for the same 293 provider requests; the depth-four app round trip fell from
25 ticks to 8 to 13.

**Control-stream failures wait briefly for the outcome.** The headless
`--matrix-test` recovery scenario had depended on that refresh chatter: with
the matrix always mid-exchange, a leaf exiting during a forwarded control RPC
was deferred until the kernel reported its outcome, and the asker saw
PEER_CLOSED. Without the chatter the matrix answered BUSY before reclamation
finished. [control.rs](source-rs/distribution/matrix/service/children/control/control.rs)
now keeps such a request pending, polling each tick, until the child's outcome
arrives or `OUTCOME_GRACE_TICKS` (50) pass inside the asker's deadline, then
answers BUSY as before. A live child that merely dropped its control link still
gets BUSY, half a second later; the `--matrix-storage-test` probes wait 100.

**Soak.** `desktop_test.py --soak CYCLES --nested-depth D` runs
[desktop_soak.py](tools/boot-harness-rs/desktop_soak.py): each cycle enters the
nested desktops, checks the text the previous cycle saved, replaces it with a
cycle token, saves, closes the app and tears the subtree down from the outer
taskbar; a cold boot checks the last token and runs two more cycles. It records
per-phase seconds, provider heap live/peak/capacity, request and collection
counts, and fails on replacements, offline storage, matrix exits, workspace
exhaustion, allocation failure or less than 32 KiB of heap headroom. The
nested-desktop helpers moved into a `Nested` class shared with the existing
workload. The harness's first cold boot failed on its own numbering bug (it
counted the fresh boot's cycles while they were still being written and
expected `c12` where `c23` had been saved); the OS was fine. Sixteen cycles at
depth two then passed on a fresh lab and two more after a cold boot:

| Cycle | Seconds | enter-1 | enter-2 | edit | save | close | teardown | Heap live | Heap peak |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 (creates users) | 9.67 | 3.64 | 4.23 | 1.03 | 0.20 | 0.28 | 0.28 | 326,664 | 369,932 |
| 2 to 16 | 7.12 to 8.08 | 2.11 to 2.61 | 2.67 to 2.89 | 1.38 to 1.84 | 0.19 to 0.23 | 0.28 to 0.31 | 0.28 to 0.31 | 322,536 to 339,032 | up to 373,116 |
| cold 1, 2 | 7.64, 9.61 | 2.16, 2.95 | 2.86, 3.78 | 1.81, 1.98 | 0.22, 0.30 | 0.30 | 0.30 | 326,872, 326,920 | up to 376,876 |

Every cycle issued 105 provider requests (165 with user creation), automatic
collection never triggered (the root stayed below its watermark) and no
replacement, exit or exhaustion marker appeared. Edit time rises with the token
length. An earlier twelve-cycle run drifted from 5.5 to 8.7 seconds per cycle
with the same flat heap and request counts; the sixteen-cycle run did not, so
treat that as host variation until a kernel-side per-cycle sample exists.

**Nested entry, before and after**, host-observed milliseconds on the cold
boot of `desktop_test.py --latency --nested-depth 4 --trace-storage`:

| Depth | Launch picker | Enter user | Launch Notes | Save |
| --- | --- | --- | --- | --- |
| 1 | 1,257 to 969 | 1,579 to 1,287 | 490 to 483 | 193 to 225 |
| 2 | 1,757 to 1,100 | 2,480 to 1,359 | 878 to 450 | 211 to 210 |
| 3 | 3,324 to 1,447 | 3,528 to 1,856 | 1,184 to 645 | 395 to 240 |
| 4 | 5,037 to 1,995 | 5,196 to 2,173 | 1,737 to 746 | 333 to 226 |

At depth four the picker launch's provider CPU fell from 1,681 ms to 765 ms and
its sectors read from 3,336 to 1,576; user entry from 1,130 ms to 785 ms. On the
fresh boot, which also installs each level's template closure, depth-four picker
launch fell from 8,306 ms to 3,337 ms, user entry from 5,205 ms to 1,996 ms and
user creation from 1,792 ms to 486 ms (provider CPU 727 ms to 98 ms). Typing
stayed at about 125 ms per key at every depth; it is bounded by injection and
capture, not the guest. One depth-zero Notes close on the fresh boot took
1,341 ms against 250 to 300 ms everywhere else, with nothing in the guest trace
between the save's completion and the next launch; it is recorded, not attributed.
Before: `build/boot-harness-rs/desktop/20261021T040219186638Z`; after:
`build/boot-harness-rs/desktop/20261022T022034263570Z`.

**Fixture repair.** `provider_live_test.py` still clicked the pre-compact
taskbar's left-hand RETRY SAVES position; its exhausted and readiness-stall
cases now click the right-hand column's row.

**Documentation.** The distribution map now says four nested desktops and names
the current launch entries; this handoff's deadline figures match the contract's
1200-tick base; the provider, store and harness guides describe the funded heap,
windowed collection and soak.

Verification:

- Host: workspace `cargo test --locked` (261 tests), `cathedral-database-objects
  --features host` (20, including the two new window tests), storage-lab engines
  (38), harness Python (12 including the soak bookkeeping), clippy for the store,
  provider and matrix in default and trace builds, `check-userspace`, layout,
  formatting and whitespace checks pass.
- Guest, on disposable media: four-level latency on fresh and cold boots; four
  nested desktops under pointer stress with protected close, confined
  persistence, repeated subtree teardown and cold reopen (three runs);
  `objects_test.py --retention`; `matrix_database_test.py --recovery` (twice);
  the plain default desktop with host verification of 553 records; the full
  provider-replacement suite on that fixture (precommit, postcommit, exhausted,
  readiness-stall, nested and deep); `--matrix-storage-test`, `--matrix-test`
  and `--matrix-desktop-test` with the final grace code; the sixteen-cycle soak.
- Logs: `build/soak-latency-four.log`, `build/suite-*.log`, `build/suite2-*.log`,
  `build/suite3-*.log`; soak artifact
  `build/boot-harness-rs/desktop/20261022T024618213478Z`.

Next, in order:

1. Fresh-boot entry is now dominated by each level's first launch installing
   its template closure (nine subtree copies, two copies and two named
   creations, 1.3 to 1.5 s of provider CPU per level) and by the engine extent
   growing from 1.5 MiB to 3.1 MiB during the first user creation, which zeroes
   3,300 sectors (0.8 s). Seed the extent at its working size and measure
   whether the closure can be shared or installed lazily.
2. Bulk program reads still cost 1.2k to 1.6k sectors per level: each 4 KiB
   chunk pulls two to three tree pages through the cache. A larger cache needs
   either a multi-object heap or a page cache outside the allocator; measure the
   hit rate before choosing.
3. Each Nested level is two matrix hops (the user matrix and the nested root).
   The refresh fix made a hop cost one transit each way; the remaining
   per-hop cost is event-loop turns, visible in the cold trace as one to two
   ticks per request at depth four.
4. The intermittent clean-close LOSS timeout did not reproduce in these runs.
   Less storage traffic means fewer turns in which a matrix is `storing()` and
   answers LOSS with BUSY; keep the trace on and attribute the next occurrence.
5. The desktop never reached the collection watermark here, so windowed
   collection ran only under host tests and the guest retention proof. A soak
   that fills a root (larger notes, more users) would exercise it live.
6. The soak samples the provider's heap only. Kernel task-slot, frame and grant
   accounting per cycle would turn the host-side cycle time into evidence.

## The engine owns its region from format (2026-10-21)

Baseline: `6c27d97d`. The owner asked why the storage engine's extent grew at
runtime at all, since the provider is handed one fixed region and Cathedral's
attenuation pattern is exactly that: you get this much, manage inside it. It
grew because redb thinks it writes an ordinary file, and the adapter turned
each file extension into a zeroing pass over the newly claimed range, 3,300
sectors and 0.8 s on the first user creation.

The region is now the engine's from format and nothing is zeroed at runtime.
redb still keeps a layout length and, at open, requires the backend to report
exactly the length it stored, so the envelope keeps mirroring that length: a
change flushes earlier engine writes, then publishes and flushes both envelope
copies, and writes nothing else. Zeroing was never a redb requirement. The
engine never reads space it has not written, and after an unclean stop it
rebuilds its allocators from its trees rather than trusting region headers, so
the bytes a regrown range hands back may be whatever the engine last wrote
there. Pinning the reported length to the capacity was considered and rejected:
redb shrinks its layout at commit whenever the last region's trailing free
space exceeds half, with a crate-private policy, and a stored layout that
differs from the reported length sends every open through its recovery path.
Mirroring the layout keeps every open clean.

Changed: [the guest adapter](source-rs/platform/libraries/storage-volume/engine/backend.rs)
and [the host adapter](tools/storage-lab/engines/volume/backend.rs) publish the
envelope only; the extent observer, `open_observed`, the provider's extent
trace and the timing tools' extent phase are gone. The guest adapter's growth
test now counts the exact device operations of a length change (one flush, two
envelope writes, two flushes) and proves engine bytes untouched across shrink
and regrowth; the host proof checks the same and its crash-cut census fell from
939 to 501 cuts because a length change has fewer operations to cut.

Measurements, host-observed milliseconds on the fresh boot of
`desktop_test.py --latency --nested-depth 4 --trace-storage`, before and after:

| Step | Observed | Provider CPU |
| --- | --- | --- |
| create-user (first, grows the layout) | 1,143 to 350 | 1,004 to 90 |
| enter-user | 1,096 to 1,014 | 580 to 514 |
| depth-1 launch picker | 2,021 to 1,652 | 1,414 to 1,104 |
| depth-4 launch picker | 3,337 to 3,249 | 1,486 to 1,493 |

No `extent` trace appears in the new run. Every other step moved within host
variation in both directions; typing stayed at about 125 ms per key. An
eight-cycle soak at depth two ran 5.1 to 5.5 s per cycle after its first, with
the provider heap peaking at 374,908 of 524,288 bytes and no failure marker.
Artifacts: `build/boot-harness-rs/desktop/20261022T033219328260Z` (latency),
`20261022T033652312438Z` (soak), `20261022T033332334361Z` (four levels under
pointer stress); logs `build/suite5-*.log`.

Verification: guest adapter tests (4), storage-lab host tests (38) and the
volume crash-cut proof; workspace `cargo test --locked` (261); harness Python
tests; clippy for the provider and adapter; `check-userspace`; formatting and
whitespace checks. Guest, on disposable media: the volume transport proof, the
funded-heap database proof, `objects_test.py --retention`,
`matrix_database_test.py`, the plain default desktop with host verification,
four-level latency on fresh and cold boots, four nested desktops under pointer
stress with cold reopen, and the eight-cycle soak.

Next: the fresh-boot entry cost is now the nested template closure installed
at each level's first launch (nine subtree copies, two copies and two named
creations per level, 1.1 to 1.5 s of provider CPU). Whether that closure
should be seeded or shared is an owner-level question about what a Nested
instance owns; measure the copy work and ask before changing it. Bulk program
reads and the per-hop event-loop cost remain as recorded in the previous entry.

## A matrix configured by its realm (2026-10-22)

Baseline: `241412ad`. The owner's framing, in order: a matrix is a program with
files beneath it; its configuration is that program with nested copies of the
programs it expects, so swapping a nested desktop's chrome is swapping a
program; and the presence or absence of those programs decides what the matrix
does, with no special path for matrices. This entry builds that shape, then
takes the live-collection and nested-fixture items left open by the previous
two entries.

**Realm-decided startup.** [startup.rs](source-rs/distribution/matrix/service/children/startup/startup.rs)
replaces the launch argument, the `WORKSPACE_ARGUMENT` contract and the runtime
template preparation (`realm.rs`, deleted). A hosted matrix reads its realm once
its grant has a width: a `Matrix` configuration makes it a workspace that runs
the saved CONFIGURE path itself; a `Login` picker beside a `Users` folder makes
it a desktop that runs that picker over those users; a realm with neither, or
no binding at all (DENIED or PEER_CLOSED from its parent), leaves it to its
owner's commands. [service.rs](source-rs/distribution/matrix/service/service.rs)
starts in `Startup::Realm` when an upstream parent exists and `Startup::Hosted`
otherwise; the `desktop-host` feature of
[main.rs](source-rs/distribution/matrix/main.rs) only encodes the funding a
desktop needs when it is launched into an app slot. An owner's explicit
CONFIGURE is satisfied by a matrix that already configured itself
([configuration.rs](source-rs/distribution/matrix/service/children/configuration/configuration.rs)),
so the boot owner's order no longer races the user matrix's own startup. The
realm selects policy, never authority: every stamp still arrives through the
storage forwarding chain.

**Root login ordering.** The root adapter admitted the chosen user before the
picker retired, so the owner's SURFACE_LIMIT for the retiring picker replaced
the user's grant epoch and its background saw NO_MEMORY on its first placement.
[children.rs](source-rs/distribution/matrix/service/children/children.rs) now
recognizes an owner RUN_OBJECT of the realm's `Login` over its `Users` in slot
zero as the picker, and [launch.rs](source-rs/distribution/matrix/service/children/launch/launch.rs)
retires it before admitting the user it chooses, so the user's first grant is
final; the owner keeps that picker's recovery
([lifetime.rs](source-rs/distribution/matrix/service/children/startup/lifetime.rs)).
The desktop shell's [login state](source-rs/distribution/desktop/session/shell/login.rs)
treats a picker PEER_CLOSED with a live user binding as entry instead of placing
a picker that no longer exists. Surface-create refusals are sampled as
`surface-create-refused` under tracing.

**Prebuilt closure.** A realm entry in the profile is a program or one of three
directives: `@folder`, `@matrix-config` and `@copy=PATH`, a breadth-first copy
of an earlier entry's subtree at revision 1 that numbers siblings before
descendants. [realm_programs.rs](tools/storage-lab/engines/volume/realm_programs/realm_programs.rs)
applies them at install, [desktop.rs](tools/storage-lab/engines/volume/desktop.rs)
models the result independently, `run.py` validates them and
`database_image.py` fingerprints a directive by its text in the installation
receipt. [profile.json](source-rs/distribution/profile.json) installs Nested as
the stock matrix, `Nested/Login`, the `Nested/Users` folder, a `Users/Template`
holding its chrome, Notes, Colors and `Matrix` configuration, and copies of
Nested into each template five levels deep; the fifth template holds no Nested,
so the fourth nested desktop the lab exercises still offers a NESTED tile. A
first launch copies nothing. Creating a user is one subtree copy: 52 objects
beneath the new user for the outer template, 42 for the first nested one. The
plain desktop verifies against the oracle: six base objects, 15 realm entries,
a 53-object user copy and 919 records.

**Collection under pressure.** The store's lab `pressure` feature
([Cargo.toml](source-rs/platform/libraries/database-objects/Cargo.toml)),
reached as `collection-pressure` on the provider and `--collection-pressure` on
`desktop_test.py --soak`, shrinks the workspace to 2 KiB and collects before
every mutation. The six-cycle soak under it ran the windowed collector live in
every cycle: one to three passes, up to 45 windows, no `Workspace` refusal, no
failure marker, the same saved text and a heap peak of 372,996 of 524,288
bytes. Saves took 4 to 10 s per cycle there because every write collects; that
is the feature, not a regression, and the host tests run without it.

**Whole diagnostic lines.** [diagnostics.rs](source-rs/platform/libraries/user-runtime/diagnostics.rs)
sends a message that fits one write as one call even when it straddles a page
in its process, copying through a page-aligned buffer. The kernel prefixes each
call, so the halves of `picker retry exhausted` read as two lines and the
nested-failures fixture failed its count twice before this.

Measurements, host-observed milliseconds on the fresh boot of
`desktop_test.py --latency --nested-depth 4 --trace-storage`, the previous
entry's figures first:

| Step | Observed | Provider CPU | Sectors read/written |
| --- | --- | --- | --- |
| create-user (outer, 53 objects) | 350 to 332 | 90 to 124 | 33/105 to 57/201 |
| enter-user | 1,014 to 1,003 | 514 to 581 | 1,280/16 |
| depth-1 launch picker | 1,652 to 957 | 1,104 to 560 | 1,192/16 |
| depth-4 launch picker | 3,249 to 1,785 | 1,493 to 812 | 1,560/16 |
| depth-1 create-user (now 42 objects) | 377 to 1,079 | 182 to 856 | 129/209 to 977/737 |
| depth-4 create-user (now 15 objects) | 494 to 840 | 121 to 425 | 73/153 to 449/321 |

Nested picker launches no longer install a template; what remains is reading
the picker's program. Nested user creation got slower because the template it copies is now the whole
closure beneath it, and that copy costs far more per object than the outer
one: the depth-1 copy of 42 objects reads 977 sectors and writes 737 with 856 ms
of provider CPU, where the outer copy of 53 reads 57, writes 201 and costs
124 ms. The cold boot reaches the fourth nested desktop with picker launches of
988 to 1,149 ms and user entries of 1,256 to 2,237 ms; typing stayed at 115 to
145 ms per key at every depth. Four nested desktops under pointer stress reach
each level in 5.4 to 8.9 s. The eight-cycle plain soak at depth two ran 4.9 to
5.9 s per cycle with the heap live between 330 and 343 KB, a peak of 375,180
bytes, no collection pass and no failure marker.

Verification:

- Host: workspace `cargo test --locked` (261); storage-lab with
  `--features engines` (39, including the directive install test); harness Python
  (29); clippy for the stock matrix in default, `trace-storage` and
  `desktop-host` builds, the provider with `collection-pressure`, the desktop
  shell and the user runtime; `python tools/source-layout/check.py`;
  formatting and whitespace checks.
- Guest, on disposable media: the plain default desktop with oracle
  verification; four nested desktops under pointer stress with cold reopen
  (twice); four-level latency on fresh and cold boots; the eight-cycle soak at
  depth two and the six-cycle soak under collection pressure; the
  provider-replacement suite (precommit, postcommit, exhausted, readiness-stall,
  nested, deep) on the plain fixture; nested startup failures (picker, user,
  background) on the stress fixture; `--matrix-storage-test`, `--matrix-test`
  and `--matrix-desktop-test`.
- Logs `build/suite6-*.log`, `build/suite7-*.log`, `build/suite8-*.log`.
  Artifacts: latency `build/boot-harness-rs/desktop/20261022T052816562717Z`,
  soak `20261022T051017179655Z`, pressure soak `20261022T051837283716Z`,
  nested failures `build/boot-harness-rs/nested-failures/20261023T004212925180Z`.

Next, in order:

1. Attribute the nested user copy. Forty-two objects at depth one cost seven
   times the IO of fifty-three at the root. Candidates: the cache is cold after
   the picker's 1.2k-sector program read, or each copied object re-proves its
   scope through the nested stamps. The `provider-begin`, `provider-end` and
   `ata-io` samples in the latency serial logs hold the answer; add a copy
   label if they do not.
2. Bulk program reads: 1.2k to 1.6k sectors per picker launch, unchanged, as
   recorded two entries ago.
3. Each Nested level is still two matrix hops; the per-hop event-loop cost
   stands as recorded.
4. The clean-close LOSS timeout did not reproduce in these runs either.
5. The soak samples the provider heap only; kernel task-slot, frame and grant
   accounting per cycle remain to be added.
6. Launching one program over one realm twice is now an owner question in
   [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md). The Nested tile launches the single
   installed matrix, and no lab fixture launches it twice.

## One-page chunks and a two-object provider heap (2026-10-22)

Baseline: `65d7528b`. The previous entry's first item asked why a nested user
copy of 42 objects cost seven times the IO of the outer copy of 53. The latency
run's per-request samples answer it: the whole cost is one COPY_TREE request,
and its reads vary with what the engine cache holds, not with the subtree.
Every picker launch reads a program through the same 256 KiB cache and evicts
the metadata pages the next copy needs. Two things made that worse than it had
to be, and both are fixed here.

**A chunk record fills one engine page.** A 4 KiB chunk, its 34-byte key and
redb's leaf bookkeeping exceeded one 4 KiB page, so redb stored every chunk as
a single-large-value leaf of two pages: a program read moved twice its bytes
through the cache and the stock matrix's 257 KB read moved a megabyte.
[lib.rs](source-rs/platform/libraries/database-objects/lib.rs) sets
`CHUNK_BYTES` to 4,032 so the record fits one page; the manifest bound and the
pin marks use `div_ceil`, the host oracles in the storage lab chunk with the
same constant, and a new store test publishes 32 chunks and checks the table's
fragmented bytes stay under a kilobyte per chunk. The plain desktop still
matches the oracle record for record.

**The provider heap spans two private objects.** [user-heap](source-rs/platform/libraries/user-heap/lib.rs)
now manages up to four extents, allocating first-fit through them in order and
returning each free to the extent that holds it; a third host test covers
overflow into a second extent, frees returning home, overlap and the extent
limit. [heap.rs](source-rs/platform/libraries/user-runtime/heap.rs) installs as
many private objects as the per-object cap requires and releases an object
whose installation fails. The provider's heap is capped at two objects so one
of its four slots stays free for the pages behind a bulk reply, and the desktop
profile funds it with 328 pages: a 1 MiB heap and a 768 KiB cache instead of
512 KiB and 256 KiB.

**A kernel bookkeeping ceiling.** The first run with two heap objects failed its
first bulk reply with NO_MEMORY although the provider had 72 pages and 84 frames
to spare. [address_space.rs](source-rs/kernel/core/user_tasks/address_space/address_space.rs)
bounded a task's mapped backing at its image, stack and two full page objects,
so the heap's two objects left nothing for a reply mapping. The ceiling now
covers every private slot plus two reader mappings; it bounds bookkeeping, not
funding, and every allocation still charges its account. The provider records
`bulk-reply-refused` in every build with the page count, the error and its page
and frame budgets, the soak treats that marker as a failure and the timing
summary accepts it.

**Per-cycle kernel accounting.** After a matrix reclaims a child subtree,
[lifecycle.rs](source-rs/distribution/matrix/service/children/lifecycle.rs)
samples `matrix-reclaimed` under `trace-storage`: the slot, the private and
reader pages the kernel still charges this account, its charged frames and the
launch slots free again. The soak records the cycle's last sample, which is the
root's teardown of the Nested tile, and fails if it differs from the first
cycle's. Across three cycles and a cold boot's two the root settled at 0 private
and 38 reader pages, 104 frames and 22 free slots every time, so a leaked slot,
page or frame now shows as drift rather than as a slower cycle.

Measurements, host-observed milliseconds on the fresh boot of
`desktop_test.py --latency --nested-depth 4 --trace-storage`: the previous
entry, then one-page chunks alone, then both changes.

| Step | Observed | Provider CPU | Sectors read |
| --- | --- | --- | --- |
| enter-user | 1,003 / 808 / 460 | 581 / 306 / 164 | 1,280 / 592 / 568 |
| depth-1 launch picker | 957 / 662 / 374 | 560 / 267 / 107 | 1,192 / 544 / 360 |
| depth-4 launch picker | 1,785 / 1,476 / 949 | 812 / 496 / 40 | 1,560 / 976 / 64 |
| depth-1 create-user | 1,079 / 873 / 278 | 856 / 600 / 62 | 977 / 585 / 1 |
| depth-4 create-user | 840 / 818 / 377 | 425 / 395 / 56 | 449 / 401 / 25 |
| depth-4 enter-user | 2,195 / 2,313 / 1,178 | 667 / 490 / 69 | 1,296 / 968 / 104 |

The nested copy now reads one sector at depth one and 25 at depth four: the
template's pages stay cached across the picker's program read, which itself
moved 360 sectors instead of 1,192 because the chooser's chunks are one page
each and partly shared with the root picker. Picker launches at depth four cost
40 ms of provider CPU instead of 812. The four-cycle soak at depth two ran 4.0
to 4.2 s per cycle (4.9 to 5.9 before), entering each level in 0.9 to 1.3 s,
with the provider heap live at 843 to 858 KB and peaking at 900,132 of
1,048,576 bytes: the engine cache fills to its budget, and the 32 KiB headroom
assertion holds with 148 KB to spare. In the per-stage spans of the new run the
nested dispatch stage averages 0.04 ticks, nested service 0.80, root service
0.58 and the provider request beneath them 0.51; an app's storage round trip
averages 3.9 ticks against 11.5 before. The cold boot's depth-4 user entry took
2,880 ms with 158 ms of provider CPU and 40 sectors read, so what remains there
is not storage.

Verification:

- Host: workspace `cargo test --locked` (262, with the new heap and chunk
  tests); storage-lab with `--features engines` (39); harness Python (30);
  `cargo check-userspace`, `cargo check-uefi`, clippy for the provider with
  `collection-pressure` and the store with `host`; formatting, layout and
  whitespace checks.
- Guest, on disposable media: the plain default desktop with oracle
  verification (twice, with the chunk change alone and with both); four-level
  latency on fresh and cold boots (twice); four nested desktops under pointer
  stress with cold reopen; nested startup failures (picker, user, background);
  the provider-replacement suite (precommit, postcommit, exhausted,
  readiness-stall, nested, deep); `objects_test.py` plain and `--retention`;
  `matrix_database_test.py --recovery`; `--matrix-storage-test`,
  `--matrix-test` and `--matrix-desktop-test`; the four-cycle soak at depth
  two, plain and under collection pressure; a three-cycle soak with the
  accounting check (`build/soak-accounting.log`).
- Logs `build/suite9-*.log` (chunks alone), `build/suite11-*.log` and
  `build/suite12-*.log` (both changes); the two refused boots are
  `build/probe-plain.log` and `build/probe-plain2.log`.

Next, in order:

1. Attribute the cold boot's depth-4 user entry: 2,880 ms observed, 158 ms of
   provider CPU, 40 sectors. The latency JSON carries input and redraw samples
   beside the storage ones; admission of the user matrix and its first
   placement through four hops are the candidates.
2. Each Nested level is two matrix hops, but the hop itself is cheap now:
   in the one-page-chunk run the nested dispatch stage averaged 0.05 ticks,
   nested service 2.8 ticks against 2.4 for the provider request beneath it.
   The remaining per-level time is the provider's own work on program reads
   and copies, plus admission.
3. The clean-close LOSS timeout did not reproduce in these runs either.
4. The soak now holds the root's kernel accounting steady across cycles; the
   nested matrices' samples are recorded but not yet compared, and nothing
   samples the provider's or the desktop's own accounts.
5. Launching one program over one realm twice stays an owner question in
   [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md).

## Fewer round trips per nested level (2026-10-22)

Baseline: `96d0e23c`. With the engine cache no longer thrashing, the cold boot's
depth-4 user entry still took 2,880 ms with 158 ms of provider CPU and 40
sectors. The tick-stamped trace shows why: a request from depth four crosses
eight matrices each way and costs 6 to 7 ticks, and a user entry issued about
36 of them in sequence, each waiting for the last. Two sources were cheap to
remove.

**One request per program.** A program launch read HEAD for the revision and
length, then BULK for the bytes. [BULK](source-rs/platform/contracts/storage.rs)
now accepts an expected revision of zero, the current one, and a length of
zero, the whole object from the offset; [bulk.rs](source-rs/platform/services/storage/bulk.rs)
takes the length from the head inside the one request and the reply states the
revision and length it read. The matrix's [program read](source-rs/distribution/matrix/service/parent/storage/pending.rs)
and its [configuration and launcher reads](source-rs/distribution/matrix/service/children/configuration/configuration.rs)
issue that single request; the HEAD step is gone. Partial reads and BULK_PIN
keep their explicit arguments.

**One request per tile.** The chooser listed executables with FILTER, then read
both metadata fields of each, two round trips per tile, to learn names whose
type the filter had already proved. [storage.rs](source-rs/platform/libraries/user-runtime/storage.rs)
gains `name`, one METADATA round trip, and the [chooser](source-rs/distribution/applications/chooser/session/realm.rs)
uses it; `metadata` still reads both fields and refuses a rename that lands
between them.

Measurements, host-observed milliseconds and provider request spans on
`desktop_test.py --latency --nested-depth 4 --trace-storage`, the previous
entry's figures first:

| Step | Observed | Provider requests |
| --- | --- | --- |
| enter-user (fresh) | 460 to 441 | 27 to 19 |
| depth-1 launch picker (fresh) | 374 to 359 | 11 to 9 |
| depth-4 launch picker (fresh) | 949 to 629 | 18 to 16 |
| depth-4 enter-user (fresh) | 1,178 to 820 | 29 to 22 |
| depth-4 enter-user (cold) | 2,880 to 821 | 29 to 22 |
| create-user, any depth | 278 to 377, now 278 to 363 | 7 to 6 |

Seven requests left each user entry and one left each user creation, and the
cold boot's depth-4 entry now matches the fresh boot's. Typing stayed at 105 to
118 ms per key. One cold-boot cluster at depth two (picker 1,570 ms, entry
2,388 ms, Notes 2,314 ms) ran with 8 to 24 sectors and under 90 ms of provider
CPU per step and no refusal, retry or stale-reply marker in either boot; the
other three depths were ordinary, so it reads as host variance rather than a
guest wait, and the next run should say.

Verification:

- Host: workspace `cargo test --locked` (262); `cargo check-userspace`;
  formatting and whitespace checks. No host test covers the provider's bulk
  path; the guest suites do.
- Guest, on disposable media (`build/suite13-*.log`): four-level latency on
  fresh and cold boots; the plain default desktop with oracle verification;
  four nested desktops under pointer stress with cold reopen; nested startup
  failures (picker, user, background); the provider-replacement suite on the
  plain fixture; `--matrix-storage-test`, `--matrix-test` and
  `--matrix-desktop-test`; `objects_test.py` plain and `--retention`;
  `matrix_database_test.py --recovery`; the three-cycle soak at depth two,
  whose cycles issue 64 provider requests instead of 85 with the root's
  accounting steady at 0 private and 38 reader pages, 104 frames and 22 free
  slots.

Next, in order:

1. A depth-4 user entry still issues 22 requests in sequence, each a 6 to 7
   tick round trip through eight matrices: the realm shape takes three
   LOOKUPs and the configuration a fourth for the same `Matrix` object, the
   chrome lists executables then names each tile. A realm read that answers
   several names at once, or a shape lookup carried into configure, would cut
   the sequence further; the per-hop cost itself is now 0.04 ticks of dispatch
   plus the turn.
2. The clean-close LOSS timeout did not reproduce in these runs either.
3. The soak holds the root's kernel accounting steady across cycles; nested
   matrices' samples are recorded but not yet compared, and nothing samples the
   provider's or the desktop's own accounts.
4. Launching one program over one realm twice stays an owner question in
   [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md).

## A realm named in one listing (2026-10-22)

Baseline: `9b073f4d`. The previous entry left 22 sequential requests in a
depth-4 user entry, most of them the chooser naming its tiles one METADATA at a
time and the matrix looking its realm up one name at a time.

**ENTRIES.** [storage.rs](source-rs/platform/contracts/storage.rs) gains a
nineteenth operation: `parent, 0, after, 0` answers the direct children after
the cursor, in number order, as one sealed page of `Entry` rows (number,
revision, Metadata; 51 per page), with the count and the last listed number in
the packet and no page for an empty listing. It is a read like LIST and FILTER,
so a scope admits it when it admits the parent, and the matrices forward its
reply page as they forward a bulk read's. [objects.rs](source-rs/platform/libraries/database-objects/objects.rs)
gathers the rows in one transaction and a host test checks order, cursors,
bounded output, an empty parent and a missing one; [listing.rs](source-rs/platform/services/storage/listing.rs)
fills the page beside the bulk reply path.

**One listing per reader.** [storage.rs](source-rs/platform/libraries/user-runtime/storage.rs)
reads a page of entries and resolves several names from one listing. The
[chooser](source-rs/distribution/applications/chooser/session/realm.rs) builds
its tiles from the realm's executables, finds the template or the background by
name and the launcher by the name the configuration record carries, all from
one listing plus one READ: two requests where a six-tile launcher spent
fourteen. The matrix's [realm shape](source-rs/distribution/matrix/service/children/startup/startup.rs)
is one listing instead of three LOOKUPs, and it carries the configuration it
found into [configure](source-rs/distribution/matrix/service/children/configuration/configuration.rs)
instead of looking it up again.

**A program must fit one object.** The first guest run of the listing panicked
the provider with `Corrupt` on its first ENTRIES. The store had never been
asked for the hidden `Template` entry's head before, and that head's length was
269,856 bytes: the first reader of the listing carried page-sized `Entry`
arrays and a per-row decode loop that grew the stock matrix by 14 KB of code,
past the 256 KiB object limit, and the installer had written the oversized head
without complaint. The reader now decodes rows on demand from the sealed page
(`Listing`), which puts the matrix back at 257,568 bytes, and the installer's
[seed.rs](tools/storage-lab/engines/volume/seed.rs) refuses any program over
`MAX_BYTES` with its size in the message, covered by a lab test, so the next
such growth fails at install rather than as a corrupt head on first read.

**A chooser is lent two reader pages.** The second guest run refused the first
listing's page at the root matrix with DENIED: pickers and launchers had always
been funded with zero reader pages, since nothing they read came back as an
attachment. [matrix.rs](source-rs/distribution/matrix/contracts/matrix.rs) names
`CHOOSER_READERS`, two pages, and the shell's root picker, the nested picker and
the configured launcher are lent that beside their private pages.

**Size headroom.** The stock matrix is now built at `opt-level = "s"`
([Cargo.toml](source-rs/Cargo.toml)): 220,704 bytes trimmed instead of 257,568,
leaving 16 percent of the object limit rather than two. The four-level latency
run measured no change: nested service averaged 0.84 ms of CPU per request
against 0.87 before, root service 0.96 against 0.92, typing 107 to 123 ms per
key either way, and every step moved within host variation. Four nested
desktops under pointer stress, the nested startup failures, the three matrix
suites and a three-cycle soak pass on it (`build/suite17-*.log`), the soak's
root settling at 95 frames after each teardown instead of 105 with the smaller
image. `run.py` prints each trimmed program's size as a share of the limit at
every build and refuses one over it before the installer does.

Measurements, host-observed milliseconds and provider request spans on
`desktop_test.py --latency --nested-depth 4 --trace-storage`, the previous
entry's figures first:

| Step | Observed | Provider requests |
| --- | --- | --- |
| enter-user (fresh) | 441 to 365 | 19 to 9 |
| depth-1 launch picker (fresh) | 359 to 354 | 9 to 5 |
| depth-4 launch picker (fresh) | 629 to 510 | 16 to 12 |
| depth-4 enter-user (fresh) | 820 to 617 | 22 to 13 |
| depth-4 enter-user (cold) | 821 to 645 | 22 to 13 |
| create-user, any depth | 278 to 363, now 283 to 295 | 6 to 3 |

A user entry at depth four now issues thirteen requests where the entry two
steps back issued twenty-nine, and takes 617 ms where it took 1,178 on a fresh
boot and 2,880 on a cold one. A user creation is three requests: the copy, the
listing and the configuration read.

Verification:

- Host: workspace `cargo test --locked` (262); the store's host tests with
  `--features host` (22, with the entries test); storage-lab with
  `--features engines` (40, with the oversized-program refusal);
  `cargo check-userspace`; formatting, layout and whitespace checks.
  The stock matrix trims to 257,568 bytes and the chooser to 78,008.
- Guest, on disposable media (`build/suite16-*.log`): four-level latency on
  fresh and cold boots; the plain default desktop with oracle verification;
  four nested desktops under pointer stress with cold reopen; nested startup
  failures (picker, user, background); the provider-replacement suite on the
  plain fixture; `--matrix-storage-test`, `--matrix-test` and
  `--matrix-desktop-test`; `objects_test.py` plain and `--retention`;
  `matrix_database_test.py --recovery`; the three-cycle soak at depth two,
  whose cycles issue 36 provider requests instead of 64 and run 3.2 to 3.5 s
  with the root's accounting steady at 0 private and 38 reader pages, 105
  frames and 22 free slots. The two refused runs are `build/suite14-*.log` (the
  oversized matrix) and `build/suite15-*.log` (no reader pages).

Next, in order:

1. A depth-4 user entry still issues 13 sequential requests, each a 6 to 7
   tick round trip through eight matrices: the matrix's listing, two bulk
   reads for the configuration and the launcher, the background's lookup and
   read, the launcher's listing and configuration read, and the picker
   retirement around them. The background's lookup could ride the listing the
   matrix already read, and a launcher could receive its arrangement with its
   launch instead of reading it; beyond that the hop count itself is the cost.
2. The stock matrix is 220,704 bytes against a 262,144-byte object limit and
   every build prints the margin. If it grows past that, the limit would have to
   grow with the provider's reply pages, which is a budget decision across the
   provider, the matrices' reader allowances and the kernel's backing ceiling.
3. The clean-close LOSS timeout did not reproduce in these runs either.
4. The soak holds the root's kernel accounting steady across cycles; nested
   matrices' samples are recorded but not yet compared, and nothing samples the
   provider's or the desktop's own accounts.
5. Launching one program over one realm twice stays an owner question in
   [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md).

## Where a forwarded request spends its time (2026-10-22)

Baseline: `44b11710`. With the request count per nested level cut, the hop
itself is what remains, and ticks were too coarse to see inside one. This entry
adds the clock to see it and reports what it shows.

**A microsecond clock in every stage trace.** [CLOCK_READ](source-rs/kernel/contracts/user.rs)
with a 24-byte record writes the tick, the kernel's microsecond clock behind it
(the TSC-derived clock the scheduler already accounts with) and this task's
measured execution, in one read; [time.rs](source-rs/platform/libraries/user-runtime/time.rs)
uses it for `storage_trace`, so every stage line carries the microsecond clock
as its fifth value and costs two syscalls and a write instead of five and a
write. The timing parsers accept the fifth value. Each matrix writes two
`trace-calibrate` stamps back to back at startup, so the cost of a trace line
itself is in the log beside what it measures.

**hop_latency.py.** [The tool](tools/boot-harness-rs/hop_latency.py) pairs
consecutive stages of a request chain in serial order and reports each
transition's count, total, mean, median, median net of the calibration and p90:
a send to the next receive is the hop (kernel delivery plus the receiving
matrix's wake), receive to admit and admit to send are that matrix's own turn,
and the provider's begin to end is its work. A unit test covers the pairing,
a backwards clock and the calibration.

Measurements, microseconds from the fresh boot of
`desktop_test.py --latency --nested-depth 4 --trace-storage`
(`build/latency-stamp.log`), medians with the trace line's own 231 Ãƒâ€šÃ‚Âµs
subtracted, then raw p90:

| Transition | Count | Net median | Raw p90 |
| --- | --- | --- | --- |
| hop up, matrix to matrix | 301 | 293 | 782 |
| hop down, matrix to matrix | 334 | 406 | 1,351 |
| hop up, root to provider | 133 | 322 | 673 |
| hop down, provider to root | 72 | 267 | 781 |
| matrix dispatch (receive to admit) | 425 | 22 | 378 |
| matrix forward (admit to send) | 437 | 8 | 338 |
| root dispatch and forward | 233 | 10 and 2 | 324 and 280 |
| provider work | 88 | 406 | 27,719 |

The cold boot reads the same within 30 Ãƒâ€šÃ‚Âµs on every row. So a hop costs 300 to
400 Ãƒâ€šÃ‚Âµs under TCG, where the matrix's own work on a request is about 30 Ãƒâ€šÃ‚Âµs: the
time is kernel delivery plus the receiving matrix's wake and switch, not matrix
code. A depth-4 request crosses eight or nine hops each way, sixteen to
eighteen in all, for about 6 ms, and the thirteen requests of a user entry
about 85 ms of its 581; the rest is admission, placement and drawing. The downward hop costs more than the upward
one because a matrix does more after answering than after forwarding before it
reaches its wait.

**Yield after forward and reply.** The scheduler gives a woken task its turn
when the running one blocks, yields or the timer fires, so a matrix's loop tail
sat between every hop. [pending.rs](source-rs/distribution/matrix/service/parent/storage/pending.rs)
now yields right after forwarding a request upstream and right after answering
a child (`build/latency-yield.log`): the hop down from the root to a matrix fell
from 222 to 54 Ãƒâ€šÃ‚Âµs net and matrix to matrix from 406 to 368, the depth-4 user
entry from 581 to 519 ms and the depth-1 picker launch from 354 to 280. The hop
up stayed at about 290 Ãƒâ€šÃ‚Âµs net, so what remains is the kernel's switch itself
under emulation: two syscall transitions and a page-table switch, with the
yield's return through the boot context when deferred work is pending. That is
a property of the lab's single CPU under TCG far more than of the design, and
the right fix, if one is wanted, is in the kernel's switch path rather than in
any matrix.

**Kernel phases.** The smoke session's report (`run.py --smoke --release`; the
suite takes 42 s here, so the default boot timeout is now 120 s instead of 30)
puts
the kernel's own work far below the hop: a trap averages 5 Ãƒâ€šÃ‚Âµs and a syscall
2 Ãƒâ€šÃ‚Âµs across 190,000 of each, and a task whose registered wait completes is
selected 8 to 13 Ãƒâ€šÃ‚Âµs later with a yielding peer. The IPC lab's echo client now
times its thirty-two round trips with a clock grant (`ipc-roundtrip`): 3,004
and 2,957 Ãƒâ€šÃ‚Âµs in the two rounds, so a round trip between two small tasks costs
94 Ãƒâ€šÃ‚Âµs and a switch about 45, four syscalls included. The matrix hop's 290 Ãƒâ€šÃ‚Âµs is
therefore neither trap handling, scheduler latency nor the switch itself: it
is the woken matrix re-touching its working set, static arenas and mapped
objects alike, through the emulator's translation cache after the page-table
reload, before it reaches its stamp. Real hardware pays a few microseconds
for all of it. A direct switch to the woken task would save the yield's own
trap, about 5 Ãƒâ€šÃ‚Âµs; the hop count, not the hop, is the lab's lever, and that
count is now what the realm layout requires.

**Two older items close.** Bulk program reads are no longer amplified: a user
entry reads the 220 KB stock matrix in 496 sectors, 1.15 times its size, where
it read 1,280 for 257 KB before the one-page chunks. The clean-close LOSS
timeout left no trace in this session's 310 desktop runs: no stale control
reply, the three storage retirements are the provider panics of the two
refused listing builds, and the 23 close refusals all report a sibling that
answered with unsaved work, which is the protected-close check the tests
exercise on purpose.

Verification:

- Host: workspace `cargo test --locked` (262); `cargo check-uefi` and the
  kernel's host tests (144) for the clock read; `cargo check-userspace`; harness
  Python (31, with the hop tool's test); formatting, layout and whitespace
  checks.
- Guest, on disposable media: four-level latency on fresh and cold boots three
  times (`build/latency-micros.log`, `build/latency-stamp.log`,
  `build/latency-yield.log`); then with the yield (`build/suite19-*.log`) the
  plain default desktop with oracle verification, four nested desktops under
  pointer stress with cold reopen, nested startup failures, the
  provider-replacement suite, `--matrix-storage-test`, `--matrix-test` and
  `--matrix-desktop-test`, and the three-cycle soak at depth two with the root's
  accounting steady at 0 private and 38 reader pages, 95 frames and 22 free
  slots; `build/suite18-*.log` covers the clock read alone on the plain desktop
  and the two input-traced matrix suites. `build/soak-accounts.log` is the
  three-cycle soak with every reclamation sample compared: each cycle, in both
  boots, reports the two picker retirements, the nested desktop's Notes close
  and the root's teardown with identical pages, frames and free slots. The
  later changes in this section each ran the same suites again:
  `build/suite20-*.log` (background in the configure listing),
  `build/suite21-*.log` (held close deadline), `build/suite22-*.log` (launch
  argument), `build/soak-custody.log` (provider custody floor) and
  `build/soak-shell.log` (shell account floor); `build/smoke-roundtrip.log` is
  the smoke suite with the timed echo exchange. A soak cycle now issues 32
  provider requests, from 85 at the start of the day.

Next, in order:

1. The per-hop cost is about 290 Ãƒâ€šÃ‚Âµs under TCG: a bare switch between two
   small tasks measures 45 Ãƒâ€šÃ‚Âµs, the kernel's trap work 5 Ãƒâ€šÃ‚Âµs and selection
   within 13 Ãƒâ€šÃ‚Âµs of completion, so the rest is the woken matrix re-touching its
   working set through the emulator's translation cache. The receive path was
   tried as a lever: a stamp at the moment the wait returned
   (`build/latency-wake.log`, `build/latency-wake2.log`) put the whole up hop
   before the wake and none of it in the matrix's turn, and showed the server
   rescanning every client after a wake; the wait now points the poll at the
   channel the kernel reported ready and a scan reads the clock once
   ([polling.rs](source-rs/platform/libraries/user-runtime/server/polling.rs)),
   which removes a receive per quiet client per wake but leaves the hop at
   294 Ãƒâ€šÃ‚Âµs net (`build/latency-poll3.log`) against 287 to 293 before. The
   per-turn stamp itself cost 250 Ãƒâ€šÃ‚Âµs a turn and was retired. The refill's
   origin is in [entry.S](source-rs/kernel/arch/isa/x86_64/interrupts/entry.S):
   every trap switches to the kernel's own root on entry and back to the
   task's root on return, and no mapping is global
   ([inherited.rs](source-rs/kernel/arch/isa/x86_64/memory/inherited.rs)
   strips the flag, [creation.rs](source-rs/kernel/arch/isa/x86_64/user/memory/creation.rs)
   masks it), so every syscall drops both the kernel's and the task's
   translations and the emulator re-walks them: 45 Ãƒâ€šÃ‚Âµs for a tiny task, far
   more for a matrix. Global supervisor leaves (the seven identical ranges and
   the slot-unique task stacks) would spare the kernel's half; keeping the
   task's half needs PCID or the kernel running in the task's root, which is
   the isolation boundary the owner has left open. Both are recorded under
   that question in OWNER_QUESTIONS.md with this evidence; a direct switch to
   the woken task is the CPU sponsorship question there. Nothing in the
   matrices or the server is left to shave.
2. A depth-4 user entry issues 11 sequential requests, each with its reason
   (`build/latency-poll3.log`): one BULK reading the user matrix's program;
   four HEADs, one per nesting level, each matrix's first-time proof that the
   new binding's scope lies within the one it was granted, which the shared
   scope machinery then retains for every later request; one ENTRIES for the
   realm's shape; one BULK for the configuration; one ENTRIES naming the
   launcher and the background; two BULKs for their programs; and the
   launcher's own ENTRIES for its tiles. The root entry is the same without
   the proofs, seven requests, and a depth-1 entry eight. The background rides
   the configure listing (`build/suite20-latency-four.log`) and a program the
   matrix runs from its realm receives its own object above a role byte in
   its launch argument ([matrix.rs](source-rs/distribution/matrix/contracts/matrix.rs)),
   so the launcher hides itself without a read (`build/suite22-latency-four.log`).
   The proofs are the trust model, one per matrix that forwards; the rest is
   the realm's content. The entry's remaining time is admission, placement and
   drawing.
3. The clean-close LOSS timeout: the mechanism is in
   [control.rs](source-rs/distribution/matrix/service/children/control/control.rs). A
   close's answer is left queued while this matrix's storage is busy, and the
   deadline used to keep running meanwhile, so the first poll after a long
   busy window could refuse a sibling that had answered cleanly as timed out.
   The pending deadline is now held at least the outcome grace ahead of now
   for as long as the matrix is busy, so only the sibling's own silence can
   time a close out. It left no trace in this session's runs before or after;
   `build/suite21-*.log` covers the fix on the close, failure, provider and
   soak suites.
4. The soak now holds every matrix's reclamation sample steady across cycles,
   in order: the nested desktop's after closing Notes and the root's after the
   teardown. The provider's heap report carries the private pages and frames
   the kernel charges it, which include reply pages their readers still hold,
   and the soak holds each cycle's quietest level: 256 pages and 533 frames,
   the two heap objects alone, in every cycle of both boots
   (`build/soak-custody.log`). The shell samples its own account about once a
   second while it idles under storage tracing (`shell-account`), and the soak
   holds its floor too: no private or reader pages, 46 frames and no free
   launch slots in every cycle of both boots (`build/soak-shell.log`). Every
   account in a session now has a per-cycle check.
5. Launching one program over one realm twice stays an owner question in
   [OWNER_QUESTIONS.md](OWNER_QUESTIONS.md).

## The hop on hardware, and global shared leaves (2026-10-22)

Baseline: `f8bab679`. The section before this one attributed the per-hop cost
under TCG to the two root switches of every trap and named global supervisor
leaves as the one lever that needs no boundary decision. This entry measures
the hop where paging is the host CPU's, takes that lever, and leaves the lab
able to time either way.

**TCG cannot show global pages.** QEMU's `cpu_x86_update_cr3` flushes the whole
software TLB on every CR3 write while paging is on, `cpu_x86_update_cr4`
flushes again only when PGE itself changes, and the x86 page walker never reads
the global bit (v9.2.0 and current master alike). Under TCG a global leaf
therefore costs nothing and saves nothing, and the 45 Ãƒâ€šÃ‚Âµs a bare switch costs
there is the emulator rebuilding its own TLB, not the guest's. The lab's QEMU
(11.0.50) also carries WHPX, the Windows hypervisor accelerator, under which
the guest's page tables and TLB run on the host CPU. [run.py](tools/boot-harness-rs/run.py)
and [desktop_test.py](tools/boot-harness-rs/desktop_test.py) take `--accel whpx`.
The guest cannot see QEMU's VM port under WHPX, so the desktop runs there with
`--relative-pointer`, which the latency and soak workloads now accept, and the
latency workload's pointer steps go through the transport-aware `point` helper
instead of sending absolute positions themselves. The smoke suite and the
depth-4 latency workload pass under WHPX
(`build/whpx-smoke.log`, `build/boot-harness-rs/desktop/20261023T060538222615Z`).

**What hardware pays.** Every serial byte is a hypervisor exit under WHPX, so a
trace line costs about 1,150 Ãƒâ€šÃ‚Âµs there against 231 under TCG, and the hop figures
net of it sit inside its own noise: 0 to 140 Ãƒâ€šÃ‚Âµs for the hop up between matrices
and 39 to 224 down over two boots, against 290 and 370 under TCG. A traced
workload is therefore bound by its own tracing under WHPX, and the untraced
entry is the end-to-end figure. All TCG figures here are today's: by the
afternoon the host ran TCG about 1.6 times slower than in the morning's runs,
the trace line itself at 434 to 649 Ãƒâ€šÃ‚Âµs against 231, on the unchanged kernel as
well (820 to 959 ms for the traced depth-4 entry before the global
leaves, `build/boot-harness-rs/desktop/20261023T063432217407Z`), so a figure compares only with one taken
beside it.

| Figure, 32 round trips or one depth-4 step | TCG | WHPX |
| --- | --- | --- |
| bare round trip between two tasks, fastest burst | 3,140 to 4,824 Ãƒâ€šÃ‚Âµs | 118 to 144 Ãƒâ€šÃ‚Âµs |
| the whole exchange, eight bursts | 36,977 to 44,346 Ãƒâ€šÃ‚Âµs | 1,094 to 1,217 Ãƒâ€šÃ‚Âµs |
| trace line itself | 434 to 649 Ãƒâ€šÃ‚Âµs (231 in the morning) | about 1,150 Ãƒâ€šÃ‚Âµs |
| hop up between matrices, net median | 332 to 512 Ãƒâ€šÃ‚Âµs (290 in the morning) | within trace noise |
| depth-4 user entry, traced | 828 to 851 ms | 787 to 989 ms |
| depth-4 user entry, untraced | 620 to 629 ms | 329 to 431 ms |
| depth-4 picker launch, untraced | 460 to 533 ms | 777 to 785 ms |

(`build/final*-smoke-*.log`, `build/boot-harness-rs/desktop/20261023T063541759172Z`, `build/boot-harness-rs/desktop/20261023T060759118274Z`, `build/boot-harness-rs/desktop/20261023T063026021739Z`,
`build/boot-harness-rs/desktop/20261023T062823481668Z`.) Two emulation costs shape the WHPX column. Every word
of an ATA PIO transfer is a port-I/O exit, so the steps that read programs from
disk are slower there: the root entry's 496 sectors take 1,655 ms against 553,
the depth-1 picker launch's 296 take 1,362 against 464, while the depth-4 steps
read 16 sectors or none and show the kernel and the matrices instead. And the
desktop runs on the relative pointer there, so a step that clicks carries the
pointer's paced walk, about 40 ms per 70 pixels, which the TCG run on the
absolute pointer does not pay; the depth-4 picker launch is mostly that walk.
What the kernel and the matrices add on hardware is small beside both.

**Global shared leaves.** [shared.rs](source-rs/kernel/arch/isa/x86_64/memory/shared.rs)
names the supervisor ranges every root maps identically, the image, the boot
and emergency stacks and the context heap, and marks the image's identity
leaves global in the kernel root after [inherited.rs](source-rs/kernel/arch/isa/x86_64/memory/inherited.rs)
has copied the firmware's tables; the stack and heap leaves are mapped global,
and [creation.rs](source-rs/kernel/arch/isa/x86_64/user/memory/creation.rs)
copies the flag into every user root with the leaf. Activation retires the
firmware's global translations and sets CR4.PGE; the user session asserts it
instead of clearing it. Nothing else is global: the firmware's aliases of
image frames and the per-slot task stacks stay per root and are retired by
each switch, so a restricted root inherits no translation from the TLB that it
does not map, and no flush site changes, since the shared ranges never change
after boot. A host test walks a copied tree and checks the identity leaves are
global, the alias of the same frames is not and the neighbours are untouched.
Measured as an interleaved A/B, three smoke runs each way under each
accelerator with a 256-round exchange: under WHPX 1,310 to 1,359 Ãƒâ€šÃ‚Âµs without
the global leaves and 1,128 to 1,175 with (one run 3,508 with a host
preemption inside), about 15 percent less per round trip, 5.2 to 4.5 Ãƒâ€šÃ‚Âµs; under
TCG 38.4 to 38.5 ms without and 36.9 to 42.0 with, unchanged within noise, as
QEMU's source says it must be; on the desktop workload under TCG, run beside
each other, the traced depth-4 entry took 820 to 959 ms without and
828 to 851 with. Kept: correct by construction, no isolation
cost, and the kernel's half of every switch spared on hardware. The task's
half still needs PCID or the kernel executing in the task's root, which is the
isolation boundary the owner has left open; both evidence paragraphs in
OWNER_QUESTIONS.md carry the hardware figures now.

**Where the rest of the entry goes.** [startup_timeline.py](tools/boot-harness-rs/startup_timeline.py)
lists one matrix startup from its calibration stamps: every request of its
chain that reached the provider, in milliseconds after it started, until its
launcher names the tiles. For the depth-4 matrix of today's traced TCG run
(`build/boot-harness-rs/desktop/20261023T063541759172Z`, boot 1) the four
scope proofs are answered by 83 ms, the realm's shape at 86, the configuration
at 117, the launcher's and background's names at 161, their programs at 195 and
252, the background is admitted and the matrix configured by about 280, and the
launcher's tiles are named at 315; the next request, at 701 ms, is the Notes
launch the harness clicks afterwards. So the matrix's startup is about 315 ms
of the 828 ms step and the launcher's admission about 28 of them; the rest is
the launcher drawing, the compositor, and the harness itself, whose every
clicked step carries 210 ms of input spacing and a 50 ms pause between
screendumps, which is why the record calls its figures upper bounds. Under
WHPX the same chain reads 538 ms traced, nearly all of it the chain's own five
hundred trace lines at 1.15 ms each; untraced, the whole step is 329 to 431 ms
with the pointer's walk inside it, so on hardware the entry sits at the
harness's floor.

**The held close deadline, pinned.** The clean-close fix in
[control.rs](source-rs/distribution/matrix/service/children/control/control.rs), which
keeps a close answer's deadline at least the outcome grace ahead while this
matrix's storage is busy, had only non-reproduction behind it. The test-only
[delay probe](source-rs/distribution/matrix/service/delay_probe.rs) lets F4
travel on from the root, and the matrix on the focused chain that receives it
from its parent arms its next close to count its storage as busy for two
seconds, exchange or none, longer than the answer's 100-tick deadline and
shorter than the chrome's 600-tick wait for the close; that close keeps its
deadline from the moment it was asked but sends its LOSS only when the window
ends ([lifecycle.rs](source-rs/distribution/matrix/service/children/lifecycle.rs)),
so the sibling's clean answer lands after the busy window and inside the
grace, which is the only timing the fix decides: a matrix polls a close's
control channel without blocking, so an answer already queued is read however
late the poll. The interaction-delay suite closes Notes in the outer desktop
from the taskbar under that window (a window's own control is the app leaving
by itself, no close at all; and that desktop is the user matrix under the
storage root, which is why the root cannot arm for it): the answer is accepted
and the close completes
(`build/hold-fixed.log`); with the extension removed the same step fails
(the late answer is refused as timed out, Notes stays, and the desktop never reports closed and collected; `build/hold-unfixed.log`), so the scenario detects the bug it
pins. Two earlier shapes could not: a busy window armed before the close
never sees one, because a matrix defers its children's requests while its
storage is busy, so the chrome's close waited for the window to end; and an
unrelated exchange at the root would have been a launch, which the root
refuses at admission beside a live three-deep subtree. Counting the storage as
busy from the close itself, with nothing in flight, is what makes the timing
reachable.

**The exchange in bursts.** 256 round trips under TCG cost 150 Ãƒâ€šÃ‚Âµs each against
93 for 32, because ten-millisecond ticks land inside a 38 ms exchange. The echo
client now runs eight bursts of 32, times each, and reports the fastest burst
beside the whole exchange (`ipc-roundtrip 32 <fastest> <total>`); the fastest
burst is the switch cost with no tick or host preemption inside it.
[lab.rs](source-rs/kernel/contracts/lab.rs) holds the counts for the kernel's
lab and the program alike, so the lab's accounting check and the program agree
by construction.

Verification:

- Host: workspace `cargo test --locked` (263); `cargo check-uefi` and the
  arch crate's host tests (13, with the shared-leaves test); `cargo
  check-userspace`; harness Python (12, with the startup timeline's);
  formatting and layout checks.
- Guest: the interaction-delay suite with the storage stall, passing with the
  held deadline and failing with it removed (`build/hold-fixed.log`,
  `build/hold-unfixed.log`); the smoke suite under TCG and WHPX twice each on
  the final tree
  (`build/final-smoke-*.log`); the interleaved A/B (`build/ab-*.log`); the
  depth-4 latency workload with traces under WHPX before and after the global
  leaves (`build/boot-harness-rs/desktop/20261023T060538222615Z`, `build/boot-harness-rs/desktop/20261023T060759118274Z`)
  and under TCG before and after (`build/boot-harness-rs/desktop/20261023T063432217407Z`, `build/boot-harness-rs/desktop/20261023T063541759172Z`); the
  untraced depth-4 workload under both (`build/boot-harness-rs/desktop/20261023T062823481668Z`,
  `build/boot-harness-rs/desktop/20261023T063026021739Z`).

Next, in order:

1. The switch path has nothing left to take without a boundary decision: the
   kernel's translations survive every trap, the task's would need PCID or the
   kernel in the task's root. PCID is possible on this host under WHPX
   (`-cpu qemu64,+pcid,+invpcid` is accepted) and impossible under TCG, whose
   CR3 write rejects the no-flush bit and whose TLB flushes whole regardless
   (QEMU source), so it would be a hypervisor-only kernel path with
   hypervisor-only validation. Both live under the isolation-boundary question
   in OWNER_QUESTIONS.md with the TCG and WHPX figures, as the owner's call.
2. Under TCG the startup chain is hop-bound, and the one lever left inside it
   was the four scope proofs: each the first request's proof, at the matrix
   that cannot add a third stamp, that the innermost scope lies within the
   other two, answered one after another in the first 83 ms of a depth-4
   startup. Warming them was tried (2026-10-23): the granting matrix sent a
   HEAD on the new scope through the child's binding right after the grant,
   before the child asked anything. On the lab's one CPU that chain runs
   ahead of the child's deferred first request instead of beside it, and adds
   a request of its own: the depth-4 chain measured 357 to 385 ms with
   eleven provider requests against 312 to 365 with ten in alternating runs
   (`build/warm-*.log`), and no entry step moved, so the code is gone again.
   OWNER_QUESTIONS.md keeps the trust question (a parent exercising a
   child's binding) with these figures; only a host where the chain could
   overlap the child's start would change them, and there a hop costs
   microseconds.
3. Timing taken under one accelerator compares only with timing under the
   same one. The TCG figures in the sections before this one stand; a figure
   for hardware is a WHPX run of the same command.
4. These experiments raised questions about instances, CPU sponsorship, isolation
   and parent-generated proof warming; the current specification and owner queue
   supersede that historical list. On 2026-10-25 the owner rejected grant-time
   requests through the child's binding: a storage grant need never be used.
   Preserve [demand-driven checks](wiki/spec/execution/components_and_matrices.md#storage-access-is-demand-driven)
   in future storage work. The warming experiment above is archived evidence,
   not a pending optimization task or a reason to manufacture child traffic.


## CPU budget decision: archived lab evidence (2026-10-23)

The owner has accepted [executing-program CPU charging](wiki/spec/resources/execution_budgets.md). The following evidence was moved from the closed owner question to retain its measurements and implementation links. Its historical references to unresolved CPU sponsorship are superseded by that contract; caller-funded execution is deferred. Other memory, capability and accounting limitations remain limitations. No tests were rerun for this documentation change.

**Rust experiment (not resolution).** [Hierarchical turns](source-rs/kernel/core/scheduler/README.md)
now follow current launch ownership. Descendants divide their owner's outer turns;
loans reattach dormant scheduling accounts, and reclamation returns them without
resetting rotation. A provider still runs within its own launch tree even when
serving another tree. The QEMU proof compares one versus three spinning descendants
against an outside account. The measured-budget extension now charges elapsed
execution, spawn attempts, page operations and reclamation through the ancestor
chain. Boot-selected burst/refill ceilings retain overrun debt through restart
and loan transfer; QEMU checks spinners, yields, repeated replacement and idle
refill. IRQ handling while a process is selected is charged to that process;
general boot/idle housekeeping has no application sponsor. These are provisional
attribution choices and ceilings, not guaranteed CPU-time reservations. The
periodic timer, unpreemptible kernel work and lab clock do not establish hard
response bounds. Shared-provider request funding remains the unresolved choice
above; merely registering a provider confers no extra scheduling authority.


**Forwarding-hop lab evidence (2026-10-22, not a scheduling decision).** With
a microsecond clock in the storage traces, one hop of a nested storage request
from a matrix to its parent costs about 290 Ãƒâ€šÃ‚Âµs under TCG net of tracing, while
the matrix's own work on the request is about 30 Ãƒâ€šÃ‚Âµs; a depth-4 request crosses
sixteen to eighteen hops. The scheduler gives a woken receiver its turn only
when the running task blocks, yields or the timer fires, so matrices now yield
right after forwarding or answering, which shortens the downward hops but
leaves the upward hop near 290 Ãƒâ€šÃ‚Âµs. The kernel's trap work is 5 Ãƒâ€šÃ‚Âµs, a woken task
is selected within 13 Ãƒâ€šÃ‚Âµs of its completion, and a bare round trip between two
small tasks measures 94 Ãƒâ€šÃ‚Âµs in the IPC lab, so the rest is the woken matrix
re-touching its working set through the emulator's translation cache after the
page-table reload; on hardware (the Windows hypervisor on the lab host) the
same 32 round trips take about 118 to 144 Ãƒâ€šÃ‚Âµs and the hop sits inside the
trace line's own noise. A direct switch to the task an IPC wakes would therefore
save only the yield's trap, and what it decides is whose turn funds the
receiver's work, which is this question; the lab keeps the yield and leaves the
switch policy open.

**Resource-subdivision lab evidence (not a production sponsorship decision).**
[The Rust resource coordinator](source-rs/kernel/core/user_tasks/resources/README.md) now
conserves runtime page/reader/channel allowances across owner-to-child assignment.
Only owned idle launch accounts can be resized, and every family validates before
commit. Accepted pages now retain the original backing sponsor after producer
death; restart cannot buy replacement capacity while a reader pins it. Explicit
withdrawal fails rather than revoking the reader. The same applies to channels.
This extends the persistent-sponsor experiment above. It does not settle funding
transfer or active-child rebudgeting. The bounded physical-memory experiment
below now includes ELF, page-table and stack admission costs.


**Executable-admission lab evidence (not a stable delegation contract).**
[Dynamic admission](source-rs/kernel/core/user_tasks/admission/pool/README.md) now separates
code authority from a bounded process pool. Exporting an owned plain launch
creates a distinct code grant; dynamic admission explicitly supplies that code
grant to its child for funded recursive launch. Device-granted launches cannot
be exported. Creator death uses the existing nearest-surviving-lender policy for
new accounts too, while retained backing can pin returned reservations. Production
semantics still need to decide general executable-capability transfer/revocation
and whether code grants should be retained independently of launch ownership.
The current lab grants permission to execute again; it does not infer that right
from arbitrary readable bytes or service registration.


**Matrix integration experiment.** Code-only boot catalogue grants no longer
consume dormant process accounts. An owner may explicitly stage copied executable
authority into an idle child; admission publishes fresh local names after load.
The matrix desktop uses this path recursively. This adopts copying for the lab,
not a production revocation contract: changing the launch's staging does not revoke
an independently installed grant, and selected dynamic code remains executable
by descendants within supplied process/resource budgets. Executable-grant lifetime
remains a separate contract; local IPC notification withdrawal and retained-resource
accounting now follow the [accepted notification rules](wiki/spec/execution/notifications.md).
Idle-child
CPU ceiling assignment preserves ancestor enforcement and debt; it confers no
priority or permission to charge an unrelated service's work to a caller.

**Physical-memory funding experiment (historical implementation evidence).**
The [memory-budget contract](wiki/spec/resources/memory_budgets.md) now settles
hierarchical conservation, retained charging and optional allocator interception.
The following measurements and sizes remain historical lab evidence, not new runs
or a production allocator ABI.
The [Rust frame bank](source-rs/kernel/core/user_tasks/resources/physical/README.md) now backs
conserved allowances with real reserved RAM. Executable pages, stacks, private
page tables and runtime buffers charge the same account. Retained readers keep
the original producer's physical charge after process death; final release refunds
it. Kernel heap/IPC arenas and boot/device reservations remain separate bounded
costs. Nested exhaustion cannot spend a sibling's reservation.

Bootstrap sizing is provisionally 256 frames per bound account plus private-page
quota, and another 256 per unbound process slot initially held by the root. This
eager, fixed session bank is lab policy, not the eventual manifest contract.
Automatic sponsorship transfer, forced revocation, exact refund routing and
production envelope-management APIs remain outside that partial contract. They
cannot erase retained charges or reclaim live backing. These exclusions do not
block the direct-grant and interposed-provider outcome above.


## Isolation decision: archived trap-path evidence (2026-10-23)

The isolation decision is settled by the contract linked above. The following
historical measurements are retained from the closed question; its discussion
of the isolation boundary as undecided is superseded. PCID validation coverage
remains a separate deferred choice, and these results were not rerun here.

**Trap-path lab evidence (2026-10-22, not a boundary decision).** The Rust lab
keeps the kernel in its own page-table root: every trap switches roots on entry
and back on return. The seven supervisor ranges every root shares (image,
stacks, context heap) are global, so the kernel's translations survive the
switches; the task's are dropped by each one. 32 bare round trips between two
tiny tasks cost about 3,140 to 4,824 Ãƒâ€šÃ‚Âµs under TCG, where the emulator rebuilds its
whole TLB on every root switch whatever the guest marks global, and about
118 to 144 Ãƒâ€šÃ‚Âµs under the Windows hypervisor on the lab host, 15 percent less
than before the global leaves. A storage hop between matrices
is about 290 Ãƒâ€šÃ‚Âµs net of tracing under TCG and within the trace line's own noise
on hardware; a depth-4 user entry spends about 85 ms of its 520 in hops under
TCG and almost nothing on hardware. Sparing the task's half needs PCID, which
the lab refuses, or the kernel executing in the task's root, which is this
boundary. PCID itself lies within the boundary, and the lab host can run it:
WHPX exposes `pcid` and `invpcid` on the i9-12900K without complaint. It
cannot run under TCG at all: QEMU's `helper_write_crN` sends a long-mode CR3
write with the no-flush bit set down its reserved-bits path, and its TLB
flushes whole on every root switch regardless (`target/i386/tcg/system/misc_helper.c`,
`target/i386/helper.c`, current master). A PCID kernel path would therefore
run, and be validated, only under the hypervisor, never in the emulator the
suites and the F5 flow use, and its TLB coherence across user roots, which
TCG would hide, is the kind of error only hardware shows. Whether a kernel
path validated only under WHPX is acceptable is the owner's call beside the
boundary. The lab leaves the roots as they are until that is chosen.
