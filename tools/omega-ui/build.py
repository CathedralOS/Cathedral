"""Build Cathedral's checked-in Omega UI, then package its pure native exports."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'omega-component'))
from elf import Elf, component
import process

ROOT = Path(__file__).resolve().parents[2]
LIBRARY = ROOT / 'source/distribution/libraries/omega-ui'
OUTPUT = ROOT / 'build/omega-ui/component'
LOCAL_COMPILER = ROOT / 'build/omega-ui/compiler/target/release' / ('omega.exe' if os.name == 'nt' else 'omega')


def select_compiler(explicit=None):
    selected = explicit or os.environ.get('OMEGA_BIN')
    if not selected:
        selected = LOCAL_COMPILER if LOCAL_COMPILER.is_file() else shutil.which('omega')
    if not selected:
        raise ValueError('Omega compiler not found; build the supported compiler as described '
                         'in tools/omega-ui/README.md#build or set OMEGA_BIN')
    return Path(selected).resolve()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fingerprint(library, compiler):
    inputs = {'compiler': digest(compiler)}
    inputs['adapter/build.py'] = digest(Path(__file__))
    inputs['adapter/process.py'] = digest(Path(process.__file__))
    inputs['adapter/../omega-component/elf.py'] = digest(Path(__file__).parent.parent / 'omega-component/elf.py')
    for folder in ('src', 'platform/native', 'tests/native_component'):
        for path in sorted((library / folder).rglob('*.omg')):
            if 'build' not in path.relative_to(library / folder).parts[:-1]:
                inputs[path.relative_to(library).as_posix()] = digest(path)
    return inputs


def prepare(compiler=None):
    library = LIBRARY.resolve()
    required = ('src', 'platform/native', 'tests/native_component/layout/main.omg',
                'tests/native_component/frame/main.omg')
    missing = [name for name in required if not (library / name).exists()]
    if missing:
        raise ValueError(f'Cathedral checkout is missing Omega UI source at {library}; '
                         f'missing: {", ".join(missing)}. Restore the checked-in distribution '
                         'library. See tools/omega-ui/README.md.')
    compiler = select_compiler(compiler)
    try:
        profile = json.loads(subprocess.check_output([str(compiler), '--build-profile'], text=True,
                                                    stderr=subprocess.PIPE))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        raise ValueError(f'cannot read build profile from Omega compiler {compiler}; '
                         'use the supported compiler in tools/omega-ui/README.md#build') from error
    if profile.get('opt_level') not in ('1', '2', '3', 's', 'z'):
        raise ValueError('an optimized Omega compiler is required')
    inputs = fingerprint(library, compiler)
    receipt = OUTPUT / 'receipt.json'
    if receipt.exists():
        saved = json.loads(receipt.read_text())
        if saved['library'] == str(library) and saved['inputs'] == inputs and all((OUTPUT / name).exists() and digest(OUTPUT / name) == value for name, value in saved['outputs'].items()):
            return OUTPUT
    OUTPUT.mkdir(parents=True, exist_ok=True)
    manifest = {'inputs': inputs, 'library': str(library), 'compiler': str(compiler), 'profile': profile, 'exports': {}, 'outputs': {}}
    assembly = []
    rust = []
    for name in ('layout', 'frame'):
        project = library / 'tests/native_component' / name
        directory = OUTPUT / name
        for step, arguments in (
            ('review', ['update', '--project', str(project), '--target', 'linux_x86_64']),
            ('compile', ['--target', 'linux_x86_64', '--build-dir', str(directory), str(project / 'main.omg')]),
        ):
            with (OUTPUT / f'{name}-{step}.log').open('w', encoding='utf-8') as log:
                result = process.run([str(compiler), *arguments], cwd=project, stdout=log, stderr=subprocess.STDOUT)
            if result.returncode:
                raise RuntimeError(f'{name} {step} failed; see {OUTPUT / (name + "-" + step + ".log")}')
        dependencies = Elf((directory / 'omega-program').read_bytes(), 2).needed()
        dependency = dependencies[0].decode() if len(dependencies) == 1 else ''
        filename = dependency.removeprefix('$ORIGIN/')
        if not filename or Path(filename).name != filename or dependency != '$ORIGIN/' + filename:
            raise ValueError('the probe must select one local component')
        source = directory / filename
        body, (symbol, entry) = component(source.read_bytes())
        blob = OUTPUT / f'{name}.bin'
        blob.write_bytes(body)
        assembly.extend([
            f'.section .text.omega_ui_{name},"ax",@progbits', '.balign 16',
            f'.global omega_ui_{name}_base', f'omega_ui_{name}_base:',
            f'.incbin "{blob.as_posix()}"', f'.global omega_ui_{name}',
            f'.set omega_ui_{name}, omega_ui_{name}_base + {entry}',
        ])
        manifest['exports'][name] = {'entry': entry, 'symbol': symbol, 'bytes': len(body), 'component': source.name,
                                     'component_sha256': digest(source), 'description_sha256': digest(source.with_suffix('.so.component'))}
        manifest['outputs'][blob.name] = digest(blob)
        rust.extend([f'pub const {name.upper()}_ENTRY: usize = {entry};',
                     f'pub static {name.upper()}_BYTES: &[u8] = include_bytes!(r#"{blob.as_posix()}"#);'])
    assembly_path = OUTPUT / 'component.S'
    assembly_path.write_text('\n'.join(assembly) + '\n')
    manifest['outputs'][assembly_path.name] = digest(assembly_path)
    rust_path = OUTPUT / 'component.rs'
    rust_path.write_text('\n'.join(rust) + '\n')
    manifest['outputs'][rust_path.name] = digest(rust_path)
    if fingerprint(library, compiler) != inputs:
        raise RuntimeError('native UI sources or compiler changed during compilation; retry with stable inputs')
    receipt.write_text(json.dumps(manifest, indent=2) + '\n')
    return OUTPUT


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path)
    parser.add_argument('--cargo', action='store_true', help='emit exact Cargo rebuild inputs before the output directory')
    arguments = parser.parse_args()
    try:
        directory = prepare(arguments.compiler)
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Omega UI preparation failed: {error}\n')
    if arguments.cargo:
        receipt = json.loads((directory / 'receipt.json').read_text())
        library = Path(receipt['library'])
        paths = [Path(receipt['compiler']), library / 'src', library / 'platform/native',
                 library / 'tests/native_component']
        paths += [Path(__file__).parent / name.removeprefix('adapter/') if name.startswith('adapter/') else library / name
                  for name in receipt['inputs'] if name != 'compiler']
        for path in paths:
            print(f'cargo:rerun-if-changed={path}')
    print(directory, flush=True)
