"""Link a checked Omega process and the reference kernel's native entry adapter.

The compiler emits the component. This tool only assembles the reviewed ABI
adapter and links an ELF at the kernel's existing userspace address. No Rust
application, Linux startup, dynamic loader, or host libc enters the image.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/omega-component'))
spec = importlib.util.spec_from_file_location('omega_component_build', ROOT / 'tools/omega-component/build.py')
component_build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(component_build)
from elf import Elf
from locking import exclusive
import manifest

ABI = ROOT / 'source/platform/libraries/user-runtime/arch/x86_64'


def validate(data):
    elf = Elf(data, 2)
    if elf.needed() or elf.dynamic is not None:
        raise ValueError('standalone process must not require a dynamic loader')
    entry, = elf.unpack('<Q', 24)
    if entry != 0x8000000000 or not any(
            base == entry and flags == 5 and files > 0
            for _, base, files, _, flags in elf.loads):
        raise ValueError('process entry must be executable at the reference kernel image base')
    if any(base % 4096 or flags not in (4, 5) for _, base, _, _, flags in elf.loads):
        raise ValueError('prototype process loads must be page-aligned and read-only')
    return elf


def prepare(source, compiler=None):
    component, _ = component_build.prepare(source, compiler)
    output = ROOT / 'build/omega-user' / source.resolve().relative_to(ROOT / 'source')
    with exclusive(component / '.prepare.lock'), exclusive(output / '.prepare.lock'):
        output.mkdir(parents=True, exist_ok=True)
        clang, linker = shutil.which('clang'), shutil.which('ld.lld')
        if not clang or not linker:
            raise ValueError('LLVM clang and ld.lld are required to assemble the native entry adapter')
        inputs = [Path(__file__), ABI / 'entry.S', ABI / 'user.ld', component / 'component.S',
                  component / 'app.bin', component / 'receipt.json', Path(manifest.__file__)]
        declaration = source / 'process.json'
        declared_bytes = declaration.read_bytes() if declaration.is_file() else None
        record = manifest.encode(json.loads(declared_bytes)) if declared_bytes is not None else None
        if record is not None:
            inputs.append(declaration)
        hashes = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
        if declared_bytes is not None:
            hashes[str(declaration)] = hashlib.sha256(declared_bytes).hexdigest()
        for source_file, object_name in ((ABI / 'entry.S', 'entry.o'), (component / 'component.S', 'program.o')):
            subprocess.run([clang, '--target=x86_64-unknown-none', '-c', str(source_file),
                            '-o', str(output / object_name)], check=True)
        objects = [output / 'entry.o', output / 'program.o']
        if record is not None:
            (output / 'imports.bin').write_bytes(record)
            imports_source = output / 'imports.S'
            imports_source.write_bytes(('.section .cathedral.imports,"a",@progbits\n.balign 8\n'
                                       f'.incbin "{(output / "imports.bin").as_posix()}"\n').encode())
            subprocess.run([clang, '--target=x86_64-unknown-none', '-c', str(imports_source),
                            '-o', str(output / 'imports.o')], check=True)
            objects.append(output / 'imports.o')
        image = output / 'program.pending.elf'
        subprocess.run([linker, '-m', 'elf_x86_64', '-T', str(ABI / 'user.ld'),
                        '--no-undefined', '-o', str(image), *map(str, objects)], check=True)
        image_bytes = image.read_bytes()
        elf = validate(image_bytes)
        if record is not None and not any(flags == 4 and files >= len(record)
                and image_bytes[offset:offset + len(record)] == record
                for offset, _, files, _, flags in elf.loads):
            raise ValueError('process imports must begin a read-only load segment')
        if declaration.is_file() != (declared_bytes is not None) or any(
                hashlib.sha256(p.read_bytes()).hexdigest() != hashes[str(p)] for p in inputs):
            raise RuntimeError('entry or component changed during linking; retry')
        final = output / 'program.elf'
        image.replace(final)
        (output / 'receipt.json').write_text(json.dumps({
            'inputs': hashes,
            'component': json.loads((component / 'receipt.json').read_text(encoding='utf-8')),
            'elf_sha256': hashlib.sha256(final.read_bytes()).hexdigest(),
            'clang': subprocess.check_output([clang, '--version'], text=True).splitlines()[0],
            'linker': subprocess.check_output([linker, '--version'], text=True).strip(),
        }, indent=2) + '\n', encoding='utf-8')
        return final


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path, help='Omega native consumer selecting runtime::entry::Program')
    parser.add_argument('--compiler', type=Path)
    args = parser.parse_args()
    try:
        print(prepare(args.source, args.compiler))
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Omega process preparation failed: {error}\n')
