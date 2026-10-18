"""Feature switches must not substitute a stale or subsequently rebuilt role."""
import os
from pathlib import Path
import tempfile
import unittest
import artifacts


class ArtifactTests(unittest.TestCase):
    def test_same_timestamp_different_features_change_bundle_input(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'same-package'
            source.write_bytes(b'normal role')
            os.utime(source, (1, 1))
            normal = artifacts.capture(source, root / 'captured')
            source.write_bytes(b'recovery role')
            os.utime(source, (1, 1))
            recovery = artifacts.capture(source, root / 'captured')
            self.assertNotEqual(normal, recovery)
            self.assertEqual(normal.read_bytes(), b'normal role')
            self.assertEqual(recovery.read_bytes(), b'recovery role')
            source.write_bytes(b'normal role')
            self.assertEqual(artifacts.capture(source, root / 'captured'), normal)


if __name__ == '__main__':
    unittest.main()
