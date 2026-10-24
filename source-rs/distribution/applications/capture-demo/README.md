# Supplied capture viewer

The installed Split matrix's CAPTURE control admits this ordinary process.
[main.rs](main.rs) declares interaction and capture, then
[preview.rs](preview.rs) acquires its supplied endpoints, requests one image,
waits for completion and presents the sealed result in its local rectangle.
It receives no source identity, storage or input-production service, and needs
no private pixel allocation. Its owner chooses source, crop, mask and placement.

Follow [the wire](../../../platform/contracts/capture.rs) and
[client](../../../platform/libraries/user-runtime/capture.rs) to Split's
[provider entrance](../split/session/capture/capture.rs). The provider, rather
than this viewer, must enforce redaction before sharing any image.

`capture-probe` inspects every masked pixel, RGBX/page padding and single-result
delivery through the actual received reader. `capture-recipient-exit` exits with
an outstanding request; `capture-timeout` expects an explicit expiry from a
disposable unresponsive source. Those features are absent from ordinary F5.
The desktop harness's `--capture` and `--capture-case` workflows own these checks.
