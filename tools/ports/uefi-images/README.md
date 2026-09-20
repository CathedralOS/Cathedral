# Inert UEFI producer/consumer fixtures

Status: **tested byte-image model; native compatibility leg blocked**. See the
[port record](../../../source/libraries/uefi/table_images.PORT.md).

```sh
python3 tools/ports/uefi-images/check.py --omega /path/to/omega
python3 tools/ports/uefi-images/check.py --host-only
```

The host check reconstructs a 376-byte boot-services image from the independently
measured pinned Rust offsets, little-endian integer encoding, and Python's CRC32.
It compares `boot-services.bin` and all 376 expectations in `golden.omg`.
The Omega fixture invokes the actual producer, compares every byte, consumes
selected slots, rejects a changed header, and exercises two pure service mocks.
The negative control changes one expected byte inside the comparison body;
the unchanged final assertion must reject the computed failure result.
`--negative-only` reruns that control and host audit after a separately recorded
positive check. Large constant byte comparisons can take several minutes.

The binary is independently authored synthetic data. Its two nonzero slot values
are fixture identifiers, not callable addresses. Its checksum is valid only for
this exact fixed image. No physical mapping, firmware service, native callback,
or production integration is performed.

`blocked/slice_length_probe.omg` retains an incidental evaluator reproducer:
the fresh compiler reports `0 == 9` for a quoted byte view's length. The image
fixture uses fixed arrays and does not depend on this blocked operation.
