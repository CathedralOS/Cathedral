"""Feature switches must not substitute a stale or subsequently rebuilt role."""
import os
import struct
from pathlib import Path
import tempfile
import unittest
import artifacts


def executable(payload, debug=b''):
    data = bytearray(256)
    data[:6] = b'\x7fELF\x02\x01'
    struct.pack_into('<Q', data, 0x20, 64)
    struct.pack_into('<HH', data, 0x36, 56, 1)
    struct.pack_into('<IIQQQQQQ', data, 64, 1, 5, 256, 0x400000, 0,
                     len(payload), len(payload) + 4096, 4096)
    struct.pack_into('<Q', data, 0x28, 256 + len(payload))
    struct.pack_into('<HHH', data, 0x3a, 64, 1, 0)
    return bytes(data) + payload + debug


class ArtifactTests(unittest.TestCase):
    def test_same_timestamp_different_features_change_bundle_input(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'same-package'
            source.write_bytes(executable(b'normal role'))
            os.utime(source, (1, 1))
            normal = artifacts.capture(source, root / 'captured')
            source.write_bytes(executable(b'recovery role'))
            os.utime(source, (1, 1))
            recovery = artifacts.capture(source, root / 'captured')
            self.assertNotEqual(normal, recovery)
            self.assertEqual(normal.read_bytes()[256:], b'normal role')
            self.assertEqual(recovery.read_bytes()[256:], b'recovery role')
            source.write_bytes(executable(b'normal role'))
            self.assertEqual(artifacts.capture(source, root / 'captured'), normal)

    def test_debug_growth_cannot_exhaust_boot_bundle_or_change_loaded_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'program'
            original = executable(b'loaded code', b'd' * (4 * 1024 * 1024))
            source.write_bytes(original)
            captured = artifacts.capture(source, root / 'captured')
            loaded = captured.read_bytes()
            self.assertEqual(source.read_bytes(), original)
            self.assertEqual(loaded[64:120], original[64:120])
            self.assertEqual(loaded[256:], b'loaded code')
            self.assertEqual(struct.unpack_from('<Q', loaded, 0x28), (0,))
            self.assertEqual(struct.unpack_from('<HHH', loaded, 0x3a), (0, 0, 0))
            source.write_bytes(executable(b'loaded code', b'different debug'))
            self.assertEqual(artifacts.capture(source, root / 'captured'), captured)


if __name__ == '__main__':
    unittest.main()
