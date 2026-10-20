# Existing desktop upgrades

The stock [profile](../profile.json) explicitly selects `label_migration`.
The harness installs the current template, then calls the host storage lab's
[label upgrade](../../../tools/storage-lab/engines/volume/labels.rs). F5 and the
desktop test use this same path. This is a bounded distribution compatibility
step, not a generic package updater or a new filesystem naming model.

`readable-labels.txt` identifies verified old stock content by its exact
BLAKE3 content-layout manifest. It records the boot-test receipts those bytes
came from; it does not identify programs by title, ELF-looking bytes or object
position. Retained template revision events also identify stock copies. The
upgrader validates the content digest before any replacement.

In the original label-only mode, only direct program children of existing users
in the stock Users folder are considered. Recognized launcher copies receive the current template launcher.
Recognized Notes/Colors receive a display-title footer while retaining their
original executable bytes. The user matrices, object identities, tree edges,
saved notes/colors, other roots and customized/unrecognized programs remain
unchanged. Thus old app features remain old; this upgrade fixes their labels.

The host locks the image against concurrent use, plans the changes, saves a
`.before-readable-labels.raw` backup, then publishes new heads/events/chunks and
the quota ledger in one durable transaction. History is preserved. Quota,
corruption or backup failures refuse publication. Repeating the operation is
idempotent; the install sidecar also records the migration fingerprint so an
unchanged F5 launch does not reopen the database for it. The backup is taken
after current template installation and before changing copied user programs.

Unknown custom launchers may keep their own UI, including numeric labels. Do
not extend the allowlist from unidentified user bytes. A future general choice
between pinned copied programs and managed updates still needs design; this
explicit migration does not settle it.

The profile also opts into `app_close_migration`. This subsequent upgrade
refreshes recognized stock Notes/Colors as well as the launcher so existing
users receive app-owned X buttons and Notes' save/discard/cancel interaction.
It additionally recognizes the previous migration's title-only copies by
hashing the original bytes without that exact footer and matching the legacy
allowlist or retained template history. A matching title alone never authorizes
replacement. Saved objects and customized programs still remain untouched.
Its separate backup suffix is `.before-app-close.raw`, and its opt-in is part
of the installer receipt. It now also refreshes recognized stock user matrix
executables so old users can implement the new startup request. Exact retained
template history or explicitly verified legacy digests are required; custom
matrix programs are never identified merely by their location or title.

After program upgrades, [hierarchy migration](../../../tools/storage-lab/engines/volume/hierarchy.rs)
adds names/type indexes and a `Matrix` configuration to the stock template and
recognized users. It preserves object numbers, saved Notes/Colors, existing
nonempty metadata and existing configurations. Changes commit in one transaction
after a `.before-hierarchy.raw` backup; a repeat is a no-op. The install sidecar
records hierarchy version 1. Unknown custom programs remain untouched and may
need explicit metadata before they appear in the executable-only chooser.
Initial configuration values come from [matrix.conf](../matrix.conf), encoded
by the host migration. Changing those defaults does not reset an existing
template or user's saved `Matrix` object.

The lab uses an 8 MiB volume and a 2 MiB retained-record quota. F5 backs up
and grows legacy 4 MiB images offline before installation. Physical extent and
logical quota exhaustion remain separate refusals; neither reformats the image.

Verification:

```powershell
# From tools/storage-lab:
cargo test --locked --features engines labels
# From repository root, with a retained pre-label desktop test fixture:
python tools/boot-harness-rs/labels_upgrade_test.py PATH/guest.raw
python tools/boot-harness-rs/desktop_test.py
```

Host tests check exact stock matching, a custom launcher with a misleading
title, collected history, corrupt content, unchanged records outside the
upgrade, quota, backup, fixed disk extent and repeatability. The compatibility
replay opens only a copy of the old fixture, enters its existing user, checks
the actual labels, reads both saved notes, and closes/relaunches Notes through
its X button inside the old user matrix. The ordinary desktop proof still
covers fresh installs, storage attenuation and cold reboot.

The same explicit stock upgrade now refreshes recognized copied launchers to
user/app tiles. The interactive disk also has a one-time
`.before-user-tiles.raw` backup. Compatibility replay defaults to selecting the
existing user and checking saved Notes. `--fill-users` additionally creates users
up to the bounded tile limit; see TASKS_RUST.md for the older-disk storage timeout
found by that extra stress, separate from the launcher migration itself.

## Named realm programs

`startup.realm_programs` adds **Split** and **Split/Notes** by local path to the
template and recognized existing stock users. Unlike the original six bootstrap
objects, these additions allocate fresh object numbers. One hierarchy
transaction covers names, code and budget changes, after a separate
`.before-realm-programs.raw` backup.

The installation sidecar records each addition's content manifest. A later update
replaces only that exact previous manifest or already-current bytes. An existing
custom entry blocks its path and descendants; a name alone never authorizes
replacement. Losing the sidecar therefore leaves unrecognized older additions
unchanged. The unchanged-install fast path excludes manifest receipts from the
input fingerprint and does not open the disk.

The migration raises only recognized old stock resource defaults: launcher pages
48 to 64 and app pages/readers/surfaces `(32,4,6)` to `(80,64,9)`. It preserves the
launcher path, key policy and custom resource choices. Version 1 vertical
geometry decodes as compact bottom-bar geometry; version 2 records bar height. The
outer desktop now funds a matrix-capable app slot with six channels and two child
slots. These are bounded distribution defaults, not executable privileges or a
new persistent configuration format.

Host tests cover existing-user data/IDs, custom layout, exact program replacement,
name collisions and idempotence. `labels_upgrade_test.py` verifies a retained
six-program desktop copy through the actual F5 installer and user boot.

## Complete F5 installation

F5 now calls [install.rs](../../../tools/storage-lab/engines/volume/install.rs)
with all programs and migration inputs together. It keeps the saved disk locked,
saves a content-named `.before-compact-<digest>.raw` backup, and runs the stages
above on a private host scratch image. Their individual backups are scratch-only;
the durable recovery point is the full original disk beside the saved image.

[compact.rs](../../../tools/storage-lab/engines/volume/compact.rs) rebuilds and
cold-compares every prepared record, refusing a result that exceeds the original
extent. Only then is it published and verified again. Compaction itself does
not resize; the explicit legacy growth step precedes it.
A failed preparation leaves the source bytes and installation receipt unchanged.
Publication errors attempt rollback; process/power loss during the offline copy
requires restoring the synced backup. This is not an atomic disk-image replacement.

Named additions now use the same head/pin reachability collector as stock-copy
refresh when their staged records exceed the logical quota. Live/pinned excess
still refuses the entire installation. This resolves the case where enough old
revisions remained for Split's addition to cross the quota after the earlier
stock-copy stage had already finished below it.
