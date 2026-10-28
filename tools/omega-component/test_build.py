"""Build-cache integrity across dependency edits, corruption and concurrent builds."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import build
from locking import exclusive


class BuildTests(unittest.TestCase):
    def test_dependency_and_output_changes_rebuild_but_an_intact_cache_does_not(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            app = root / 'source/distribution/app'
            library = root / 'source/platform/lib'
            app.mkdir(parents=True)
            library.mkdir(parents=True)
            (app / 'build.omg').write_text('location: "../../platform/lib"')
            (library / 'build.omg').write_text('library')
            code = library / 'main.omg'
            code.write_text('first')
            compiler = root / 'omega'
            compiler.write_bytes(b'compiler')
            output = root / 'build/omega-component/distribution/app'

            def compile(args, **_kwargs):
                if '--build-dir' in args:
                    native = output / 'native'
                    native.mkdir(exist_ok=True)
                    (native / 'omega-program').write_bytes(b'program')
                    (native / 'libapp.so').write_bytes(b'library')
                return subprocess.CompletedProcess(args, 0)

            with patch.object(build, 'ROOT', root), \
                    patch.object(build.subprocess, 'check_output', return_value='{"opt_level":"3"}'), \
                    patch.object(build.process, 'run', side_effect=compile) as invoke, \
                    patch.object(build, 'Elf') as elf, \
                    patch.object(build, 'component', return_value=(b'checked code', ('entry', 0))):
                elf.return_value.needed.return_value = [b'$ORIGIN/libapp.so']
                actual, paths = build.prepare(app, compiler)
                self.assertEqual(actual, output)
                self.assertIn(code, paths)
                self.assertEqual(invoke.call_count, 2)
                build.prepare(app, compiler)
                self.assertEqual(invoke.call_count, 2)
                code.write_text('second')
                build.prepare(app, compiler)
                self.assertEqual(invoke.call_count, 4)
                (output / 'app.bin').write_bytes(b'corrupt')
                build.prepare(app, compiler)
                self.assertEqual(invoke.call_count, 6)
                self.assertEqual((output / 'app.bin').read_bytes(), b'checked code')
                receipt = json.loads((output / 'receipt.json').read_text())
                self.assertEqual(receipt['inputs'][str(code)], build.digest(code))

    def test_lock_serializes_a_second_process_and_releases_on_exception(self):
        with tempfile.TemporaryDirectory() as directory:
            lock = Path(directory) / 'prepare.lock'
            entered = Path(directory) / 'entered'
            script = ('import sys; from pathlib import Path; from locking import exclusive; '
                      'print("ready", flush=True)\n'
                      'with exclusive(Path(sys.argv[1])): Path(sys.argv[2]).write_text("entered")\n')
            process = None
            try:
                with self.assertRaisesRegex(RuntimeError, 'release'):
                    with exclusive(lock):
                        process = subprocess.Popen([sys.executable, '-c', script, str(lock), str(entered)],
                            cwd=Path(__file__).parent, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                        self.assertEqual(process.stdout.readline().strip(), 'ready')
                        with self.assertRaises(subprocess.TimeoutExpired):
                            process.wait(timeout=0.2)
                        self.assertFalse(entered.exists())
                        raise RuntimeError('release')
                stdout, stderr = process.communicate(timeout=5)
                self.assertEqual(process.returncode, 0, stdout + stderr)
                self.assertEqual(entered.read_text(), 'entered')
            finally:
                if process and process.poll() is None:
                    process.kill()
                    process.communicate()


if __name__ == '__main__':
    unittest.main()
