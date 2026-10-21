"""Wait reports retain units, populations and unfinished/canceled observations."""
import unittest
import contextlib
import io
import timing_report as report


def row(task=0, kind="Readiness", **updates):
    values = dict(completed=2, selected=2, cancelled=0, pending=0,
                  total_ready_us=300, max_ready_us=200, due=2,
                  total_late_ticks=3, max_late_ticks=3)
    values.update(updates)
    return f"WAIT_DISPATCH task={task} kind={kind} " + " ".join(f"{key}={values[key]}" for key in report.WAIT_FIELDS)


def case(record, blocked=2):
    return f"CPU timing: cap_us=0 yielding_peer=0\n{record}\nCPU timing: periodic=x peer=y periodic_waits={blocked}\n"


class WaitReports(unittest.TestCase):
    def test_all_immediate_timeouts_need_no_registered_wait_sample(self):
        self.assertEqual(report.verify_periodic_waits(case("", blocked=0)), 1)

    def test_unselected_population_does_not_print_a_zero_latency(self):
        rows = report.summarize_waits(case(row(selected=0, pending=2, total_ready_us=0, max_ready_us=0)))
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            report.print_waits(rows["busy, cap=0 us"])
        fields = output.getvalue().splitlines()[-1].split()
        self.assertEqual(fields[5:7], ["-", "-"])

    def test_periodic_counts_and_units(self):
        text = row() + "\n" + case(row())
        self.assertEqual(report.verify_periodic_waits(text), 1)
        samples = report.summarize_waits(text)["busy, cap=0 us"]
        self.assertEqual(samples[(0, "Readiness")]["total_ready_us"], 300)
        self.assertEqual(samples[(0, "Readiness")]["total_late_ticks"], 3)
        self.assertIn("other sessions", report.summarize_waits(text, True))

    def test_missing_duplicate_and_incomplete_observations_refuse_verification(self):
        for text in ["", case(""), case(row() + "\n" + row()),
                     case(row(selected=1, pending=1)), case(row(cancelled=1)),
                     case(row(), blocked=3), case(row(max_ready_us=301)),
                     "CPU timing: cap_us=0 yielding_peer=0\n" + row()]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                report.verify_periodic_waits(text)

    def test_aggregation_keeps_cancellations_and_maxima(self):
        text = case(row(selected=1, cancelled=1)) + case(row(total_ready_us=400, max_ready_us=250))
        sample = report.summarize_waits(text)["busy, cap=0 us"][(0, "Readiness")]
        self.assertEqual((sample["completed"], sample["selected"], sample["cancelled"]), (4, 3, 1))
        self.assertEqual((sample["total_ready_us"], sample["max_ready_us"]), (700, 250))


if __name__ == "__main__":
    unittest.main()
