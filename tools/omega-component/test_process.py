"""A compiler allocation failure must stay inside its process budget."""
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch
import process


class ProcessBudget(unittest.TestCase):
    def invoke(self, source):
        return subprocess.run([sys.executable, str(Path(__file__).with_name('process.py')),
                               str(128 * 1024**2), sys.executable, '-c', source],
                              capture_output=True, text=True, timeout=20)

    def test_small_program_keeps_output_and_status(self):
        result = self.invoke('print("bounded child"); raise SystemExit(7)')
        self.assertEqual(result.returncode, 7)
        self.assertEqual(result.stdout.strip(), 'bounded child')

    def test_large_allocation_is_refused(self):
        result = self.invoke('try:\n bytearray(512 * 1024**2)\nexcept MemoryError:\n print("refused")\nelse:\n raise SystemExit(9)')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), 'refused')

    def test_build_wrapper_preserves_output(self):
        result = process.run([sys.executable, '-c', 'print("wrapped")'],
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), 'wrapped')

    def test_build_wrapper_stops_on_deadline(self):
        with patch.object(process, 'COMPILER_TIMEOUT', 0.5):
            with self.assertRaisesRegex(RuntimeError, 'exceeded'):
                process.run([sys.executable, '-c', 'import time; time.sleep(60)'],
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)


if __name__ == '__main__':
    unittest.main()
