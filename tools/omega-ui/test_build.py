import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch
from build import LIBRARY, ROOT, fingerprint, prepare


class FingerprintTests(unittest.TestCase):
    def test_missing_source_is_reported_before_invoking_an_unrelated_installed_compiler(self):
        with tempfile.TemporaryDirectory() as directory:
            with patch('build.LIBRARY', Path(directory)), patch('build.subprocess.check_output') as invoke:
                with self.assertRaisesRegex(ValueError, 'Cathedral checkout is missing Omega UI source'):
                    prepare(Path(directory) / 'missing-compiler')
                invoke.assert_not_called()

    def test_default_library_and_all_package_dependencies_are_inside_cathedral(self):
        import re
        self.assertEqual(LIBRARY, ROOT / 'source/distribution/libraries/omega-ui')
        for source in ('src/main.omg', 'platform/native/main.omg', 'platform/native/font_atlas.omg',
                       'tests/native_component/layout/main.omg', 'tests/native_component/frame/main.omg'):
            self.assertTrue((LIBRARY / source).is_file(), source)
        for manifest in LIBRARY.rglob('build.omg'):
            if 'build' in manifest.relative_to(LIBRARY).parts[:-1]:
                continue
            for relative in re.findall(r'\blocation:\s*"([^"]+)"', manifest.read_text(encoding='utf-8')):
                dependency = (manifest.parent / relative).resolve()
                self.assertTrue(dependency.is_relative_to(LIBRARY) and dependency.is_dir(), str(manifest))

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
