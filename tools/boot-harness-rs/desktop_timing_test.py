"""Timing reports must not invent durations from incomplete or mismatched traces."""
import unittest
from desktop_timing import storage_samples, redraw_samples, input_samples
from hop_latency import intervals, summarize
import startup_timeline


class StorageTimingTests(unittest.TestCase):
    def test_input_keeps_hops_separate_and_ignores_releases(self):
        samples = input_samples('''INPUT_TRACE source-key 10 5000
INPUT_TRACE source-release 10 5100
INPUT_TRACE matrix-key 11 100
INPUT_TRACE matrix-key 12 9000
INPUT_TRACE notes-key 13 50''')
        self.assertEqual([s['guest_tick'] for s in samples], [10, 11, 12, 13])
        self.assertEqual([s['account_cpu_us'] for s in samples], [5000, 100, 9000, 50])

    def test_collection_is_nested_in_provider_and_io_belongs_to_matching_request(self):
        samples = storage_samples('''Cathedral Rust lab: STORAGE_TRACE provider-begin 0 3 10 100 1000500
STORAGE_TRACE nested-send 0 259 11 900 1010000
STORAGE_TRACE collect-begin 0 3 12 200 1020000
STORAGE_TRACE publish-begin 0 3 15 500 1050000
STORAGE_TRACE ata-io 0 3 4 9 3 2 18
STORAGE_TRACE provider-end 0 3 17 800 1070000''')
        self.assertEqual([s['phase'] for s in samples], ['collection', 'provider'])
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


class HopLatencyTests(unittest.TestCase):
    def test_consecutive_stages_pair_into_named_intervals_in_serial_order(self):
        groups = intervals("""STORAGE_TRACE trace-calibrate 0 0 9 50 4000
STORAGE_TRACE trace-calibrate 0 0 9 60 4100
STORAGE_TRACE nested-send 2 257 10 100 5000
STORAGE_TRACE root-receive 7 257 10 300 5400
STORAGE_TRACE root-admit 7 257 10 350 5450
STORAGE_TRACE root-send 0 1 10 400 5600
STORAGE_TRACE provider-begin 0 1 10 900 5900
STORAGE_TRACE ata-io 0 1 0 0 0 0 0
STORAGE_TRACE provider-end 0 1 11 1900 7900
STORAGE_TRACE root-reply 7 257 11 500 8200
STORAGE_TRACE nested-reply 2 257 11 200 8600
STORAGE_TRACE nested-send 2 1 11 250 4000""")
        self.assertEqual(groups['hop up: matrix to root'], [400])
        self.assertEqual(groups['root dispatch'], [50])
        self.assertEqual(groups['root forward'], [150])
        self.assertEqual(groups['hop up: root to provider'], [300])
        self.assertEqual(groups['provider work'], [2000])
        self.assertEqual(groups['hop down: provider to root'], [300])
        self.assertEqual(groups['hop down: root to matrix'], [400])
        # A clock that runs backwards between two lines is not an interval.
        self.assertNotIn('hop up: matrix to matrix', groups)
        self.assertEqual(groups['trace line itself'], [100])
        rows = summarize(groups)
        self.assertEqual(rows[0]['stage'], 'provider work')
        self.assertEqual((rows[0]['count'], rows[0]['median_us'], rows[0]['net_median_us']), (1, 2000, 1900))
        self.assertEqual(next(r for r in rows if r['stage'] == 'root dispatch')['net_median_us'], 0)

class StartupTimelineTests(unittest.TestCase):
    LOG = '''Cathedral Rust lab: STORAGE_TRACE trace-calibrate 0 0 5 100 1000000
Cathedral Rust lab: STORAGE_TRACE trace-calibrate 0 0 5 300 1000400
Cathedral Rust lab: STORAGE_TRACE nested-send 1 275 5 400 1001000
Cathedral Rust lab: STORAGE_TRACE root-send 1 257 5 500 1010000
Cathedral Rust lab: STORAGE_TRACE provider-begin 0 1 5 600 1011000
Cathedral Rust lab: STORAGE_TRACE provider-end 0 1 5 700 1012000
Cathedral Rust lab: Cathedral matrix: configured from realm paths
Cathedral Rust lab: STORAGE_TRACE root-send 1 275 5 800 1020000
Cathedral Rust lab: STORAGE_TRACE provider-end 0 19 5 900 1025000
Cathedral Rust lab: STORAGE_TRACE root-send 1 263 5 950 1400000
'''

    def test_startup_runs_from_the_calibration_to_the_launcher_naming_its_tiles(self):
        rows = startup_timeline.timeline(self.LOG)
        self.assertEqual(rows, [
            (10.0, 'root-send', 'HEAD'), (12.0, 'provider-end', 'HEAD'),
            (None, 'console', 'Cathedral matrix: configured from realm paths'),
            (20.0, 'root-send', 'ENTRIES'), (25.0, 'provider-end', 'ENTRIES')])
        self.assertEqual(startup_timeline.summarize(rows),
                         '2 requests reached the provider; the last was answered 25.0 ms after the matrix started')

    def test_an_earlier_matrix_is_chosen_by_index_and_a_log_without_stamps_is_empty(self):
        earlier = '''Cathedral Rust lab: STORAGE_TRACE trace-calibrate 0 0 4 100 500000
Cathedral Rust lab: STORAGE_TRACE trace-calibrate 0 0 4 300 500400
Cathedral Rust lab: Cathedral matrix: configured from realm paths
Cathedral Rust lab: STORAGE_TRACE root-send 1 275 4 800 505000
Cathedral Rust lab: STORAGE_TRACE provider-end 0 19 4 900 509000
'''
        self.assertEqual(startup_timeline.timeline(earlier + self.LOG, 0), [
            (None, 'console', 'Cathedral matrix: configured from realm paths'),
            (5.0, 'root-send', 'ENTRIES'), (9.0, 'provider-end', 'ENTRIES')])
        self.assertEqual(startup_timeline.timeline(earlier + self.LOG)[0], (10.0, 'root-send', 'HEAD'))
        self.assertEqual(startup_timeline.timeline('Cathedral Rust lab: nothing traced'), [])
        self.assertEqual(startup_timeline.summarize([]), 'no request of this startup reached the provider')


if __name__ == '__main__':
    unittest.main()
