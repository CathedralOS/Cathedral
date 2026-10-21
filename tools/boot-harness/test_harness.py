"""Host-only controls: a stale image must never count as a successful build."""

import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('boot', Path(__file__).with_name('run.py'))
boot = importlib.util.module_from_spec(spec)
spec.loader.exec_module(boot)


class BootHarnessTests(unittest.TestCase):
    def test_overlapping_runs_cannot_replace_each_others_image_or_log(self):
        with tempfile.TemporaryDirectory() as temporary:
            parent = Path(temporary)
            first = boot.create_run_directory(parent)
            second = boot.create_run_directory(parent)
            self.assertNotEqual(first, second)
            def compile_image(command):
                output = Path(command[command.index('--build-dir') + 1])
                (output / 'omega-program.exe').write_bytes(output.parent.name.encode())
            with patch.object(boot, 'run_checked', side_effect=compile_image):
                first_image = boot.build_image(['omega'], Path('main.omg'), 'uefi_x86_64', first)
                second_image = boot.build_image(['omega'], Path('main.omg'), 'uefi_x86_64', second)
            (first / 'serial.log').write_bytes(b'first')
            (second / 'serial.log').write_bytes(b'second')
            self.assertEqual(first_image.read_bytes(), first.name.encode())
            self.assertEqual(second_image.read_bytes(), second.name.encode())
            self.assertEqual((first / 'serial.log').read_bytes(), b'first')

    def test_smoke_requires_ordered_complete_reports_through_resident_readback(self):
        boot.check_boot_milestones(b'Hello from Omega\r\nOwned 128 MiB\r\nKernel entered\r\nMemory split\r\nMemory checked\r\nMemory paired\r\n')
        boot.check_boot_milestones(b'Hello from Omega\nOwned 99999+ MiB\nKernel entered\r\nMemory split\r\nMemory checked\nMemory paired\n')
        for output in (b'', b'Hello from Omega\r\n',
                       b'Owned 128 MiB\nHello from Omega\n',
                       b'Hello from Omega\nOwned rubbish MiB\n',
                       b'Hello from Omega\nOwned 128 MiB',
                       b'Hello from Omega\nOwned 128 MiB\n',
                       b'Hello from Omega\nKernel entered\r\nOwned 128 MiB\n',
                       b'Hello from Omega\nOwned 128 MiB\nKernel entered',
                       b'Hello from Omega\nOwned rubbish MiB\nKernel entered\r\n',
                       b'Hello from Omega\nOwned 128 MiB\nKernel entered\r\n',
                       b'Hello from Omega\nOwned 128 MiB\nMemory split\r\nKernel entered\r\n',
                       b'Hello from Omega\nOwned 128 MiB\nKernel entered\r\nMemory split'):
            with self.subTest(output=output):
                with self.assertRaises(ValueError):
                    boot.check_boot_milestones(output)

    def test_split_only_or_invalid_resident_report_cannot_satisfy_smoke(self):
        through_split = b'Hello from Omega\r\nOwned 128 MiB\r\nKernel entered\r\nMemory split\r\n'
        for output in (through_split,
                       through_split + b'Memory checked',
                       through_split + b'Memory checked\r',
                       through_split + b'Memory failed\r\n',
                       through_split + b'Not Memory checked\r\n',
                       b'Memory checked\r\n' + through_split,
                       through_split.replace(b'Memory split\r\n', b'Memory checked\r\nMemory split\r\n')):
            with self.subTest(output=output):
                with self.assertRaisesRegex(ValueError, 'resident initialization'):
                    boot.check_boot_milestones(output)

    def test_source_checkout_uses_current_cli_package(self):
        with tempfile.TemporaryDirectory() as temporary:
            repository = Path(temporary)
            (repository / 'Cargo.toml').touch()
            command = boot.compiler_command(None, repository)
            self.assertEqual(command[-3:], ['-p', 'omega', '--'])

    def test_single_resident_or_invalid_pair_report_cannot_satisfy_smoke(self):
        prefix = b'Hello from Omega\r\nOwned 128 MiB\r\nKernel entered\r\nMemory split\r\nMemory checked\r\n'
        for output in (prefix, prefix + b'Memory paired', prefix + b'Memory paired\r',
                       prefix + b'Not Memory paired\r\n', b'Memory paired\r\n' + prefix,
                       prefix.replace(b'Memory checked\r\n', b'Memory paired\r\nMemory checked\r\n')):
            with self.subTest(output=output):
                with self.assertRaisesRegex(ValueError, 'coexisting resident'):
                    boot.check_boot_milestones(output)

    def test_explicit_binary_wins(self):
        command = boot.compiler_command('selected-omega', Path('absent'))
        self.assertEqual(command, [str(Path('selected-omega').resolve())])

    def test_failed_build_does_not_replace_or_return_stale_image(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            stale = directory / 'esp/EFI/BOOT/BOOTX64.EFI'
            stale.parent.mkdir(parents=True)
            stale.write_bytes(b'old')
            with patch.object(boot, 'run_checked', side_effect=subprocess.CalledProcessError(1, 'omega')):
                with self.assertRaises(subprocess.CalledProcessError):
                    boot.build_image(['omega'], Path('main.omg'), 'uefi_x86_64', directory)
            self.assertEqual(stale.read_bytes(), b'old')

    def test_success_without_product_rejects_stale_image(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / 'omega-program.exe').write_bytes(b'old')
            with patch.object(boot, 'run_checked'):
                with self.assertRaisesRegex(ValueError, 'without publishing'):
                    boot.build_image(['omega'], Path('main.omg'), 'uefi_x86_64', directory)

    def test_publishes_only_fresh_product(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            def compile_image(command):
                output = Path(command[command.index('--build-dir') + 1])
                (output / 'omega-program.exe').write_bytes(b'new')
            with patch.object(boot, 'run_checked', side_effect=compile_image):
                image = boot.build_image(['omega'], Path('main.omg'), 'uefi_x86_64', directory)
            self.assertEqual(image.read_bytes(), b'new')
            self.assertEqual(list(directory.glob('compile-*')), [])


if __name__ == '__main__':
    unittest.main()
