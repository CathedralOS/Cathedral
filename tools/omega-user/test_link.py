"""Exercise launch declarations through the real assembler and linker."""
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import build
from manifest import encode


@unittest.skipUnless(shutil.which('clang') and shutil.which('ld.lld'), 'requires LLVM')
class LinkTests(unittest.TestCase):
    def test_import_record_is_sealed_at_readonly_load_start_and_can_be_removed(self):
        # Stand in only for compilation. The native adapter, linker script and
        # packaging are real; a previous imports.o must not survive removal.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'source/program'
            source.mkdir(parents=True)
            component = root / 'component'
            component.mkdir()
            (component / 'app.bin').write_bytes(b'\xc3')  # ret; never executed
            (component / 'component.S').write_bytes((
                '.section .text.omega_app,"ax",@progbits\n.global omega_app_step\n'
                'omega_app_step:\n'
                f'.incbin "{(component / "app.bin").as_posix()}"\n').encode())
            (component / 'receipt.json').write_text('{}', encoding='utf-8')
            declaration = source / 'process.json'
            metadata = {'imports': [dict(contract=1, minimum=1, maximum=1, required=True)],
                        'channels': 2, 'descendants': 0}
            declaration.write_text(json.dumps(metadata), encoding='utf-8')
            with patch.object(build, 'ROOT', root), patch.object(
                    build.component_build, 'prepare', return_value=(component, None)):
                image = build.prepare(source)
                data = image.read_bytes()
                loads = build.validate(data).loads
                readonly = [offset for offset, _, files, _, flags in loads if flags == 4 and files]
                self.assertEqual(len(readonly), 1)
                self.assertEqual(data[readonly[0]:readonly[0] + 144], encode(metadata))
                receipt = json.loads(image.with_name('receipt.json').read_text())
                self.assertIn(str(declaration), receipt['inputs'])

                declaration.unlink()
                data = build.prepare(source).read_bytes()
                self.assertNotIn(b'CTHIMP03', data)
                receipt = json.loads(image.with_name('receipt.json').read_text())
                self.assertNotIn(str(declaration), receipt['inputs'])


if __name__ == '__main__':
    unittest.main()
