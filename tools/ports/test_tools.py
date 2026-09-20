"""Failure-path tests for provenance and coverage; these do not test Omega."""
import copy
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
import inventory
import vectors
import rust_layout


class LlvmBytesTests(unittest.TestCase):
    def test_hex_and_backslash_escapes(self):
        self.assertEqual(rust_layout.decode_bytes(r'A\00\\\FF'), b'A\x00\\\xff')


class InventoryTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.checkout = self.root / 'upstream'
        self.checkout.mkdir()
        for arguments in [('init', '-q'), ('config', 'user.email', 'fixture@example.invalid'), ('config', 'user.name', 'Fixture')]:
            subprocess.run(['git', '-C', str(self.checkout), *arguments], check=True)
        self.source = self.checkout / 'lib.rs'
        self.source.write_text('pub struct Header {\n pub size: u32,\n}\nimpl Header {\n pub const FLAG: u32 = 1;\n pub const fn size(&self) -> u32 { self.size }\n}\nnewtype_enum! {\npub enum Code: u32 => {\n SUCCESS = 0,\n}}\n')
        subprocess.run(['git', '-C', str(self.checkout), 'add', '.'], check=True)
        subprocess.run(['git', '-C', str(self.checkout), 'commit', '-qm', 'fixture'], check=True)
        self.revision = inventory.git(self.checkout, 'rev-parse', 'HEAD').strip()
        self.manifest = inventory.snapshot(self.checkout, self.revision, ['lib.rs'], 'https://example.invalid/upstream')

    def check(self):
        return inventory.check(self.manifest, self.checkout, self.root)

    def test_snapshot_covers_fields_operations_and_macro_values(self):
        self.assertEqual(self.check()['symbols']['pending'], 6)
        self.assertIn('6:size', self.manifest['files']['lib.rs']['symbols'])
        with self.assertRaisesRegex(ValueError, 'not transcribed'):
            inventory.check(self.manifest, self.checkout, self.root, True)

    def test_raw_const_pointer_is_not_a_declaration(self):
        self.assertEqual(inventory.rust_anchors('    value: *const u8,\n'), {})
        self.assertEqual(inventory.rust_anchors('    value: *const ffi::c_void,\n'), {})

    def test_raw_identifier_field_is_inventoried(self):
        self.assertEqual(inventory.rust_anchors('pub r#type: u8,'),
                         {'1:r#type': 'pub r#type: u8,'})

    def test_lifetimes_and_static_modifiers(self):
        anchors = inventory.rust_anchors("pub const NAME: &'static str = \"x\";\nstatic mut NEXT: u64 = 0;\nstatic ref VALUE: u64 = 1;")
        self.assertEqual(list(anchors), ['1:NAME', '2:NEXT', '3:VALUE'])

    def test_match_arm_is_not_enum_value(self):
        self.assertEqual(inventory.rust_anchors('None => 0,\nVALUE == 0'), {})

    def test_missing_checkout_fails(self):
        with self.assertRaisesRegex(ValueError, 'checkout absent'):
            inventory.check(self.manifest, self.root / 'absent')

    def test_missing_symbol_fails(self):
        del self.manifest['files']['lib.rs']['symbols']['2:size']
        with self.assertRaisesRegex(ValueError, 'symbol inventory differs'):
            self.check()

    def test_dirty_upstream_fails(self):
        self.source.write_text(self.source.read_text() + 'pub const EXTRA: u32 = 2;\n')
        with self.assertRaisesRegex(ValueError, 'working source differs'):
            self.check()

    def test_wrong_revision_fails(self):
        self.manifest['upstream']['revision'] = '0' * 40
        with self.assertRaisesRegex(ValueError, 'HEAD differs'):
            self.check()

    def test_missing_file_fails(self):
        self.manifest['files'].clear()
        with self.assertRaisesRegex(ValueError, 'source file inventory differs'):
            self.check()

    def test_missing_disposition_reason_fails(self):
        self.manifest['files']['lib.rs'].update(disposition='omitted', reason='')
        with self.assertRaisesRegex(ValueError, 'needs a reason'):
            self.check()

    def test_deleted_target_fails(self):
        self.manifest['files']['lib.rs'].update(disposition='translated', targets=[{'path': 'missing.omg', 'anchor': 'data Header'}])
        with self.assertRaisesRegex(ValueError, 'target missing'):
            self.check()

    def test_changed_target_anchor_fails(self):
        (self.root / 'header.omg').write_text('pub data Header { size: u32; }\n')
        entry = self.manifest['files']['lib.rs']
        entry.update(disposition='translated', targets=[{'path': 'header.omg', 'anchor': 'data Header'}])
        self.check()
        (self.root / 'header.omg').write_text('pub data Other { size: u32; }\n')
        with self.assertRaisesRegex(ValueError, 'anchor missing'):
            self.check()

    def test_duplicate_json_key_fails(self):
        path = self.root / 'bad.json'
        path.write_text('{"format": 1, "format": 2}')
        with self.assertRaisesRegex(ValueError, 'duplicate JSON key'):
            inventory.read_json(path)


class VectorTests(unittest.TestCase):
    def setUp(self):
        self.expected = {'format': 'cathedral-port-vectors-v1', 'target': {'abi': 'uefi-x64', 'pointer_bits': 64, 'endian': 'little'}, 'provenance': {'kind': 'test-fixture', 'description': 'synthetic test only', 'sources': ['test_tools.py']}, 'measurements': {'Header.size': {'kind': 'size', 'value': 24}, 'Header.alignment': {'kind': 'alignment', 'value': 8}}}
        self.observed = copy.deepcopy(self.expected)
        self.observed['provenance'].update(kind='omega-inspection', compiler_revision='0' * 40, artifact_sha256='0' * 64)

    def test_exact_match(self):
        self.assertEqual(vectors.compare(self.expected, self.observed), 2)

    def test_expected_is_not_observation(self):
        with self.assertRaisesRegex(ValueError, 'Omega observation'):
            vectors.compare(self.expected, self.expected)

    def test_wrong_layout_fails(self):
        self.observed['measurements']['Header.size']['value'] = 32
        with self.assertRaisesRegex(ValueError, 'values differ'):
            vectors.compare(self.expected, self.observed)

    def test_missing_measurement_fails(self):
        del self.observed['measurements']['Header.size']
        with self.assertRaisesRegex(ValueError, 'coverage differs'):
            vectors.compare(self.expected, self.observed)

    def test_wrong_target_fails(self):
        self.observed['target']['pointer_bits'] = 32
        with self.assertRaisesRegex(ValueError, 'ABI mismatch'):
            vectors.compare(self.expected, self.observed)

    def test_alignment_and_types(self):
        for value in (0, 3, -8, True, 8.0):
            with self.subTest(value=value), self.assertRaises(ValueError):
                self.expected['measurements']['Header.alignment']['value'] = value
                vectors.validate(self.expected)


if __name__ == '__main__':
    unittest.main()
