"""Host admission checks: profile values cannot overpromise kernel backing."""
import unittest
from resource_budgets import channel_limits, cpu_limits

class Reservations(unittest.TestCase):
    def test_cpu_inheritance_and_ceiling_values(self):
        self.assertEqual(cpu_limits({"initial": {}, "launches": [{}, {}]}), [0, 0, 0])
        self.assertEqual(cpu_limits({"initial": {"cpu_budget_us": 20000}, "launch": {"cpu_budget_us": 100000}}), [20000, 100000])
        for value in [-1, 100001, True, "20000", 0.5, None]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                cpu_limits({"initial": {"cpu_budget_us": value}})
    def test_defaults_fit_all_slots_and_do_not_depend_on_graph_size(self):
        self.assertEqual(channel_limits({"initial": {}}), [4])
        self.assertEqual(channel_limits({"initial": {}, "launches": [{} for _ in range(7)]}), [4] * 8)

    def test_explicit_zero_and_full_pool(self):
        self.assertEqual(channel_limits({"initial": {"ipc_channels": 0}, "launch": {"ipc_channels": 8}}), [0, 8])
        self.assertEqual(channel_limits({"initial": {"ipc_channels": 8}, "launches": [{"ipc_channels": 8}, {}, {}, {}, {}]}), [8, 8, 4, 4, 4, 4])

    def test_invalid_values_and_overcommit(self):
        for value in [-1, 9, True, "4", 1.5, None]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                channel_limits({"initial": {"ipc_channels": value}})
        with self.assertRaises(ValueError):
            channel_limits({"initial": {"ipc_channels": 8}, "launches": [{} for _ in range(7)]})
        with self.assertRaises(ValueError):
            channel_limits({"initial": {}, "launches": [{} for _ in range(8)]})

if __name__ == "__main__":
    unittest.main()
