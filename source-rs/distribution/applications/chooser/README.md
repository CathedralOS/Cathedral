# Chooser

The program that lists a realm and launches from it. Press **F5**: the first
chooser is shown the folder of users, and a user's chrome is the same program
shown that user's realm.

It is an ordinary matrix child with nothing but its binding and its interaction
connection. On start and after every action it lists its realm with `LIST` and
shows up to four entries by object number. Clicking an entry, or Enter on it,
sends `LAUNCH` with that number: the host reads the object through this
chooser's binding, so only what the chooser can see can run, and starts it as
the chooser's sibling in the slot its owner opened. The status row shows the
host's answer: RUNNING, BUSY while the slot is taken, NOT OPEN when the owner
gave the slot no policy, NEEDS SERVICE for a missing required import, BAD VERSION
for incompatible required imports, and REFUSED otherwise.

In the user picker, **NEW** copies the first entry whole with `COPY_TREE`, making
a user from the template. FULL means the root's quota refused the copy.
In the user's launcher, the same row is **STORAGE ON/OFF** for subsequent launches.
OFF withholds storage from the owner's offers. Notes permits this and receives
no storage endpoint; Colors requires storage and refuses. Toggling does not
revoke storage from an already-running app. The chooser cannot widen the owner's
offer. Names/kinds remain separate owner questions; entries are still numbers.

**CLOSE** sends `CLOSE`: the host asks the running sibling with `LOSS` whether
stopping would lose work and stops it only on an answer of nothing. KEPT means
it had work or did not answer; NOTHING OPEN means the slot was empty.

The chooser answers `HEALTH` and `LOSS` on its control link with nothing to
lose. Entry argument 1 labels it CHOOSE USER; otherwise it labels itself PROGRAMS.
It draws one bitmap of eight rows: the heading, entries, NEW (or STORAGE ON/OFF), CLOSE and the
status, and is funded for two of them so a replacement can be drawn beside the
one on screen. [session.rs](session/session.rs) is the loop,
[realm.rs](session/realm.rs) the listing and copy, [view.rs](session/view.rs)
the bitmap.
