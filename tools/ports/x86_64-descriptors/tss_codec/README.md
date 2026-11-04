# Task-state bytes

This executable checks Cathedral’s actual task-state encoder and decoder. The [byte converters](../../../../source/platform/drivers/facts/x86_descriptor_bytes.omg) live beside the pure hardware definitions, so the kernel can use them without depending on a platform implementation. It imports the typed `TaskStateSegment` and the existing byte codecs, preserving the three privilege-stack and seven interrupt-stack entries. Distinct values exercise every byte of the 104-byte hardware representation.

The oracle adapts `../cases.omg::bytes_TaskStateSegment`: the privilege-stack words differ from every interrupt-stack word, every encoded byte must match its fixed expected value, and decoding the independent golden bytes must reproduce every typed field. The `corrupt_byte` twin changes byte 60 from 32 to 33 without changing the oracle; both engines must trap. Neither program installs a TSS or claims that the packed typed-field layout works.

From the repository root, using a compiler and its matching bundled library:

```sh
omega update --project tools/ports/x86_64-descriptors/tss_codec --target macos_arm64
# Review any requested decisions, then use omega update --resume for that project.
omega run --both tools/ports/x86_64-descriptors/tss_codec/main.omg
omega update --project tools/ports/x86_64-descriptors/tss_codec/corrupt_byte --target macos_arm64
omega run --both tools/ports/x86_64-descriptors/tss_codec/corrupt_byte/main.omg
```

Expected: the original exits 0 in both engines; the corrupted twin traps in both. The Build files bind all four hosted targets; only targets with recorded execution are qualified.
