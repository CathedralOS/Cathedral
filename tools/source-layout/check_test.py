"""Dependency-law regressions: public API exceptions must not admit implementations."""
import unittest
from check import check_edge


class OwnershipTests(unittest.TestCase):
    def test_userspace_can_import_public_kernel_contracts(self):
        for source in ('platform/libraries/user-runtime', 'platform/contracts',
                       'distribution/applications/example'):
            check_edge(source, 'kernel/contracts')

    def test_userspace_cannot_import_kernel_implementation_or_lookalike(self):
        for source in ('platform/services/display', 'distribution/desktop'):
            for target in ('kernel/core', 'kernel/arch', 'kernel/boot/uefi',
                           'kernel/contracts-extra', 'kernel/contracts/internal-provider'):
                with self.subTest(source=source, target=target), self.assertRaises(ValueError):
                    check_edge(source, target)

    def test_contracts_cannot_hide_provider_dependencies(self):
        for source in ('kernel/contracts', 'kernel/contracts/nested',
                       'platform/contracts', 'platform/contracts/display',
                       'platform/libraries/block-device', 'distribution/matrix/contracts'):
            for target in ('kernel/core', 'platform/services/display',
                           'platform/libraries/user-runtime', 'distribution/libraries/boot-scene'):
                with self.subTest(source=source, target=target), self.assertRaises(ValueError):
                    check_edge(source, target)

    def test_kernel_cannot_import_platform_protocols(self):
        for source in ('kernel/core', 'kernel/arch', 'kernel/boot/uefi', 'kernel/contracts'):
            with self.subTest(source=source), self.assertRaises(ValueError):
                check_edge(source, 'platform/contracts')

    def test_existing_bootstrap_and_distribution_boundaries(self):
        check_edge('kernel/boot/uefi', 'platform/drivers/uart_16550')
        check_edge('kernel/arch', 'platform/drivers/facts')
        check_edge('contracts/uefi/raw', 'contracts/uefi/scalars')
        check_edge('distribution/desktop', 'distribution/libraries/boot-scene')
        for source in ('platform/libraries/matrix', 'kernel/core'):
            with self.assertRaises(ValueError):
                check_edge(source, 'distribution/libraries/boot-scene')
        with self.assertRaises(ValueError):
            check_edge('kernel/contracts', 'platform/drivers/facts')
        with self.assertRaises(ValueError):
            check_edge('contracts/uefi/raw', 'platform/libraries/uefi')

    def test_stock_matrix_extensions_cannot_become_platform_dependencies(self):
        check_edge('distribution/matrix', 'platform/libraries/matrix')
        check_edge('distribution/matrix/contracts', 'platform/contracts')
        for source in ('platform/libraries/matrix', 'platform/contracts', 'kernel/core'):
            with self.assertRaises(ValueError):
                check_edge(source, 'distribution/matrix/contracts')
        with self.assertRaises(ValueError):
            check_edge('distribution/matrix/contracts', 'distribution/matrix')

    def test_alternative_matrix_is_independent_of_stock_distribution(self):
        source = 'distribution/lab/matrix-composition/matrix'
        check_edge(source, 'platform/libraries/matrix')
        check_edge(source, 'platform/libraries/user-runtime')
        for target in ('distribution/matrix', 'distribution/matrix/contracts', 'distribution/desktop'):
            with self.assertRaises(ValueError):
                check_edge(source, target)


if __name__ == '__main__':
    unittest.main()
