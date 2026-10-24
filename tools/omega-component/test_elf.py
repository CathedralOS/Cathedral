"""Small malformed-container controls for the static native packaging boundary."""
import struct
import unittest
from elf import component


def fixture(extra_tag=1, extra_flags=2, undefined=False):
    data = bytearray(0x2200)
    data[:8] = b'\x7fELF\x02\x01\x01\0'
    struct.pack_into('<HHIQQQIHHHHHH', data, 16, 3, 62, 1, 0, 64, 0x2000, 0, 64, 56, 2, 64, 6, 1)
    struct.pack_into('<IIQQQQQQ', data, 64, 1, 5, 0x1000, 0x1000, 0, 16, 16, 4096)
    struct.pack_into('<IIQQQQQQ', data, 120, 1, 4, 0x1800, 0x1800, 0, 8, 8, 16)
    names = b'\0.shstrtab\0.text\0.data\0.dynstr\0.dynsym\0'
    data[256:256 + len(names)] = names
    data[320:326] = b'\0test\0'
    struct.pack_into('<IBBHQQ', data, 360, 1, 0x12, 0, 0 if undefined else 2, 0x1000, 16)
    def section(index, name, tag, flags, address, offset, size, link=0, alignment=1, entry=0):
        struct.pack_into('<IIQQQQIIQQ', data, 0x2000 + index * 64,
                         names.index(name), tag, flags, address, offset, size, link, 0, alignment, entry)
    section(1, b'.shstrtab', 3, 0, 0, 256, len(names))
    section(2, b'.text', 1, 6, 0x1000, 0x1000, 16, alignment=16)
    section(3, b'.data', extra_tag, extra_flags, 0x1800, 0x1800, 8, alignment=8)
    section(4, b'.dynstr', 3, 0, 0, 320, 6)
    section(5, b'.dynsym', 11, 0, 0, 360, 24, link=4, alignment=8, entry=24)
    data[0x1000:0x1010] = b'checked text!!!!'
    data[0x1800:0x1808] = b'constant'
    return data


class ComponentTests(unittest.TestCase):
    def test_internal_relative_spacing_and_export_are_retained(self):
        body, export = component(fixture())
        self.assertEqual(export, ('test', 0))
        self.assertEqual(body[:16], b'checked text!!!!')
        self.assertEqual(body[0x800:], b'constant')
        self.assertEqual(body[16:0x800], bytes(0x800 - 16))

    def test_unsupported_runtime_state_and_resolution_are_refused(self):
        for data in (fixture(extra_tag=8), fixture(extra_tag=4), fixture(extra_flags=0x402), fixture(undefined=True)):
            with self.subTest(), self.assertRaises(ValueError):
                component(data)

    def test_malformed_extents_and_execution_permissions_are_refused(self):
        for offset, form, value in ((68, '<I', 7), (96, '<Q', 1 << 63), (112, '<Q', 3),
                                    (136, '<Q', 0x1000), (0x2000 + 2 * 64 + 24, '<Q', 0x1001)):
            data = fixture()
            struct.pack_into(form, data, offset, value)
            with self.subTest(offset=offset), self.assertRaises(ValueError):
                component(data)
        with self.assertRaises(ValueError):
            component(fixture()[:63])


if __name__ == '__main__':
    unittest.main()
