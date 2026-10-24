import tempfile
from pathlib import Path
import unittest
from build import fingerprint


class FingerprintTests(unittest.TestCase):
    def test_compiler_and_source_changes_invalidate_but_nested_build_output_does_not(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            compiler = root / 'omega'
            compiler.write_bytes(b'compiler one')
            (root / 'src/build').mkdir(parents=True)
            source = root / 'src/main.omg'
            source.write_text('source one')
            first = fingerprint(root, compiler)
            (root / 'src/build/generated.omg').write_text('ignored build output')
            self.assertEqual(fingerprint(root, compiler), first)
            source.write_text('source two')
            self.assertNotEqual(fingerprint(root, compiler), first)
            second = fingerprint(root, compiler)
            compiler.write_bytes(b'compiler two')
            self.assertNotEqual(fingerprint(root, compiler), second)


if __name__ == '__main__':
    unittest.main()
