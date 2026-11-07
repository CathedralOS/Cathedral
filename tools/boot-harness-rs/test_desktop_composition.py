"""Desktop migration must preserve grants, other providers and nested identity."""
from copy import deepcopy
import json
import subprocess
import sys
import unittest

import desktop_composition as profile


class DesktopCompositionTests(unittest.TestCase):
    def setUp(self):
        self.stock = json.loads(profile.STOCK.read_text(encoding='utf-8'))

    def test_root_grants_and_other_providers_survive(self):
        original = deepcopy(self.stock)
        selected = profile.omega_matrix(self.stock)
        self.assertEqual(self.stock, original)
        self.assertEqual(selected['startup']['initial'], original['startup']['initial'])
        self.assertEqual(selected['startup']['links'], original['startup']['links'])
        for before, after in zip(original['startup']['launches'], selected['startup']['launches']):
            if before.get('package') == 'cathedral-stock-matrix':
                self.assertEqual(after['entry'], 'distribution/matrix/native/main.omg')
                for key in ('target', 'clock', 'ipc_channels', 'private_pages', 'shared_pages'):
                    self.assertEqual(after[key], before[key])
            else:
                self.assertEqual(after, before)

    def test_every_stock_nested_matrix_uses_the_same_image(self):
        selected = profile.omega_matrix(self.stock)
        for collection in ('programs', 'realm_programs'):
            for before, after in zip(self.stock['startup'][collection], selected['startup'][collection]):
                if before.get('package') == 'cathedral-stock-matrix':
                    self.assertEqual(after['entry'], 'distribution/matrix/native/main.omg')
                    self.assertNotIn('features', after)
                    for key in ('inside', 'path', 'display_title'):
                        self.assertEqual(after.get(key), before.get(key))
                else:
                    self.assertEqual(after, before)

    def test_mixed_host_fixture_keeps_rust_descendants_bounded(self):
        selected = profile.omega_matrix(self.stock, rust_user=True)
        self.assertIn('desktop-host', selected['startup']['programs'][2]['features'])
        self.assertEqual(selected['startup']['realm_programs'], self.stock['startup']['realm_programs'])

    def test_ambiguous_root_is_rejected(self):
        root = next(p for p in self.stock['startup']['launches'] if p.get('package') == 'cathedral-stock-matrix')
        self.stock['startup']['launches'].append(deepcopy(root))
        with self.assertRaisesRegex(ValueError, 'one root matrix'):
            profile.omega_matrix(self.stock)

    def test_cli_refuses_other_boot_modes_before_building(self):
        launcher = profile.ROOT / 'tools/boot-harness-rs/run.py'
        for options in (['--smoke'], ['--kernel-only'],
                        ['--profile', 'tools/boot-harness-rs/profiles/session.json']):
            with self.subTest(options=options):
                result = subprocess.run([sys.executable, str(launcher), '--omega-desktop', *options],
                                        cwd=profile.ROOT, capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
                self.assertIn('--omega-desktop requires the default desktop composition', result.stderr)

    def test_moved_template_is_rejected(self):
        self.stock['startup']['programs'][2]['inside'] = 0
        with self.assertRaisesRegex(ValueError, 'user template'):
            profile.omega_matrix(self.stock)


if __name__ == '__main__':
    unittest.main()
