# Omega UI

This is Cathedral's canonical, checked-in component library. The Rust app API is
in `source-rs/distribution/libraries/omega-ui`; see the
[Cathedral build and usage guide](../../../../tools/omega-ui/README.md).
The commands below run from this directory and also support the Windows gallery.

A native, immediate-mode UI library written in Omega. The package provides a
bounded box tree, two-pass layout, input handling, controls, text storage, themes,
docking, and software drawing. The Windows adapter supplies font measurement,
text editing, and cached bitmap painting; applications retain their windows.

The Windows gallery demonstrates controls, tasks, a virtual list, waveform
animation, typography, themes, docking, and a command palette. The headless
application checks the reusable library without opening a window.

The canonical library is `src/`, and the gallery source is `examples/gallery/`.
Run the built Windows UI from `build/gallery/omega-program.exe`. Older UI source
copies and gallery builds are not retained.

## Build and test

Install Python 3 and the Rust toolchain pinned by your Omega compiler checkout.
Set `OMEGA_REPO` to that checkout, or pass `--omega-repo <path>` to the helper.
The helper uses `mbx` when available and Cargo otherwise.

This project targets the development compiler. `build/compiler.json` records the
exact producer, local repairs, optimization profile and executable digest used
for validation. Compatibility with a released compiler has not been established.

```powershell
$env:OMEGA_REPO = '../Omega'
python tools/dev.py review --project headless_test
python tools/dev.py test
python tools/dev.py review --project gallery
python tools/dev.py gallery
python tools/dev.py snapshots
```

On macOS/Linux, set `OMEGA_REPO` with `export` and use the same Python commands
for the headless suite. The gallery and its capture host require Windows.

Each command builds an optimized compiler and records its revision, build
profile, working-tree status, and binary digest in `build/compiler.json`.
`--compiler-target-dir <path>` or `OMEGA_COMPILER_TARGET_DIR` reuses a compiler
build directory. `--compiler-profile dev` selects the optimized development
profile instead of the default release profile.

The `review` action refreshes package inputs. If it reports pending decisions,
inspect its generated review and source diff, edit only the pending decisions,
then run `python tools/dev.py resume --project <headless_test|gallery>`.
Generated `omega.lock` files bind local source paths and stay out of Git.

`test` runs the headless program with `omega run --both`, comparing Terminal Psi
and native execution. `gallery` produces `build/gallery/omega-program.exe` and
deploys its font assets. `snapshots` runs that image's `OMEGA_HEADLESS` route,
checking input, keyboard focus, redraw invalidation, task layouts, and filtering
before writing captures to `build/gallery/`. The native route also alternates
1440-by-945 and 3840-by-2160 slider frames between partial bitmap updates and full
row uploads, reporting `FRAME_US` records. The helper checks 48 RGB capture pairs and launcher background occlusion,
including partial/full repainting, checkbox animation, prepared/uncached dot
rendering, and command-menu filtering and selection. It rejects mismatched pixels. Dot timing records distinguish full
repaints (`DOT_FRAME_US`) from partial hover and animation updates (`DOT_HOVER_US`).
Command-menu edit frames report `SEARCH_FRAME_US`. Panel drag records
(`PANEL_FRAME_US`) cover both splitter directions, preview movement, and release.
Continuous records (`STREAM_FRAME_US`, `PANEL_STREAM_FRAME_US`) time consecutive
input updates without captures or forced reference repaints between them.
`PANEL_CACHE` and `PANEL_STREAM_CACHE` record corner-cache reads, misses, and row reuse.
The refresh-rate target and experiment records are in [docs/performance.md](docs/performance.md).
These offscreen timings exclude compositor pacing and do not measure display FPS
or input-to-photon latency.

The gallery tabs retain a moving selection and support Left/Right, Home, and End
when focused. Its launcher has an independent 255-unit UTF-16 query with selection,
word navigation, clipboard shortcuts, undo/redo, and case-insensitive filtering.
The Windows editing engine owns query semantics; Omega paints the visible control.

The Windows host adapter has a native edit, bitmap-span and clipping regression: from
`tests/native_host`, run `omega update --offline`, review its proposal if needed,
then `omega run main.omg`. The library suite stays portable and uses `--both`.

## Using the package

Add a `Build::depend` path to `src/` in your application's `build.omg`.
Import the modules you use, then retain one `UiContext` across frames:

1. Advance `context.motion` once with `UiMotion::advance` and elapsed milliseconds.
   Supply input and call `UiFrame::begin`.
2. Add containers and controls with stable, nonzero IDs. Close containers with
   `Ui::end`; update your application model from the returned interactions.
3. Supply actual font measurements with `UiContext::set_text_metrics` before
   layout. `WindowsUiFonts::measure` does this with the same fonts used to paint.
   Call `UiFrame::next_pass`; when it returns true, build and measure again.
   Input edges are consumed once, the frame counter advances once, and four
   passes bound a view that keeps changing. Inspect `context.frame_error`.
4. Keep scheduling while `context.motion.animating` is true; set `motion.reduced`
   to respect a reduced-motion preference. Prepare `UiScene` once after layout;
   its band cursor paints surfaces and identifies foreground nodes in order.
   `Raster::render` provides a 64-by-64 convenience path. Physical scanline hosts
   supply an explicit `UiViewport` through `Raster::configure`.
5. Rebuild after input or animation changes. The next frame retires old text spans.

The parsed lifecycle example in `src/main.omg` and executable examples show
the actual API. Input uses the preceding frame's hit rectangles; a new view must
be laid out before a pointer can hit its controls.

This revision changes the development API. Replace integer layout options with
`UiAxis`, `UiSizing`, `UiAlignment`, and `UiJustify`; pass `UiContentKind` to
`add_node`. The former glyph-only `Ui::button` is `Ui::icon_button`; `Ui::button`
now takes a label and `ButtonOptions`. `Ui::text` returns its node index for
measurement. Physical coordinate helpers take the logical extent explicitly.
Application-specific content uses a named `UiContentKind::Extension` tag rather
than sharing the built-in control tags. `ThemePalette` is the single theme source.

`WindowsUiFonts` borrows the host's fonts and clipped device context in transparent
text mode. The host retains and releases those resources. `WindowsTextField`
borrows its hidden edit handle; query filtering remains application code.

## Source layout

```text
src/                       omega_ui package and public modules
    frame.omg              input consumption and bounded rebuild lifecycle
    scene.omg              prepared surfaces and shared paint cursor
    paint_boxes.omg        ordered box commands for a physical viewport
    viewport.omg           logical-to-physical coordinate contract
    list.omg               viewport-derived uniform-list ranges
    text_field.omg         selection and caret geometry
platform/windows/          omega_ui_windows adapter package
    fonts.omg              native intrinsic measurement and text painting
    text_field.omg         Unicode editing, selection and pointer navigation
    box_painter.omg        cached scanline and direct bitmap backend
    corner_cache.omg       exact rounded-corner samples
    fill_cache.omg         native opaque span copies
examples/gallery/          Windows gallery and host
    main.omg               application model, entrypoint, native host
    view.omg               demonstration views
    painting.omg           demonstration drawing
    damage.omg             cached repaint tracking
    dock_damage.omg        old/new dock surface invalidation
    dock_rows.omg          example-specific scanline reuse boundaries
    text_clip.omg          one native text pass through the dirty union
    dots.omg               prepared waveform columns
    search.omg             command titles and filtering over a library editor
tests/
    headless/              executable library checks
    native_component/      independent freestanding layout/frame consumers
    native_host/           Windows adapter checks
    tools/                 Python development-tool checks
support/standard_library/  retained platform source
assets/fonts/inter/        gallery fonts and their notice
tools/dev.py               build, review, test, and capture commands
build/                     ignored compiler output and validation records
```

`src/main.omg` explains the frame lifecycle; `context.omg` owns layout and input,
`widgets.omg` the controls, `types.omg` and `palette.omg` their geometry and color
roles, `motion.omg` elapsed transitions, `text.omg` and `utf16.omg` text storage,
`dock.omg` docking geometry, and `raster.omg` and `tester.omg` drawing and input
simulation. The package name and `use omega_ui::...` imports remain unchanged.

The standard-library snapshot is source, not generated output. The gallery and
headless suite use it through the same relative dependency, independently of the
compiler checkout. `--project headless_test` selects `tests/headless/`.

The [freestanding first surface](platform/native/README.md) uses the same core
inside Cathedral, with a button, slider and bounded editable text field. Its
host adapter receives scoped input and publishes software images. The Windows
gallery remains the canonical full component showcase.

Run the Python tool checks with:

```sh
python -m unittest discover -s tests/tools
```

## Current limits

The context has 128 nodes, an eight-level container stack, and 8192 UTF-8 label
bytes, plus 64 retained motion channels. Channels unseen for 120 frames can be
reused. Docking has twelve slots. Drawing uses a 4096-pixel buffer. Allocation
failure returns -1 and a frame error; it cannot alias the root or unbalance
containers. These bounds are explicit; this is not an unbounded document UI.

The number field supports bounded stepping, keyboard navigation, and focus. Direct
numeric text editing and native accessibility exposure remain unfinished. The
gallery scales a fixed logical canvas; captures do not establish live interaction
or DPI correctness on every display. Linux and macOS window hosts are not included.

## Cathedral and other freestanding hosts

The [native component backend](platform/native/README.md) exposes caller-authored
frames with the same widgets, layout, editing, motion and scalar raster. It owns
no demo tree or app model. Cathedral's `cathedral-omega-ui` crate supplies typed
frame construction and scoped image publication; its demo is an ordinary client.
Apps compose views without adding providers or changing this library's exports.
