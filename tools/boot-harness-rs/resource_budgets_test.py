"""Host admission checks: profile values cannot overpromise kernel backing."""
import unittest
from resource_budgets import channel_limits, cpu_limits

class Reservations(unittest.TestCase):
    def test_nine_accounts_fit_with_explicit_backing(self):
        startup = {"initial": {}, "launches": [{}, {}, {"ipc_channels": 20}, {"ipc_channels": 0}], "dynamic_tasks": 4}
        self.assertEqual(channel_limits(startup), [4, 4, 4, 20, 0, 0, 0, 0, 0])
    def test_dynamic_accounts_have_no_unfunded_channel_capacity(self):
        startup = {"initial": {}, "launches": [{}], "dynamic_tasks": 6}
        self.assertEqual(channel_limits(startup), [4, 4, 0, 0, 0, 0, 0, 0])
        self.assertEqual(cpu_limits(startup), [0] * 8)
        for value in [-1, 14, True, "2"]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                channel_limits({"initial": {}, "launches": [{}], "dynamic_tasks": value})
    def test_cpu_inheritance_and_ceiling_values(self):
        self.assertEqual(cpu_limits({"initial": {}, "launches": [{}, {}]}), [0, 0, 0])
        self.assertEqual(cpu_limits({"initial": {"cpu_budget_us": 20000}, "launch": {"cpu_budget_us": 100000}}), [20000, 100000])
        for value in [-1, 100001, True, "20000", 0.5, None]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                cpu_limits({"initial": {"cpu_budget_us": value}})
    def test_defaults_are_stable_and_eight_full_reservations_fit(self):
        self.assertEqual(channel_limits({"initial": {}}), [4])
        self.assertEqual(channel_limits({"initial": {}, "launches": [{} for _ in range(7)]}), [4] * 8)

    def test_explicit_zero_and_full_pool(self):
        self.assertEqual(channel_limits({"initial": {"ipc_channels": 0}, "launch": {"ipc_channels": 8}}), [0, 8])
        self.assertEqual(channel_limits({"initial": {"ipc_channels": 8}, "launches": [{"ipc_channels": 8}, {}, {}, {}, {}]}), [8, 8, 4, 4, 4, 4])

    def test_invalid_values_and_overcommit(self):
        for value in [-1, 93, True, "4", 1.5, None]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                channel_limits({"initial": {"ipc_channels": value}})
        with self.assertRaises(ValueError):
            channel_limits({"initial": {"ipc_channels": 8}, "launches": [{"ipc_channels": 12} for _ in range(9)]})
        with self.assertRaises(ValueError):
            channel_limits({"initial": {}, "launches": [{} for _ in range(15)]})

if __name__ == "__main__":
    unittest.main()
