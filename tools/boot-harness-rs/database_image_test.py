"""Check first creation and refusal to overwrite persistent database media."""
from pathlib import Path
import tempfile
import subprocess
import unittest
from unittest.mock import patch
import database_image


class DatabaseImageTests(unittest.TestCase):
    def test_changed_install_compacts_first_and_unchanged_install_never_opens(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'desktop.raw'
            path.write_bytes(bytes(database_image.IMAGE_BYTES))
            program = Path(directory) / 'app.elf'
            program.write_bytes(b'app')
            with patch.object(database_image, 'oracle', return_value=Path(directory) / 'target/release/oracle.exe'), patch.object(database_image.subprocess, 'run') as run:
                database_image.prepare(path, [(0, program)])
                self.assertEqual([call.args[0][3] for call in run.call_args_list], ['compact', 'seed'])
                run.reset_mock()
                database_image.prepare(path, [(0, program)])
                run.assert_not_called()

    def test_failed_maintenance_never_installs_or_records_success(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'desktop.raw'
            path.write_bytes(bytes(database_image.IMAGE_BYTES))
            program = Path(directory) / 'app.elf'
            program.write_bytes(b'app')
            with patch.object(database_image, 'oracle', return_value=Path(directory) / 'target/release/oracle.exe'), patch.object(database_image.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, 'compact')) as run:
                with self.assertRaises(subprocess.CalledProcessError):
                    database_image.prepare(path, [(0, program)])
                self.assertEqual(run.call_count, 1)
                self.assertEqual(run.call_args.args[0][3], 'compact')
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
