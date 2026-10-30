"""Build a profile executable with the producer named by its source entry.

The Rust kernel accepts the same ELF contract from either producer. Omega uses
the standalone runtime compiler; the desktop's legacy component compiler is a
separate selection. Capture and resource checks remain with the boot harness.
"""
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
BUILD = ROOT / 'build/boot-harness-rs'


def omega_source(program):
    if Path(program['entry']).suffix != '.omg':
        return None
    entry = (ROOT / 'source' / program['entry']).resolve()
    if not any(entry.is_relative_to(ROOT / 'source' / owner)
               for owner in ('distribution', 'platform/services')):
        raise ValueError('Omega profile entry must belong to distribution or platform services')
    if entry.name != 'main.omg' or not entry.is_file() or not entry.with_name('build.omg').is_file():
        raise ValueError('Omega profile entry must name a native package main.omg')
    if program['target'] != 'x86_64-unknown-none':
        raise ValueError('standalone Omega programs currently require the x86-64 ELF target')
    return entry.parent


def image(program, release):
    source = omega_source(program)
    if source is not None:
        return ROOT / 'build/omega-user' / source.relative_to(ROOT / 'source') / 'program.elf'
    return BUILD / 'cargo' / program['target'] / ('release' if release else 'debug') / program['package']


def build(program, release, features):
    source = omega_source(program)
    if source is not None:
        if features:
            raise ValueError('Cargo features cannot configure an Omega executable; select its native package')
        compiler = Path(os.environ.get('OMEGA_USER_BIN') or
                        ROOT / 'build/omega-user/compiler' / ('omega.exe' if os.name == 'nt' else 'omega'))
        subprocess.run([sys.executable, str(ROOT / 'tools/omega-user/build.py'), str(source),
                        '--compiler', str(compiler.resolve())], cwd=ROOT, check=True)
    else:
        command = ['cargo', 'build', '--locked', '--package', program['package'],
                   '--target', program['target'], '--target-dir', str(BUILD / 'cargo')]
        if features:
            command += ['--features', ','.join(features)]
        if release:
            command.append('--release')
        subprocess.run(command, cwd=ROOT / 'source-rs', check=True)
    return image(program, release)
