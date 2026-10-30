"""A mixed desktop must select the right compiler and reject ignored options."""
import os
from pathlib import Path
import unittest
from unittest.mock import patch

import program_build
import probe_programs


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

    def test_notes_probe_is_an_explicit_producer_change(self):
        program = dict(self.program, package='cathedral-notes',
                       entry='distribution/applications/notes/native/main.omg')
        with self.assertRaisesRegex(ValueError, 'Unknown'):
            probe_programs.notes(program, 'trace-storage')
        self.assertTrue(program['entry'].endswith('.omg'))
        probe_programs.notes(program, 'control-delay-probe')
        self.assertEqual(program['entry'], 'distribution/applications/notes/main.rs')
        with patch.object(program_build.subprocess, 'run') as run:
            program_build.build(program, True, program['features'])
        self.assertEqual(run.call_args.args[0][0], 'cargo')
        self.assertIn('control-delay-probe', run.call_args.args[0])
        with self.assertRaisesRegex(ValueError, 'Notes program'):
            probe_programs.notes(self.program, 'control-delay-probe')


if __name__ == '__main__':
    unittest.main()
