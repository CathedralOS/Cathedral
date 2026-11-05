"""Sparse leaf timestamps must not invent successful replies or parent mappings."""
import unittest

from control_timing import background_spans


def stamp(origin, phase, micros, cpu):
    return f"Bg ctl: {origin:016x} {phase} {micros:016x} {cpu:016x}\n"


class BackgroundTimingTests(unittest.TestCase):
    def test_interleaved_instances_and_failed_reply(self):
        text = (stamp(1, 0, 100, 10) + stamp(1, 1, 1000, 100)
                + stamp(2, 1, 1100, 20) + stamp(2, 3, 1200, 40)
                + stamp(1, 2, 3000, 1100))
        rows, pending = background_spans(text)
        self.assertEqual(rows, [(2, 0.1, 0.02, 0.08, "failed"),
                                (1, 2.0, 1.0, 1.0, "published")])
        self.assertEqual(pending, {})

    def test_unanswered_receipt_stays_pending(self):
        rows, pending = background_spans(stamp(1, 1, 1000, 100))
        self.assertEqual(rows, [])
        self.assertEqual(pending, {1: (1000, 100)})

    def test_missing_or_overlapping_receipt_rejected(self):
        for text in (stamp(1, 2, 1000, 100),
                     stamp(1, 1, 1000, 100) + stamp(1, 1, 1100, 200)):
            with self.subTest(text=text), self.assertRaises(ValueError):
                background_spans(text)

    def test_nonmonotonic_cpu_rejected(self):
        with self.assertRaises(ValueError):
            background_spans(stamp(1, 1, 1000, 100) + stamp(1, 2, 2000, 99))

    def test_older_log_has_no_leaf_samples(self):
        self.assertEqual(background_spans("Matrix control: old log"), ([], {}))


if __name__ == "__main__":
    unittest.main()
