"""Capture each compiled role before Cargo reuses its package output path.

Keep Cargo's debug artifact intact; only loadable ELF bytes enter the boot bundle.
Content-addressed paths also change the kernel build's watched environment when
switching cached feature sets, even if Cargo restores an older file timestamp.
"""
import hashlib
from pathlib import Path
from elf_trim import loadable_bytes


def capture(source, directory):
    data = loadable_bytes(source)
    directory = Path(directory)
    directory.mkdir(parents=True, exist_ok=True)
    destination = directory / (hashlib.sha256(data).hexdigest() + '.elf')
    try:
        with destination.open('xb') as target:
            target.write(data)
    except FileExistsError:
        if destination.read_bytes() != data:
            raise RuntimeError(f'Artifact snapshot differs from its digest: {destination}')
    return destination.resolve()
