import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch
from build import fingerprint, prepare


class FingerprintTests(unittest.TestCase):
    def test_missing_source_is_reported_before_invoking_an_unrelated_installed_compiler(self):
        with tempfile.TemporaryDirectory() as directory:
            with patch('build.subprocess.check_output') as invoke:
                with self.assertRaisesRegex(ValueError, 'OMEGA_UI_REPO'):
                    prepare(Path(directory), Path(directory) / 'missing-compiler')
                invoke.assert_not_called()

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
