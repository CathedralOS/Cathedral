# AGENTS.md

Avoid unnecessary secondary labels and descriptions. Let the layout do the explaining.

Keep layout, input, rendering and platform ownership explicit. Measure
responsiveness against the active monitor's refresh budget.
Keep published source, documentation and examples free of external UI reference branding.

The canonical library core is `src/`; its Windows adapter is `platform/windows/`.
The freestanding component-command backend is `platform/native/`, with its exact
independent-export consumers in `tests/native_component/`.
The canonical gallery is `examples/gallery/`.
Host tools live in Cathedral's `tools/omega-ui/`; run them from the repository root.
Build the runnable gallery only into `build/omega-ui/gallery/`. Do not retain
duplicate UI source trees, old gallery builds, or UI archives in this repository.
