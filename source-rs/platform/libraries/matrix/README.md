# Matrix mechanisms

Start at [lib.rs](lib.rs), then [world/world.rs](world/world.rs): accept a parent
grant, place local children, invalidate revoked views, and dispatch local requests.
The owner chooses the child count through `World<N>`. No slot has a launcher,
application, login or desktop role.

- [Input](world/input.rs) confines focus, capture, coordinates and keys to grants.
  Pointer hit testing follows the owner's surface ranges; equal ranges use slot
  order for input-only children.
- [Surfaces](world/surface/surface.rs) conserves allowances, assigns disjoint rank
  ranges (explicit owner placement or free-space allocation), and clips/translates
  retained drawing. Changing one child's allowance
  does not move another child's live rank range. Fragmentation can refuse an
  allocation; live ranges are never silently compacted.
- [Layout](world/layout.rs) accepts owner-computed rectangles and an explicitly
  delegated controller. It validates the whole arrangement before applying it;
  there is no strip width, margin, launcher path or configuration schema here.
- [Storage scopes](world/storage/storage.rs), [management](world/management.rs)
  and [launch observations](world/launches.rs) use independently selected child
  capacities. They work without display state.

This is a library, not a compulsory initial process or a desktop server. Another
distribution can compose these mechanisms with its own processes and protocols,
or implement the public service interfaces itself. The existing
[stock matrix executable](../../../distribution/matrix/main.rs) remains the
F5 consumer and owns its two-slot layout, configuration, control router and
recovery policy. Shared protocols stay in [platform contracts](../../contracts/matrix.rs).

Host tests cover a three-child vertical arrangement with the last child as the
authorized controller, stable surface ranges through sibling allowance changes,
and four-child headless scope/management ledgers. Those are mechanism proofs,
not claims of a runnable phone or embedded distribution. The fixed surface and
input limits in the experimental protocol remain explicit resource bounds.
