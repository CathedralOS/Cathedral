"""Independent scanout oracle for the two-client retained compositor experiment."""
from pathlib import Path
import capture


def verify(stream, output, serial):
    for marker in (
        "scopes nested clips atomic snapshots stale and foreign offers passed",
        "movement overlap and exposure redrawn from retained content",
        "client fault spin flood and replacement preserved peer progress",
        "provider replacement reconnected both surviving clients",
    ):
        if "Cathedral compositor: " + marker not in serial:
            raise RuntimeError(f"Missing compositor evidence: {marker}")
    capture.command(stream, "screendump", {"filename": str(output / "display.ppm")})
    with (output / "display.ppm").open("rb") as source:
        if source.readline() != b"P6\n":
            raise RuntimeError("Unexpected scanout encoding")
        width, height = map(int, source.readline().split())
        if source.readline() != b"255\n" or (width, height) != (1024, 768):
            raise RuntimeError("Expected 1024x768 RGB scanout")
        pixels = source.read()
    expected = bytearray(bytes.fromhex("101827") * width * height)
    def rect(x, y, w, h, rgb):
        for row in range(y, y + h):
            start = (row * width + x) * 3
            expected[start:start + w * 3] = bytes.fromhex(rgb) * w
    font = (Path(__file__).resolve().parents[2] / "source-rs/platform/libraries/bitmap-font/font.hex").read_text().splitlines()
    for x, color, title in ((80, "e96f6f", "APP ONE"), (400, "79c99e", "APP TWO")):
        rect(x, 160, 256, 200, color)
        # Independent intersection of the two nested groups: 64 by 56, not 96 by 96.
        rect(x + 48, 216, 64, 56, "779bea")
        rect(x + 56, 224, 16, 16, "59d9cc")
        for index, char in enumerate(title):
            for row, bits in enumerate(bytes.fromhex(font[ord(char) - 32])):
                for column in range(5):
                    if bits & (1 << (4 - column)):
                        rect(x + 16 + index * 6 + column, 172 + row, 1, 1, "ffffff")
    if pixels != expected:
        raise RuntimeError(f"Compositor scanout mismatch: {output / 'display.ppm'}")
    capture.save_png(output, width, height, pixels)
    print(f"PASS: compositor isolation/recovery markers and all {width * height} scanout pixels; {output / 'display.png'}")
