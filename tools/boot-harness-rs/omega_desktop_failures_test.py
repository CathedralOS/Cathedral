"""Healthy desktop checks must retain failures that cleanup can obscure."""
import unittest

from omega_desktop_test import failed_children


class DesktopFailuresTests(unittest.TestCase):
    def test_cancellation_does_not_hide_failed_startup(self):
        # Depth-three soak: the chooser failed, then its owner cancelled it
        # before collection, so the exit report alone looked intentional.
        for connection in ('owner', 'broker'):
            failure = f'Cathedral chooser: {connection} handshake failed'
            log = ('Cathedral Rust lab: ' + failure + '\n'
                   'Cathedral Rust lab: Cathedral matrix: child 0 0000000000000003 0000000000000000\n')
            self.assertEqual(failed_children(log), [failure])

    def test_healthy_exits_and_intentional_cancellation_are_allowed(self):
        log = ('Cathedral matrix: child 0 0000000000000003 0000000000000000\n'
               'Cathedral matrix: child 4 0000000000000001 0000000000000000\n')
        self.assertEqual(failed_children(log), [])

    def test_failed_exit_is_reported_without_startup_diagnostics(self):
        failure = 'Cathedral matrix: child 4 0000000000000001 ffffffffffffff92'
        self.assertEqual(failed_children(failure), [failure])


if __name__ == '__main__':
    unittest.main()
