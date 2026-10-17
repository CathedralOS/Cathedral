"""Check first creation and refusal to overwrite persistent database media."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import database_image


class DatabaseImageTests(unittest.TestCase):
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
