"""Reject incorrect pixels and unavailable reference execution."""
import struct
import tempfile
import unittest
from pathlib import Path
from tools.dev import verify_bitmap_pair, verify_overlay_background, verify_reference_execution


def bitmap(pixels, bits=32, width=2, height=1):
    header = bytearray(54)
    header[:2] = b"BM"
    struct.pack_into("<I", header, 2, 54 + len(pixels))
    struct.pack_into("<I", header, 10, 54)
    struct.pack_into("<IiiHH", header, 14, 40, width, height, 1, bits)
    return bytes(header) + pixels


class DevelopmentOracleTests(unittest.TestCase):
    def test_reference_decline_cannot_pass_as_dual_execution(self):
        verify_reference_execution("PASS: library\nnative exit 0\ninterp exit: 0\n")
        for output in ("native exit 0\ninterp: DECLINED (fuel allowance)\n", "native exit 0\n"):
            with self.subTest(output=output), self.assertRaisesRegex(ValueError, "Terminal reference"):
                verify_reference_execution(output)

    def test_visible_pixel_change_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            direct, reference = Path(temporary) / "direct.bmp", Path(temporary) / "full.bmp"
            direct.write_bytes(bitmap(bytes([1, 2, 3, 0, 4, 5, 6, 0])))
            reference.write_bytes(bitmap(bytes([1, 2, 3, 255, 4, 5, 6, 255])))
            verify_bitmap_pair(direct, reference, (2, 1))
            reference.write_bytes(bitmap(bytes([1, 2, 3, 0, 4, 9, 6, 0])))
            with self.assertRaisesRegex(ValueError, "repaint mismatch"):
                verify_bitmap_pair(direct, reference, (2, 1))

    def test_overlay_preserves_pixels_outside_its_bounds(self):
        with tempfile.TemporaryDirectory() as temporary:
            background, overlay = Path(temporary) / "page.bmp", Path(temporary) / "menu.bmp"
            pixels = bytes(36)
            changed = bytearray(pixels)
            changed[16] = 200
            background.write_bytes(bitmap(pixels, width=3, height=3))
            overlay.write_bytes(bitmap(changed, width=3, height=3))
            verify_overlay_background(background, overlay, (1, 1, 2, 2))
            changed[0] = 100
            overlay.write_bytes(bitmap(changed, width=3, height=3))
            with self.assertRaisesRegex(ValueError, "background outside"):
                verify_overlay_background(background, overlay, (1, 1, 2, 2))

    def test_wrong_pixel_format_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            image = Path(temporary) / "wrong.bmp"
            image.write_bytes(bitmap(bytes(8), bits=24))
            with self.assertRaisesRegex(ValueError, "32-bit BI_RGB"):
                verify_bitmap_pair(image, image, (2, 1))


if __name__ == "__main__":
    unittest.main()
