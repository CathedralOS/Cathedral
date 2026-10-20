"""Check first creation and refusal to overwrite persistent database media."""
from pathlib import Path
import os
import tempfile
import subprocess
import unittest
from unittest.mock import patch
import database_image


class DatabaseImageTests(unittest.TestCase):
    def test_fresh_lab_sets_aside_disk_and_receipt_without_reusing_templates(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'desktop.raw'
            sidecar = path.with_name(path.name + '.programs')
            path.write_bytes(b'old users and template snapshots')
            sidecar.write_bytes(b'old program receipt')
            backup = database_image.reset(path)
            self.assertEqual((backup / path.name).read_bytes(), b'old users and template snapshots')
            self.assertEqual((backup / sidecar.name).read_bytes(), b'old program receipt')
            self.assertFalse(path.exists())
            self.assertFalse(sidecar.exists())
            self.assertIsNone(database_image.reset(path))

    def test_failed_fresh_lab_restores_disk_if_receipt_move_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'desktop.raw'
            sidecar = path.with_name(path.name + '.programs')
            path.write_bytes(b'old disk')
            sidecar.write_bytes(b'old receipt')
            move = database_image.shutil.move
            def fail_receipt(source, destination):
                if source == str(sidecar):
                    raise OSError('receipt move refused')
                return move(source, destination)
            with patch.object(database_image.shutil, 'move', side_effect=fail_receipt):
                with self.assertRaisesRegex(OSError, 'receipt move refused'):
                    database_image.reset(path)
            self.assertEqual(path.read_bytes(), b'old disk')
            self.assertEqual(sidecar.read_bytes(), b'old receipt')

    def test_legacy_extent_grows_even_when_installed_programs_are_unchanged(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'desktop.raw'
            path.write_bytes(bytes(database_image.OFFSET + database_image.LEGACY_VOLUME_BYTES + 4096))
            program = Path(directory) / 'app.elf'
            program.write_bytes(b'app')
            import hashlib
            import json
            receipt = json.dumps([[0, hashlib.sha256(b'app').hexdigest()]])
            path.with_name(path.name + '.programs').write_text(receipt)
            def grow(command, **kwargs):
                self.assertEqual(command[3], 'grow')
                path.write_bytes(bytes(database_image.IMAGE_BYTES))
            with patch.object(database_image, 'oracle', return_value=Path(directory) / 'oracle.exe'), patch.object(database_image.subprocess, 'run', side_effect=grow) as run:
                database_image.prepare(path, [(0, program)])
                self.assertEqual(run.call_count, 1)
            self.assertEqual(path.with_name(path.name + '.programs').read_text(), receipt)

    def test_failed_growth_keeps_original_image_and_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'desktop.raw'
            original = bytes(database_image.OFFSET + database_image.LEGACY_VOLUME_BYTES + 4096)
            path.write_bytes(original)
            sidecar = path.with_name(path.name + '.programs')
            sidecar.write_text('[]')
            with patch.object(database_image, 'oracle', return_value=Path(directory) / 'oracle.exe'), patch.object(database_image.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, 'grow')):
                with self.assertRaises(subprocess.CalledProcessError):
                    database_image.prepare(path)
            self.assertEqual(path.read_bytes(), original)
            self.assertEqual(sidecar.read_text(), '[]')

    @unittest.skipUnless(os.name == 'nt', 'Windows image sharing diagnostic')
    def test_busy_desktop_refuses_before_build_or_disk_changes(self):
        import run as harness
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'cathedral-storage.raw'
            original = b'keep the running guest disk'
            path.write_bytes(original)
            with path.open('rb'), patch.object(harness, 'BUILD', Path(directory)), \
                    patch('sys.argv', ['run.py', '--release', '--window']), \
                    patch.object(harness.subprocess, 'run') as build:
                with self.assertRaisesRegex(RuntimeError, 'Close the existing Cathedral QEMU'):
                    harness.main()
                build.assert_not_called()
            self.assertEqual(path.read_bytes(), original)
            database_image.require_available(path)  # Released handle permits a retry.

    def test_availability_probe_does_not_create_missing_media(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'missing' / 'desktop.raw'
            database_image.require_available(path)
            self.assertFalse(path.parent.exists())

    def test_changed_install_is_staged_and_unchanged_install_never_opens(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'desktop.raw'
            path.write_bytes(bytes(database_image.IMAGE_BYTES))
            program = Path(directory) / 'app.elf'
            program.write_bytes(b'app')
            with patch.object(database_image, 'oracle', return_value=Path(directory) / 'target/release/oracle.exe'), patch.object(database_image.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, '', '')) as run:
                database_image.prepare(path, [(0, program)])
                self.assertEqual([call.args[0][3] for call in run.call_args_list], ['install'])
                run.reset_mock()
                database_image.prepare(path, [(0, program)])
                run.assert_not_called()

    def test_failed_maintenance_never_installs_or_records_success(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'desktop.raw'
            path.write_bytes(bytes(database_image.IMAGE_BYTES))
            program = Path(directory) / 'app.elf'
            program.write_bytes(b'app')
            with patch.object(database_image, 'oracle', return_value=Path(directory) / 'target/release/oracle.exe'), patch.object(database_image.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, 'install')) as run:
                with self.assertRaises(subprocess.CalledProcessError):
                    database_image.prepare(path, [(0, program)])
                self.assertEqual(run.call_count, 1)
                self.assertEqual(run.call_args.args[0][3], 'install')
            self.assertFalse(path.with_name(path.name + '.programs').exists())

    def test_new_volume_has_guarded_extent_and_existing_bytes_are_never_reformatted(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'desktop.raw'
            database_image.prepare(path)
            original = path.read_bytes()
            self.assertEqual(len(original), database_image.IMAGE_BYTES)
            self.assertEqual(original[:4096], b'\x5a' * 4096)
            self.assertEqual(original[-4096:], b'\xa5' * 4096)
            with patch.object(database_image.subprocess, 'run') as run:
                database_image.prepare(path)
                run.assert_not_called()
            self.assertEqual(path.read_bytes(), original)

    def test_unknown_existing_media_is_not_replaced(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'desktop.raw'
            path.write_bytes(b'keep this unrelated file')
            with patch.object(database_image.subprocess, 'run') as run:
                with self.assertRaises(RuntimeError):
                    database_image.prepare(path)
                run.assert_not_called()
            self.assertEqual(path.read_bytes(), b'keep this unrelated file')


if __name__ == '__main__':
    unittest.main()
