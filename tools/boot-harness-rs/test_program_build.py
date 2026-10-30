"""A mixed desktop must select the right compiler and reject ignored options."""
import os
from pathlib import Path
import unittest
from unittest.mock import patch

import program_build


class ProgramBuildTests(unittest.TestCase):
    def setUp(self):
        self.program = dict(package='cathedral-background', target='x86_64-unknown-none',
                            entry='distribution/applications/background/native/main.omg')

    def test_omega_uses_the_standalone_producer_and_image(self):
        compiler = program_build.ROOT / 'build/test-standalone-compiler/omega.exe'
        with patch.dict(os.environ, {'OMEGA_USER_BIN': str(compiler)}), \
                patch.object(program_build.subprocess, 'run') as run:
            image = program_build.build(self.program, True, [])
        command = run.call_args.args[0]
        self.assertEqual(command[-2:], ['--compiler', str(compiler.resolve())])
        self.assertEqual(Path(command[1]), program_build.ROOT / 'tools/omega-user/build.py')
        self.assertEqual(image, program_build.ROOT /
                         'build/omega-user/distribution/applications/background/native/program.elf')

    def test_omega_refuses_rust_features_and_unsupported_architecture(self):
        with patch.object(program_build.subprocess, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'Cargo features'):
                program_build.build(self.program, True, ['startup-fault'])
            with self.assertRaisesRegex(ValueError, 'x86-64'):
                program_build.build(dict(self.program, target='aarch64-unknown-none'), True, [])
            with self.assertRaisesRegex(ValueError, 'must belong'):
                program_build.image(dict(self.program, entry='../outside/main.omg'), True)
            run.assert_not_called()

    def test_rust_keeps_its_profile_and_features(self):
        program = dict(self.program, entry='distribution/applications/background/main.rs')
        with patch.object(program_build.subprocess, 'run') as run:
            image = program_build.build(program, False, ['startup-fault'])
        command = run.call_args.args[0]
        self.assertEqual(command[:4], ['cargo', 'build', '--locked', '--package'])
        self.assertEqual(command[-2:], ['--features', 'startup-fault'])
        self.assertNotIn('--release', command)
        self.assertEqual(image, program_build.BUILD / 'cargo/x86_64-unknown-none/debug/cathedral-background')


if __name__ == '__main__':
    unittest.main()
