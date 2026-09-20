# UEFI table producer/consumer fixtures — UEFI-010

Status: **tested** for the pure fixed byte-image model; **blocked** for native
producer/consumer ABI compatibility. Independently authored Cathedral fixtures;
no Rust implementation or firmware image is copied. Numeric geometry comes from
the pinned UEFI table corpus documented in
[tables.PORT.md](../../contracts/uefi/raw/tables.PORT.md). Its upstream pin,
UEFI specification references and cross-target Rust measurements remain the
authority for those geometry expectations.

## Implemented and checked

`table_images.omg` produces one initialized 376-byte little-endian boot-services
image. It includes the 24-byte header, all 44 slot positions, a zero reserved
slot, and two inert mock identifiers at offsets 240 and 248. The header includes
a fixed-image CRC32. Python independently rebuilds the full image with `struct`
and `zlib`, using the pinned table vectors for geometry. The committed binary and
all 376 Omega byte expectations must match that reconstruction.

The Omega consumer checks the fixed header and decodes bounded slot indices.
`mock_header_matches` compares this fixture's header fields; it is not a general
CRC validator or firmware admission routine. The full-image test supplies the
separate check that all body bytes match. Slot numbers are bounded by a contract.
Raw integer tokens never become pointers or authority.

`mock_stall` returns success through one million microseconds and invalid
parameter above that artificial limit. `consume_mock_stall` requires the
expected fixture header and token before returning that mock result; it returns
unsupported otherwise. The pure monotonic mock returns the next integer or
out-of-resources at u64 maximum. These are explicit test policies, not claims
about firmware timing, counter persistence or the full UEFI service semantics.

`tools/ports/uefi-images/main.omg` checks every produced byte, selected first,
reserved, populated and last slots, stall boundaries, a malformed header, and
counter success/overflow. Its negative control changes one byte expectation
inside the actual comparison body and requires the unchanged final assertion
to reject result 1. These are semantic-evaluator tests, not native execution.

Verified compiler: clean Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`,
SHA-256 `2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`.

## Remaining language/design boundaries

- `PORT-BLOCKED[omega:layout-reflection-capacity]`: a complete native
  `BootServices` plan has 45 fields; the current source layout carrier admits
  32. The existing reproducible table-layout failures and fresh validation are
  recorded in [CONFORMANCE.md](../../contracts/uefi/raw/CONFORMANCE.md).
- `PORT-BLOCKED[omega:callback-field-materialization]`: native table slots need
  private callback field destinations. Omega's [native realization implementation
  note](../../../../Omega/omega-rust/omega/compiler/native-realization/README.md)
  explicitly limits the bounded route to one direct callback parameter and says
  field destinations and multiple callbacks need further work. The
  [private-callback contract](../../../../Omega/wiki/spec/build/private_callbacks.md)
  requires exact named callback selection, placement and lifetime/custody. Integer
  fixture IDs do not implement that seam. Calling policy and lifetime contracts
  still require ordinary authored work; their absence alone is not a blocker.
- An incidental evaluator failure is retained at
  `tools/ports/uefi-images/blocked/slice_length_probe.omg`: the literal
  `"123456789"` reports length zero during this constant evaluation. Fixed-array
  fixtures avoid that path, so it does not block the completed byte model.

The task remains unchecked because matching explicitly encoded byte images
does not demonstrate that independently emitted native producer and consumer
layouts agree. That final compatibility test awaits the native layout/callback
mechanisms above. No production import root or ExitBootServices flow changes.

## Reproduction

```sh
python3 tools/ports/uefi-images/check.py --omega /path/to/omega
```

Host-only checks and the exact negative-control command are documented in the
[fixture README](../../../tools/ports/uefi-images/README.md). Changes to any slot,
header field or token require regenerating the binary and byte oracle from the
independent host reconstruction, updating the fixed CRC, and rerunning both
positive and behavior-mutating controls.
