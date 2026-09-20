#!/usr/bin/env python3
"""Check the fixed UEFI byte fixture and execute its Omega semantic tests."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import tempfile
import zlib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def expected_image():
    vectors = json.loads((ROOT / 'source/contracts/uefi/raw/tables.vectors.json').read_text())
    facts = {name: row['value'] for name, row in vectors['measurements'].items()}
    assert facts['BootServices.size'] == 376
    assert facts['Header.size'] == 24
    data = bytearray(facts['BootServices.size'])
    struct.pack_into('<Q', data, facts['Header.signature.offset'], facts['BOOT_SERVICES_SIGNATURE'])
    struct.pack_into('<I', data, facts['Header.revision.offset'], 0x00020064)
    struct.pack_into('<I', data, facts['Header.size.offset'], len(data))
    struct.pack_into('<Q', data, facts['BootServices.stall.offset'], 0xcafe0001)
    struct.pack_into('<Q', data, facts['BootServices.get_next_monotonic_count.offset'], 0xcafe0002)
    crc = zlib.crc32(data)
    struct.pack_into('<I', data, facts['Header.crc.offset'], crc)
    return bytes(data), crc


def host_check():
    data, crc = expected_image()
    assert (HERE / 'boot-services.bin').read_bytes() == data, 'binary fixture differs from independent image'
    golden = (HERE / 'golden.omg').read_text()
    entries = [(int(i), int(value)) for i, value in re.findall(r'image\.bytes\[(\d+)\] == (\d+)', golden)]
    assert entries == list(enumerate(data)), 'Omega oracle must cover every byte exactly once, in order'
    chunks = re.findall(r'^machine (chunk\d+)\(', golden, re.M)
    calls = re.findall(r'\b(chunk\d+)\(image\)', golden)
    assert chunks == calls, 'every byte-check chunk must be called once'
    assert crc == 0x1c2abadf
    print(f'Independent fixed image: {len(data)} bytes; CRC32={crc:08x}; all Omega oracle bytes match.', flush=True)


def negative_check(compiler):
    marker = 'image.bytes[0] == 66'
    golden = (HERE / 'golden.omg').read_text()
    assert golden.count(marker) == 1
    with tempfile.TemporaryDirectory(prefix='cathedral-uefi-images-negative-') as directory:
        fixture = Path(directory)
        for source in HERE.glob('*.omg'):
            text = source.read_text()
            if source.name == 'build.omg':
                for relative in ('../../../source/contracts/uefi/raw', '../../../source/libraries/uefi'):
                    assert relative in text
                    text = text.replace(relative, str((HERE / relative).resolve()))
            if source.name == 'golden.omg':
                text = text.replace(marker, 'image.bytes[0] == 67')
            (fixture / source.name).write_text(text)
        result = subprocess.run([str(compiler), '--check', str(fixture / 'main.omg')],
                                cwd=ROOT, capture_output=True, text=True)
        output = result.stdout + result.stderr
        if result.returncode == 0 or 'cannot prove requires contract' not in output or '1 == 0' not in output:
            raise SystemExit('Mutated byte expectation did not fail the unchanged success assertion:\n' + output)
    print('Mutated golden byte evaluated false; unchanged final assertion rejected result 1.', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega', type=Path, default=ROOT.parent / 'Omega/target/release/omega')
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--host-only', action='store_true')
    mode.add_argument('--negative-only', action='store_true', help='host audit and mutation control; omit positive compilation')
    args = parser.parse_args()
    host_check()
    if args.host_only:
        return
    compiler = args.omega.resolve()
    print('Omega binary SHA-256:', hashlib.sha256(compiler.read_bytes()).hexdigest(), flush=True)
    if not args.negative_only:
        subprocess.run([str(compiler), '--check', str(HERE / 'main.omg')], cwd=ROOT, check=True)
    negative_check(compiler)
    print('Semantic byte-image fixtures only; native ABI compatibility and firmware calls are not tested.')


if __name__ == '__main__':
    main()
