# Example platform support

`standard_library/` is the retained Omega standard-library source used by both
examples. It includes native text, drawing, bitmap capture, and pointer bindings
that the gallery needs. The gallery uses a writable DIB section, flushes GDI
before CPU writes, and copies only validated scanline extents. Its host also reads
the performance counter and animation preference, waits for input while idle,
and synchronizes live presents with the compositor. Keeping this source separate from generated `build/`
output lets a checkout reproduce those bindings.

The compiler is selected through Cathedral's `tools/omega-ui/dev.py`. Refresh package
inputs after editing this snapshot; do not replace it with whichever library
happens to accompany a compiler binary.
