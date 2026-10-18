# User picker and app launcher

F5 opens a full-screen user picker. Existing users appear as a horizontal row
of avatar tiles; **NEW USER** copies the hidden template. Click a user to enter
its workspace. The same chooser program becomes the user's left-hand app strip,
with icon tiles for **NOTES** and **COLORS**. The template and the launcher itself
are hidden. There are no normal status rows, storage toggles or launcher CLOSE
control; apps own their X buttons. A failed launch gives a short message such as
CLOSE APP FIRST. Arrows and Enter work while the chooser has focus.

This is selection, not authentication. The bounded listing accommodates the
hidden template plus three users; at the limit the add-user tile disappears.
There is deliberately no scrolling, pagination, account editor or logout yet.
User labels are ordinals, not identities. Editable names and object kinds remain
owner questions. Hiding the first entry follows the current stock layout and
must change when explicit object roles replace that convention.

Start at [session.rs](session/session.rs): load the realm, route input, dispatch
launch/copy and repaint. [realm.rs](session/realm.rs) lists through the granted
storage binding and copies the template with COPY_TREE. [layout.rs](session/layout.rs)
shares tile positions between hit testing and rendering. [view.rs](session/view.rs)
orders background, visible tiles and failure text; [tiles.rs](session/tiles.rs)
rasterizes small avatars and app icons. Rust mod.rs contains only plumbing.

The chooser is an ordinary matrix child. LAUNCH names only an object reachable
through its binding; its owner supplies the launch policy, placement and service
offers. Removing the storage test toggle does not remove required/optional
service declarations or attenuation from the platform protocol.

[labels.rs](session/labels.rs) reads the program's 32-byte display-title footer
through the chooser's binding. It grants no authority. The host's explicit
[stock-program upgrade](../../migrations/README.md) refreshes recognized existing
user launchers, retaining their child data and customized programs.

Five surfaces cover the background, up to three tiles, and exceptional text.
Each 128x96 tile uses 12 pages; the 48-page allowance covers three tiles and one
replacement. The desktop proof exercises creation, selection, app launches,
close decisions and cold-boot persistence through the normal F5 composition.
