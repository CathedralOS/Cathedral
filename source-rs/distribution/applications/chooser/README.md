# User picker and app launcher

F5 opens a full-screen user picker. Existing users appear as a horizontal row
of avatar tiles; **NEW USER** copies the hidden template. Click a user to enter
its workspace. Root stops and collects this picker before loading the user's
compact bottom taskbar, a separate process running another instance of the chooser
with icon tiles for **Notes**, **Colors**, **Split** and **Nested**. The template and the launcher itself
are hidden. There are no normal status rows, storage toggles or launcher CLOSE
control; apps own their X buttons. A failed launch gives a short message such as
NO ROOM. Arrows and Enter work while the chooser has focus.

The user taskbar has **SETTINGS** at the bottom (or F2 while the taskbar has
focus). Its small editor changes bar height and outer spacing. **SAVE** writes
the user's `Matrix` object, then asks its matrix to apply geometry immediately.
**CANCEL** discards local edits and returns to app tiles. Running apps retain
their state and repaint their resized local views. SAVED; RESTART means the save
committed but live application was refused; retry Save or restart. The picker
has no settings editor. Budgets and launcher selection remain startup-only.

[settings/settings.rs](session/settings/settings.rs) owns the draft and save
decision; [persistence.rs](session/settings/persistence.rs) reads a consistent
revision with small inline replies and publishes with an expected revision.
A lost save acknowledgement stays uncertain until a later Save reads back the
desired bytes; an intervening different edit is refused as CHANGED. Cancel does
not undo a write that already committed. Draft/uncertain work reports loss over
the ordinary control link. The launcher uses its explicitly granted whole user
realm; Notes and Colors still hold only their own subtrees.

Saving then sends `APPLY_LAYOUT` on the launcher's interaction connection.
[Stock configuration](../../../distribution/matrix/service/children/configuration.rs)
grants this operation only to the selected launcher's endpoint incarnation;
the [world layout policy](../../../platform/libraries/matrix/world/layout.rs)
checks current epoch, authority and complete geometry before moving either view.
Ordinary apps, storage endpoints, stale launchers and unconfigured pickers are
refused. This grants no owner-control endpoint, key mask or resource authority.
Current views keep their key masks and surface limits; revoked sibling views
stay revoked. The standing placement for later app launches follows the new
layout. Save and apply are separate operations, so a failed application does
not roll back durable configuration. Geometry retries are idempotent.

This is selection, not authentication. The bounded listing accommodates the
hidden template plus three users; at the limit the add-user tile disappears.
There is deliberately no scrolling, pagination, account editor or logout yet.
User labels come from hierarchy names; NEW creates `User N`. This is not an
identity or account model. The picker resolves `Template` by path. The taskbar
resolves its own hidden launcher from the matrix's saved configuration. Neither
depends on listing position. An exact media-type index selects executable entries;
an `.exe` extension alone does not add a tile or grant execution authority.

Start at [session.rs](session/session.rs): load the realm, route input, dispatch
launch/copy and repaint. [realm.rs](session/realm.rs) lists through the granted
storage binding and copies the template with COPY_TREE. [layout.rs](session/layout.rs)
shares tile positions between hit testing and rendering. [view.rs](session/view.rs)
orders background, visible tiles and failure text; [tiles.rs](session/tiles.rs)
rasterizes small avatars and app icons. Rust mod.rs contains only plumbing.
Local input selection lives in [selection.rs](session/selection.rs); the session
routes input to either tiles or the settings editor before publishing the scene.

The chooser is an ordinary matrix child. LAUNCH names only an object reachable
through its binding; its owner supplies the launch policy, placement and service
offers. Removing the storage test toggle does not remove required/optional
service declarations or attenuation from the platform protocol.

[labels.rs](session/labels.rs) constructs the bounded default user name. Visible
labels use hierarchy metadata, not executable footers: one name read per tile,
since the executable filter already proved each tile's type. The host's explicit
[stock-program upgrade](../../migrations/README.md) refreshes recognized existing
user launchers, retaining their child data and customized programs.

Five surfaces cover the background, up to three tiles/panels, and the footer.
Each app tile uses 12 pages and each settings panel 10. The footer is released
before replacing images, keeping the peak within the existing 48-page grant.
The desktop proof exercises creation, selection, app launches, close decisions
and cold boot. The settings proof checks cancel, live layout, dirty Notes drafts,
post-resize input/close, later app placement, cold persistence, independent users
and lost-save reconciliation. Its render-race mode forces invalidation during
surface creation and bitmap publication, then checks document retention.
