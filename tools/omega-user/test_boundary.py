"""Keep the native adapter matched to the kernel and reject invalid process images."""
import re
import struct
import unittest

from build import ABI, ROOT, validate


def image(*, entry=0x8000000000, flags=5, address=0x8000000000, kind=1):
    data = bytearray(8192)
    data[:8] = b'\x7fELF\x02\x01\x01\0'
    struct.pack_into('<HHIQQQIHHHHHH', data, 16, 2, 62, 1, entry, 64, 0, 0,
                     64, 56, 1, 0, 0, 0)
    struct.pack_into('<IIQQQQQQ', data, 64, kind, flags, 4096, address, address, 4096, 4096, 4096)
    return data


class BoundaryTests(unittest.TestCase):
    def test_admitted_entry_and_permissions(self):
        validate(image())
        for data in (image(entry=0), image(flags=7), image(flags=6),
                     image(address=0x8000001000), image(kind=2), image()[:100]):
            with self.subTest(header=data[:128]):
                with self.assertRaises(ValueError):
                    validate(data)

    def test_native_calls_match_the_kernel_abi(self):
        contract = (ROOT / 'source-rs/kernel/contracts/user.rs').read_text(encoding='utf-8')
        numbers = dict(re.findall(r'pub const (\w+): u64 = (\d+);', contract))
        assembly = (ABI / 'entry.S').read_text(encoding='utf-8')
        calls = re.findall(r'(?:mov eax, (\d+)|xor eax, eax)\s*// (\w+)\n', assembly)
        self.assertEqual(len(calls), 48)  # source sites, including the shared-access macro
        for number, name in calls:
            self.assertEqual(int(number or '0'), int(numbers[name]), name)

    def test_table_order_matches_the_omega_layout(self):
        declaration = (ABI.parents[1] / 'kernel.omg').read_text(encoding='utf-8')
        fields = re.findall(r'^    (\w+): addr;', declaration, re.MULTILINE)
        assembly = (ABI / 'entry.S').read_text(encoding='utf-8')
        table = re.findall(r'^    \.quad cathedral_(\w+)$', assembly, re.MULTILINE)
        self.assertEqual(['yield' if field == 'yield_now' else field for field in fields], table)
        self.assertEqual(len(table), 40)


if __name__ == '__main__':
    unittest.main()
