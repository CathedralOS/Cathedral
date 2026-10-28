"""Reject incorrect pixels and unavailable reference execution."""
import json
import struct
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
import dev
from dev import verify_bitmap_pair, verify_overlay_background, verify_reference_execution


def bitmap(pixels, bits=32, width=2, height=1):
    header = bytearray(54)
    header[:2] = b"BM"
    struct.pack_into("<I", header, 2, 54 + len(pixels))
    struct.pack_into("<I", header, 10, 54)
    struct.pack_into("<IiiHH", header, 14, 40, width, height, 1, bits)
    return bytes(header) + pixels


class DevelopmentOracleTests(unittest.TestCase):
    def test_review_uses_the_shared_compiler_and_checked_in_project(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            compiler = root / 'omega'
            compiler.write_bytes(b'compiler')
            profile = {'schema': 'omega-compiler-build-profile-v1', 'opt_level': '3'}
            with patch('dev.ROOT', root), patch('dev.select_compiler', return_value=compiler) as select, \
                    patch('dev.run', return_value=SimpleNamespace(stdout=json.dumps(profile))) as run, \
                    patch('sys.argv', ['dev.py', 'review', '--project', 'headless_test']):
                dev.main()
            select.assert_called_once_with(None)
            self.assertEqual(dev.PROJECTS['headless_test'], dev.LIBRARY / 'tests/headless')
            run.assert_any_call([compiler, 'update'], cwd=dev.PROJECTS['headless_test'])
            self.assertTrue((root / 'build/omega-ui/compiler.json').is_file())

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
