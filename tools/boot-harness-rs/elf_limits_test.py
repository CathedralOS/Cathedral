"""Fail before guest launch when a compiler output exceeds kernel image backing."""
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch
import elf_limits


def image(*segments):
    data = bytearray(64 + len(segments) * 56)
    data[:6] = b'\x7fELF\x02\x01'
    struct.pack_into('<Q', data, 32, 64)
    struct.pack_into('<HH', data, 54, 56, len(segments))
    for index, (kind, memory) in enumerate(segments):
        # No file data: these tests deliberately charge zero-filled memory too.
        struct.pack_into('<IIQQQQQQ', data, 64 + index * 56, kind, 4, 0, 0, 0, 0, memory, 4096)
    return data


class ImageLimitTests(unittest.TestCase):
    def test_separate_rounding_and_bss(self):
        self.assertEqual(elf_limits.load_pages(image((1, 4097), (1, 1), (4, 999999))), 3)

    def test_limit_matches_kernel_source_and_rejects_the_extra_page(self):
        limit = elf_limits.image_limit()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'recovery-provider.elf'
            path.write_bytes(image((1, (limit - 1) * 4096), (1, 4096)))
            with patch('builtins.print'):
                self.assertEqual(elf_limits.check(path), limit)
            path.write_bytes(image((1, (limit - 1) * 4096), (1, 4097)))
            with self.assertRaisesRegex(ValueError, rf'recovery-provider.elf.*{limit + 1}.*{limit}'):
                elf_limits.check(path)

    def test_bad_or_truncated_header_never_reports_success(self):
        valid = image((1, 4096))
        for data in (b'', valid[:63], valid[:-1], image((4, 4096))):
            with self.assertRaises(ValueError): elf_limits.load_pages(data)
        valid[5] = 2
        with self.assertRaises(ValueError): elf_limits.load_pages(valid)


if __name__ == '__main__':
    unittest.main()
