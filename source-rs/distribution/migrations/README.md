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
of the installer receipt. This does not update the user's matrix executable.

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
