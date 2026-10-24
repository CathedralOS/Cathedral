"""Compile one pure Omega application export for the Rust reference host."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
from elf import Elf, component
from locking import exclusive

ROOT = Path(__file__).resolve().parents[2]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def prepare(source, compiler=None):
    source = source.resolve()
    output = ROOT / 'build/omega-component' / source.relative_to(ROOT / 'source')
    with exclusive(output / '.prepare.lock'):
        return prepare_locked(source, compiler, output)


def prepare_locked(source, compiler, output):
    local = ROOT / 'build/omega-ui/compiler/target/release' / ('omega.exe' if os.name == 'nt' else 'omega')
    selected = compiler or os.environ.get('OMEGA_BIN') or (local if local.is_file() else shutil.which('omega'))
    if not selected:
        raise ValueError('build the supported compiler in tools/omega-ui/README.md#build or set OMEGA_BIN')
    compiler = Path(selected).resolve()
    profile = json.loads(subprocess.check_output([str(compiler), '--build-profile'], text=True))
    if profile.get('opt_level') not in ('1', '2', '3', 's', 'z'):
        raise ValueError('an optimized Omega compiler is required')
    # The native consumer and app plus their explicit local package dependencies.
    import re
    roots, pending = set(), [source]
    while pending:
        root = pending.pop().resolve()
        if root in roots:
            continue
        if not root.is_relative_to(ROOT / 'source'):
            raise ValueError(f'package outside checked-in source: {root}')
        roots.add(root)
        manifest = (root / 'build.omg').read_text(encoding='utf-8')
        pending.extend(root / path for path in re.findall(r'location:\s*"([^"]+)"', manifest))
    paths = {compiler, Path(__file__), Path(__file__).with_name('elf.py'), Path(__file__).with_name('locking.py')}
    for root in roots:
        paths.update(p for p in root.rglob('*.omg') if 'build' not in p.relative_to(root).parts[:-1])
    inputs = lambda: {str(path): digest(path) for path in sorted(paths)}
    before = inputs()
    receipt = output / 'receipt.json'
    if receipt.is_file():
        saved = json.loads(receipt.read_text())
        if saved['inputs'] == before and all((output / name).is_file() and digest(output / name) == value for name, value in saved['outputs'].items()):
            return output, roots | paths
    output.mkdir(parents=True, exist_ok=True)
    for step, args in (
        ('review', ['update', '--project', str(source), '--target', 'linux_x86_64', '--offline']),
        ('compile', ['--target', 'linux_x86_64', '--build-dir', str(output / 'native'), str(source / 'main.omg')]),
    ):
        log = output / (step + '.log')
        with log.open('w', encoding='utf-8') as stream:
            result = subprocess.run([str(compiler), *args], cwd=source, stdout=stream, stderr=subprocess.STDOUT)
        if result.returncode:
            raise RuntimeError(f'{step} failed: {log}')
    needed = Elf((output / 'native/omega-program').read_bytes(), 2).needed()
    dependency = needed[0].decode() if len(needed) == 1 else ''
    filename = dependency.removeprefix('$ORIGIN/')
    if not filename or Path(filename).name != filename or dependency != '$ORIGIN/' + filename:
        raise ValueError('app must select exactly one local component')
    binary = output / 'native' / filename
    body, (symbol, entry) = component(binary.read_bytes())
    blob = output / 'app.bin'
    blob.write_bytes(body)
    (output / 'component.S').write_text('\n'.join([
        '.section .text.omega_app,"ax",@progbits', '.balign 16',
        'omega_app_base:', f'.incbin "{blob.as_posix()}"',
        '.global omega_app_step', f'.set omega_app_step, omega_app_base + {entry}', '',
    ]))
    (output / 'component.rs').write_text(f'pub const ENTRY: usize = {entry};\npub static BYTES: &[u8] = include_bytes!(r#"{blob.as_posix()}"#);\n')
    if inputs() != before:
        raise RuntimeError('source/compiler changed during compilation; retry')
    receipt.write_text(json.dumps({'inputs': before, 'profile': profile, 'symbol': symbol,
        'component_sha256': digest(binary), 'entry': entry, 'bytes': len(body),
        'outputs': {name: digest(output / name) for name in ('app.bin', 'component.S', 'component.rs')}}, indent=2) + '\n')
    return output, roots | paths


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path, help='native consumer package')
    parser.add_argument('--compiler', type=Path)
    parser.add_argument('--cargo', action='store_true')
    args = parser.parse_args()
    try:
        output, paths = prepare(args.source, args.compiler)
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Omega application preparation failed: {error}\n')
    if args.cargo:
        for path in sorted(paths):
            print(f'cargo:rerun-if-changed={path}')
    print(output)
