"""Timing reports must not invent durations from incomplete or mismatched traces."""
import unittest
from desktop_timing import storage_samples, redraw_samples


class StorageTimingTests(unittest.TestCase):
    def test_extent_is_nested_in_provider_and_io_belongs_to_matching_request(self):
        samples = storage_samples('''Cathedral Rust lab: STORAGE_TRACE provider-begin 0 13 10 100
STORAGE_TRACE nested-send 0 269 11 900
STORAGE_TRACE extent-begin 4096 8192 12 200
STORAGE_TRACE extent-end 4096 8192 15 500
STORAGE_TRACE ata-io 0 13 4 9 3 2 18
STORAGE_TRACE provider-end 0 13 17 800''')
        self.assertEqual([s['phase'] for s in samples], ['extent', 'provider'])
        self.assertEqual(samples[0]['cpu_us'], 300)
        self.assertEqual(samples[1]['guest_ticks'], 7)
        self.assertEqual(samples[1]['read_sectors'], 4)

    def test_truncated_mismatched_or_restarted_clock_produces_no_sample(self):
        for ending in ('', 'provider-end 1 13 20 200', 'provider-end 0 13 9 90'):
            self.assertEqual(storage_samples('STORAGE_TRACE provider-begin 0 13 10 100\nSTORAGE_TRACE ' + ending), [])

    def test_unrelated_io_cannot_be_charged_to_provider(self):
        samples = storage_samples('''STORAGE_TRACE provider-begin 0 3 10 100
STORAGE_TRACE ata-io 1 3 999 999 999 999 999
STORAGE_TRACE provider-end 0 3 11 200''')
        self.assertNotIn('read_sectors', samples[0])

    def test_redraw_cpu_is_never_subtracted_from_other_processes(self):
        samples = redraw_samples('''INPUT_TRACE compositor-redraw-begin 10 100
INPUT_TRACE present-start 10 800
INPUT_TRACE compositor-redraw-end 11 250
INPUT_TRACE compositor-redraw-begin 12 300''')
        self.assertEqual(samples, [dict(guest_ticks=1, cpu_us=150)])


if __name__ == '__main__':
    unittest.main()
