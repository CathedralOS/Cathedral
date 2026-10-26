"""Fatal kernel reports end a protocol run even when the guest remains halted."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import protocol_test


class ProtocolFailureTests(unittest.TestCase):
    def test_halted_kernel_fault_is_reported_as_failure_not_timeout(self):
        command = [sys.executable, '-c',
                   "from pathlib import Path; import time; "
                   "Path('serial.log').write_text('CATHEDRAL_RS_FAULT: vector=8'); "
                   "time.sleep(60)"]
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(RuntimeError, 'Protocol guest failed: CATHEDRAL_RS_FAULT'):
                protocol_test.run(command, Path(directory), 5,
                                  getattr(subprocess, 'CREATE_NO_WINDOW', 0),
                                  'success', abrupt=True)


if __name__ == '__main__':
    unittest.main()
