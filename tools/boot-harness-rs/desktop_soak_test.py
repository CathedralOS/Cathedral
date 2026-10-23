"""Soak bookkeeping: provider heap reports and cycle numbering across boots."""
import json
from pathlib import Path
import tempfile
import unittest
from desktop_soak import cycles_before, heap_reports, last_token, reclaimed, shell_accounts


class SoakTests(unittest.TestCase):
    def test_heap_reports_keep_phase_live_peak_failures_and_capacity(self):
        log = ('Cathedral Rust lab: Cathedral objects heap: recovered live=66720 peak=66796 largest=9432 failures=0 capacity=229376 pages=256 frames=532\n'
               'noise\nCathedral objects heap: committed live=134408 peak=160332 largest=9432 failures=1 capacity=229376 pages=276 frames=552\n'
               'Cathedral objects heap: prepared live=1 peak=2 largest=3 failures=0 capacity=4\n')
        reports = heap_reports(log)
        self.assertEqual([r['phase'] for r in reports], ['recovered', 'committed'])
        self.assertEqual(reports[1], dict(phase='committed', live=134408, peak=160332, failures=1, capacity=229376, pages=276, frames=552))

    def test_reclamation_samples_keep_slot_pages_frames_and_free_slots(self):
        log = ('Cathedral Rust lab: STORAGE_TRACE matrix-reclaimed 3 40 96 612 20\n'
               'Cathedral Rust lab: STORAGE_TRACE matrix-budget 1024 4096 40 96\n'
               'Cathedral Rust lab: STORAGE_TRACE matrix-reclaimed 1 24 64 420 26\n')
        self.assertEqual(reclaimed(log), [
            dict(slot=3, private_pages=40, reader_pages=96, frames=612, free_slots=20),
            dict(slot=1, private_pages=24, reader_pages=64, frames=420, free_slots=26)])

    def test_shell_account_samples_keep_pages_frames_and_free_slots(self):
        log = 'Cathedral Rust lab: STORAGE_TRACE shell-account 12 40 300 26' + chr(10) + 'STORAGE_TRACE shell-account 12 41 301 25' + chr(10)
        self.assertEqual(shell_accounts(log), [
            dict(private_pages=12, reader_pages=40, frames=300, free_slots=26),
            dict(private_pages=12, reader_pages=41, frames=301, free_slots=25)])

    def test_cold_boot_continues_the_fresh_boot_numbering(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            self.assertEqual(cycles_before(output), 0)
            self.assertEqual(last_token(output), '')
            (output / 'soak-boot-1.json').write_text(json.dumps(dict(nested_depth=2, reboot=False, cycles=[dict(cycle=1, token='c1'), dict(cycle=2, token='c2'), dict(cycle=3, token='c3')])))
            self.assertEqual(cycles_before(output), 3)
            self.assertEqual(last_token(output), 'c3')


if __name__ == '__main__':
    unittest.main()
