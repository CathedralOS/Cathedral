# Omega UI for Cathedral applications

`cathedral-omega-ui` is the reusable userspace library in
`source-rs/distribution/libraries/omega-ui`. Its complete Omega implementation,
native backend, fonts, examples and tests are checked in at
`source/distribution/libraries/omega-ui`. Rust apps describe their own component
trees; compiled Omega code handles control behavior, editing, motion, layout and
pixels. The demo's entire model and view now live in
`source-rs/distribution/applications/omega-ui/view.rs`.

## Build

Add `cathedral-omega-ui.workspace = true` to an app's Cargo dependencies. Its build
script prepares the same native exports automatically; the boot harness contains
no special preparation rule for the demo. The build uses Cathedral's checked-in distribution source. Its standard library
is an ordinary immutable Git package dependency, acquired through Omega's normal
package review; the UI does not keep a private standard-library snapshot.
The canonical UI source uses explicit machine arguments and Build-owned native
bindings. Its compiler requirements and current qualification are recorded in
[the handoff](../../TASKS_RUST.md#canonical-omega-ui-source).
The earlier `20dcd0409b66` producer and desktop runs below are historical evidence
for the previous source revision; that compiler cannot compile this migrated UI.
An arbitrary later compiler revision is not a substitute for a qualified producer.
Python 3 is required by the existing Cathedral build tools. No font runtime or new
external Cargo dependency is needed. `OMEGA_UI_COMPONENT_DIR` explicitly selects
already-prepared artifacts instead.
Review and compilation use the same [bounded compiler process](../omega-component/process.py)
as application components: 4 GiB and five minutes per invocation. A failed or
exhausted compiler never publishes a new component receipt.

The Rust F5 profile includes the UI demo. Its existing prepared-component input
keeps that consumer independent of the compiler used for other desktop packages.
Keep an existing legacy compiler checkout intact. For the canonical UI compiler,
run these commands from the Cathedral root. Cargo
uses the Rust toolchain pinned in the compiler repository:

```text
git clone https://github.com/CathedralOS/Omega.git build/omega-ui/canonical-compiler
cd build/omega-ui/canonical-compiler
git checkout b51ac370b6cecc9d2abd0948d86b01fccfc943eb
cargo build --locked -p omega
```

Omega's ordinary Cargo profile is optimized at level 1; the `debug` output
directory does not mean an unoptimized compiler. Return to Cathedral and prepare
the UI with the selected compiler:

```text
python tools/omega-ui/build.py --compiler build/omega-ui/canonical-compiler/target/debug/omega
```

On Windows the executable is `omega.exe`. Package review may first request
ordinary acceptance; inspect the named proposal and resume, then repeat the
build. Preserve that component directory and its receipt together.

Other desktop packages still contain legacy Omega source. Until they migrate,
keep their existing `20dcd0409b66` compiler selection and set
`OMEGA_UI_COMPONENT_DIR` to the absolute `build/omega-ui/component` directory
prepared above. This is the library's existing explicit artifact input; it
selects no compiler behind the caller's back. The ABI 2 Rust callers stay the
same. A cold F5 run using the new artifacts still needs Windows/QEMU validation;
the earlier desktop evidence does not establish it.

For current UI source development, pass the canonical compiler explicitly or
set `OMEGA_BIN` to it. The build selects an explicit `--compiler`,
then `OMEGA_BIN`, then `build/omega-ui/compiler/target/release/omega.exe` (Windows)
or `omega` (macOS/Linux), then `omega` on PATH. A selected compiler's failure is
reported rather than silently trying another one. For a compiler built elsewhere,
set `OMEGA_BIN` to its absolute executable path. The local compiler checkout is
ignored build tooling; the UI library itself remains tracked distribution source.
Generated package locks and component images stay in ignored build output;
all UI source and font inputs are in this repository.

The build records source/compiler/export hashes in
`build/omega-ui/component/receipt.json` and registers those inputs with Cargo.
Changes in the library, compiler, packaging or selected input paths rebuild the
consumer. Manual preparation remains available as `python tools/omega-ui/build.py`.
The pure-image validator refuses imports, unresolved symbols, runtime relocations,
TLS and implicit mutable storage. Components are statically linked into the app's
normal ELF; this is not dynamic component admission or a proof verdict for the OS.

## Source and development tools

The Cathedral distribution is the single source of truth for Omega UI. Keep
changes in `source/distribution/libraries/omega-ui` and the matching Rust package;
do not recreate a standalone UI repository. Host tooling lives here in
`tools/omega-ui`, and host artifacts stay under `build/omega-ui`. Per-package
review state and build caches remain ignored beside their source manifests.

```text
python tools/omega-ui/dev.py review --project headless_test
python tools/omega-ui/dev.py test
python tools/omega-ui/dev.py review --project gallery
python tools/omega-ui/dev.py gallery
python tools/omega-ui/dev.py snapshots
```

These commands use the same compiler selection as the component builder. Initial
review may acquire the pinned Git dependency; `--offline` restricts review to
already acquired sources. Review the actual pending rows and resume through
Omega's normal acceptance workflow before running the program.

Omega operations are ordinary module declarations with explicit runtime
arguments, such as `omega_ui::context::setup(&mut context, width, height)`.
Import public data by its exact path (`use omega_ui::context::UiContext;`);
nested machines scope code and receive their own arguments.
`gallery` and `snapshots` require Windows; the executable and captures are in
`build/omega-ui/gallery`. Regenerate the font atlas only when changing fonts with
`python tools/omega-ui/bake_native_font.py` (Pillow required for regeneration).

## App lifecycle

1. Acquire a scoped interaction binding through the normal startup services.
2. Call `native_api().layout()`, allocate exclusive state and 16 KiB packet backing,
   and construct `Ui`. Buffers can be disjoint slices of one private allocation.
   Call `resize` with the app's available logical extent.
3. Retain your app model, including a separate `TextInput` per field. Use `Ui::build`
   to describe the whole view. A changed model causes another pass with consumed
   input edges; update models from returned `Response` values.
4. Stage each local pointer/key edge, then build before staging the next edge.
   Advance `tick(elapsed_ms)` and build while `stats().animating` is true. The host
   owns scheduling; widgets never poll global devices.
5. Read paint bounds from the completed build's `Node` values and rows through
   `widgets::row`, or create `Presenter` for the supplied view and borrow the app's
   `Services` owner for presentation. The
   app selects regions covering its painted content to fit its granted image
   backing. Publish once after a bounded input batch, rather than per input edge.

The crate's `lib.rs` contains a typechecked lifecycle example. Available controls
are buttons (four variants), toggles, checkboxes, sliders, progress and editable
text fields, with labels/headings, rows, columns, cards, spacers and dividers.
`Frame::theme` selects shared theme tokens. Interactive control IDs must be positive and unique within the frame. No app actions or IDs are reserved by the backend.

`Node` values belong to one instance and one build pass; stale or foreign nodes
are refused. Field values carry independent text, caret, selection and scroll
state. The shared editor supports selection, Ctrl+A, Shift navigation, Home/End,
Delete and Backspace when those events are supplied. Malformed packets, invalid
frame phases, duplicate IDs, overflow and unfinished nesting are refused. A failed
view invalidates its scene; a subsequent valid build recovers the instance.

`Presenter` owns its surface names for one supplied view epoch, but no raw endpoint
pair or second transport client. `present` and `retire` borrow the same mutable
`Services` owner used by input polling and acquisition. The demo selects shared
transport before its first acquisition; image grants retain their immutable
custody while application packets use shared backing. Its manifest funds one
notification, private mailbox page and accepted reader page. It creates and
retires region surfaces as the app's composition changes, committing images and
removals atomically. It refuses insufficient backing before beginning a batch;
later staging failures abort the batch. Call `Presenter::retire` to release its surfaces on a live binding. A retired
view releases its surfaces through the parent. The app continues to own startup, input translation, loss
handling, model persistence and process lifetime. The library imports only common
platform/kernel contracts and runtime facilities, with no distribution dependency.

## Demo and validation

```text
python tools/boot-harness-rs/run.py --release --window --fresh
```

F5 runs this same composition. Create/select a user and choose the fifth launcher
icon, Omega UI. Theme changes appearance, Reset restores the model, Clear empties
the field, and X closes the app. The volume switch removes/restores the slider to
demonstrate changing composition through the same public API. Text is transient.
Trace-input is opt-in; ordinary builds do not log typed text.

```text
cd source-rs
cargo test --locked -p cathedral-omega-ui -p cathedral-omega-ui-demo
cd ..
python -m unittest discover -s tools/omega-ui -p "test_*.py"
python -m unittest discover -s tools/omega-component -p "test_*.py"
python tools/omega-ui/test_guest.py
```

Windows tests RX-map the actual compiled exports and execute different app views,
independent instances, multiple editors, click/drag/motion, invalid frames and
recovery. The guest test uses disposable disks and real QMP input on fresh/cold
boots, including region removal/restoration, editing and close/reopen.

## Bounds

The core has 128 nodes including the root, eight nesting slots including the root,
8 KiB of frame text, 256 bytes per label and 128 bytes per editable field. The
viewport is at most 3840 by 2160. Current rendering is CPU-based x86-64 System V
with grayscale Inter glyphs. Physical-key transport in this demo does not yet
supply modifiers/composed text; generic hosts may supply modifier flags through
the public API. Unicode/IME, clipboard integration, ARM64 and monitor-refresh
performance are not established.

Reusable app code exceeded the former 512 KiB program-object ceiling. Bounded
objects now permit 256 pages (1 MiB). Per-task private/reader caps remain
1024/4096, ordinary app private grants remain 128 pages, root quota remains 2 MiB,
and image permissions/admission checks remain enforced. Every object still needs
actual funding. Matrix default physical reservation covers process load plus its
private images; explicit physical requests retain their amounts. Storage splits
its funded private pages between cache and outstanding replies, instead of
starving its cache to reserve the maximum protocol reply. Larger objects/apps may
need explicitly larger owner-selected grants. The image-frame ledger tracks all
funded translation tables; the caller's admission ceiling and physical frame
source remain the allocation gates.

On macOS the kernel address-space ceiling preserves the launcher’s measured
existing VM reservation, then allows the declared allocation budget above it.
Large shared-image/allocator reservations are not resident compiler memory.
The ceiling remains kernel-enforced and inherited by child processes; this is
an address-space allowance, not an aggregate resident-set guarantee. An inherited
stricter hard ceiling is retained.
