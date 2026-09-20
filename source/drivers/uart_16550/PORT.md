# UART 16550 audit and pure plans (UART-000)

## Scope and status

**Tested pure audit, 2026-09-20.** Existing facts remain owned by
`../facts/uart_16550.omg`; 65 missing constants and a module declaration are
additive. Existing names and values are unchanged. This package translates
pure configuration, bit manipulation, arithmetic, constructor geometry and
register request plans. It deliberately provides no live UART driver/backend.

Source inventory covers all nine `src/` Rust files and `tests/api.rs` at the pin.
Live polling, instructions, volatile access, Rust trait plumbing and hardware
tests are individually classified omissions with explicit boundary semantics.
They are not invented Omega blockers. No production boot/core import root is
changed; the existing boot path retains explicit `PortIo` reach and poll budgets.

Files: `pure.omg`, `baud.omg`, `errors.omg`, `enum_ordinals.omg`, `plans.omg`;
`BOUNDARIES.md` records every authority-bearing operation family. Package root
`build.omg` depends only on existing hardware facts. Developer fixtures live in
repository-root `tools/ports/uart_16550/`. Validation used the fresh Omega build recorded below.

## Upstream pin and licensing

- [rust-osdev/uart_16550](https://github.com/rust-osdev/uart_16550), exact commit
  `653455f17e92c67a67e44b4c4b6eaba22411e57b`.
- Checkout: `reference_code/rust-osdev/uart_16550`; absent/wrong checkout fails.
- Preserved `MIT OR Apache-2.0`, with modified derivative files identified.
- Original notices/license texts:
  [licenses/rust-osdev/uart_16550](../../../licenses/rust-osdev/uart_16550/),
  [THIRD_PARTY_NOTICES.md](../../../THIRD_PARTY_NOTICES.md).
- Existing primary-fact source ownership stays intact. Added upstream register
  organization, helpers, plans and adapted tests retain derivative provenance.

## Source and public-symbol map

[inventory.json](inventory.json):10 files,321 lexical anchors;238 translated,
83 explicitly omitted,zero pending/blocked. `--require-transcribed` passes.
It is a lexical audit, not a Rust parser; all89 plain `Divisor` variants are
additionally covered by `enum_ordinals.omg` and the compiled upstream probe.

| Upstream | Destination / treatment |
| --- | --- |
| `spec.rs` register values and offsets | Existing `../facts/uart_16550.omg`, with independent value checks |
| `spec.rs` field setters/getters, interrupt decoding, baud math | `pure.omg`, translated spec tests |
| `spec.rs::Divisor` | `enum_ordinals.omg`, all89 ordinal identities, explicitly not physical divisor values |
| `config.rs` | `pure.omg::Config/default_config/baud_compare`, `baud.omg` preserving named/Custom variants |
| `error.rs`, spec/TTY error payloads | `errors.omg`; Rust formatting/Error source chaining omitted |
| `lib.rs` constructors, setup and read/write decisions | Pure predicates, plans and `BOUNDARIES.md`; no ambient UART object |
| `backend/*.rs` | Constructor/register geometry and explicit PortIo/MMIO boundaries; instructions/volatile pointer wrappers omitted |
| `tty.rs` | `tty_byte` byte transformations; live initialization/loopback wrapper omitted |
| `embedded_io.rs` | Polling repetition and no-op flush audited in `BOUNDARIES.md`; Rust adapter omitted |
| Source tests/API fixture | All pure spec cases and constructor cases translated; volatile dummy test adapted to divisor/stride plan expectations; Rust Send/public-driver identity tests omitted |

## Primary specifications and vectors

Sources consulted:

- [Semiconductor Design Solutions UART16550 IP datasheet v1.0, 2000-12-15](https://caro.su/msx/ocm_de1/16550.pdf),
  the exact datasheet cited by upstream: register map, interrupt decoding,
  prescaler and DMA-end extensions.
- [TI TL16C550D/DI SLLS597E, revised December2008](https://www.ti.com/lit/ds/symlink/tl16c550d.pdf),
  accessible-register summary, FIFO/LCR/LSR and divisor behavior.

The upstream IP-core's DMA-end interrupt bits and PSD register are extensions,
not universal16550 features. They are marked accordingly in the existing fact
file and are never enabled by generic initialization. MCR bit5 is reserved in
the cited upstream IP but autoflow control in the TI device; retained upstream
reserved naming is not a universal assertion about derivatives.

[vectors.json](vectors.json), `cathedral-port-vectors-v1`, has168 independently
compiled upstream values:79 register/global constants and89 enum ordinals.
The x64/little-endian profile describes the host audit; these values do not
assert Omega storage geometry. Register access remains byte-wide; numeric port
bases/offsets and request records are not placed MMIO layouts. No binary ABI
for the internal helper/config/error records is claimed.

`tools/ports/uart_16550/measure.py` checks source pin, executes the locked local
Rust value probe, compares every mapped fact/ordinal and checks the committed
vectors. Existing combined constants are also evaluated by the original UART
fact canary. No hardware is accessed by either check.

## Pure behavior, tests and deviations

The semantic fixture evaluates baud/divisor/frequency examples and noninteger
results; all field setter round trips, unrelated-bit preservation and stale-bit
regressions; all interrupt encodings/priorities; parity/FIFO/word decoding;
LSR error bits; constructor edges; default config; baud variant conversions;
ordered initialization/read requests; divisor restoration; CTS/loopback/send
capacity; and TTY bytes. Upstream's dummy-MMIO test becomes checked divisor3
and stride1/4 request expectations, without claiming a volatile-memory emulator.

`check.py` runs actual Omega bodies from a constant initializer and proves the
result equals zero. Its negative control changes expected baud115200 to115201
**inside the test body**, so the same evaluated result must become1 and be
rejected. It does not merely alter the final assertion. Separately, all45
upstream Rust library tests pass on the host.

Deliberate API differences:

- Registers/bitfields and configuration selections use raw-byte encodings;
  getters mask only their own bits. Closed Rust enum identity is not a foreign
  register representation. Baud named/Custom identity is retained separately;
  config stores its numeric value and ordering uses that numeric value.
- Optional config values use explicit presence/enabled fields. Config defaults
  remain9600,8N1,FIFO14,interrupts disabled,CTS checking disabled.
- Arithmetic returns `Calculation { value, error }` for zero input, invalid
  prescaler, overflow, noninteger result or out-of-u16 divisor. Upstream uses
  debug assertions/panics for some of these. Valid-input results match the pin;
  original input payload types remain in `errors.omg` for adapters. Masks used
  after validation make representable narrowing explicit; they do not silently
  turn invalid input into a valid calculation.
- MMIO constructors preserve acceptance of power-of-two strides64/128, although
  the pin's later checked-u8 offset can reject some registers. Per-operation
  geometry reports this separately. Numeric zero is rejected explicitly.
- Init planning accepts an already validated divisor. Upstream can return after
  setting DLAB on invalid baud; planning validates first. A prescaler participates
  in arithmetic but neither the pin nor this plan writes PSD during init.
- `try_send_byte` preserves the pin's CTS-error collapse into NoCapacity.
- Pure request/response plans expose wait/retry and scratch mismatch. Live loops
  need a separate bounded executor; no claim of eventual device readiness is made.

## Authority, integration and blockers

Owning [drivers charter](../CHARTER.md): facts reach nothing; behavior here
produces values and requests. [BOUNDARIES.md](BOUNDARIES.md) records PortIo/MMIO
range/custody, read side effects, DLAB exclusion, access width, initialization
ordering, FIFO loss, polling, completion and loopback restoration obligations.
No address constructs an authority-bearing value, and no Rust `unsafe` is
translated into unchecked dereference.

No Omega language blocker was required for the claimed pure slice. Engineering
proof obligations were handled with explicit bounds and ordinary helper locals.
Actual PortIo/MMIO executors and device integration are intentionally outside
this audit; native execution, simulator/hardware and emitted foreign layouts
are not tested. Existing boot logic and its policy remain unchanged.

The old UART fact canary now uses current `Source::Path`, an isolated source
copy (escaping source symlinks reject), exact imports and a semantic assertion
with a mutated-COM1 negative control. This replaces its obsolete numbered JSON
artifact assumptions; current `--check` does not emit those files. Its original
16 fact expectations remain the same.

## Verification commands and results

Exact freshly built Omega checkout: `eaa7993a23623cd8fabf45350340479c5c9c7879`.
Compiler: `/tmp/cathedral-omega-eaa7993/release/omega`, SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.
Run from Cathedral root:

| Command | Result |
| --- | --- |
| `python3 tools/ports/inventory.py check --checkout reference_code/rust-osdev/uart_16550 --require-transcribed source/drivers/uart_16550/inventory.json` | PASS:238 translated,83 deliberate omissions,0 pending/blocked |
| `python3 tools/ports/uart_16550/measure.py` | PASS:168 pinned Rust values agree with source and vectors |
| `cargo test --quiet --manifest-path reference_code/rust-osdev/uart_16550/Cargo.toml --lib` | PASS:45 host Rust tests; no live port/device IO |
| `python3 tools/ports/uart_16550/check.py --omega /tmp/cathedral-omega-eaa7993/release/omega` | PASS:16 source files; pure semantic tests and body-mutating negative control |
| `OMEGA_BIN=/tmp/cathedral-omega-eaa7993/release/omega tools/uart-16550-facts-canary/run.sh` | PASS:existing16 facts evaluated; mutatedCOM1 rejected |
| Native/hardware/Omega foreign layout | Not run; outside pure audit claim |
