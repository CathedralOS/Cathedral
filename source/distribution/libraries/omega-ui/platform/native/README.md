# Freestanding component backend

ABI 2 exposes caller-authored frames over the same `UiContext`, `UiFrame`, widgets,
scene, editor and raster used by the library. Applications own models and stable
IDs. There is no built-in view, model, control ID or application action here.

`UiNativeLayout::query` returns state bytes (0), alignment (1) and ABI version (2).
`UiNativeFrame::apply` receives opaque zero-initialized state and a separate
`[i32; 4096]` packet. The caller retains exclusive access to both buffers. Packet
operations and positions are documented at the top of `main.omg`; commands are
implemented in `commands.omg`, editing in `editor.omg`, state in `native_state.omg`.

The lifecycle is resize, stage one input edge, begin, build containers/controls,
finish pass, then rasterize. A requested rebuild repeats the complete view with
input edges consumed once. Four passes bound an unstable model. Begin invalidates
the previous scene; a rejected or aborted build cannot publish a partial tree.
Interactive IDs must be positive and unique in each frame. Layout, text, nesting
and node capacity errors are returned rather than silently overflowing storage.

Controls include labeled primary/secondary/ghost/danger buttons, toggles,
checkboxes, sliders, progress and editable fields. Containers include columns,
rows and cards, plus spacers/dividers. Text and field values are copied into the
frame. Each caller-owned field value carries its own bounded editor state; fields
have independent caret, selection and horizontal scrolling, without a global
editor or fixed field slots.

Text uses grayscale coverage baked from bundled Inter fonts. Measurement, pointer
caret placement and painting use the same advances. Regenerate the checked-in
atlas from the Cathedral root with `python tools/omega-ui/bake_native_font.py`
using Pillow; ordinary builds need
no font runtime. Font digests/settings are in `font_atlas.json`.

The limits are explicit: 128 nodes including the root, eight nesting slots
including the root, 8 KiB of frame text, 256 bytes per label, 128 bytes per editable
field, a 3840-by-2160 maximum viewport, and 16/24-pixel font rasters. Text input is
ASCII. The backend uses x86-64 System V. Unicode composition, clipboard services,
shaped text, ARM64 and monitor-refresh performance remain outside this backend.

The independent consumers in `tests/native_component/{layout,frame}` each select
one exact export requirement. Cathedral's reusable Rust crate prepares these
components automatically for every consumer, checks their pure images, and links
the original relative code/data layout into a normal application ELF. Its
`Frame` API describes arbitrary views; `Presenter` publishes app-selected regions
through a supplied scoped interaction binding. See Cathedral's
`tools/omega-ui/README.md` and the parsed example in its library `lib.rs`.

An optimized compiler containing the pure ELF and zero-frame contribution repairs
(`16a35ba7f429`) is required. Packaging keeps source/compiler/export hashes in the
host's ignored build receipt. Normal OS image admission and resource funding still
apply; static packaging supplies neither a dynamic loader nor an OS proof verdict.
