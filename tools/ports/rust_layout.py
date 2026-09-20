"""Read measured integer constants from a Rust cross-target LLVM artifact.

This is upstream layout evidence only; it does not observe Omega output.
"""
from pathlib import Path
import re
import subprocess
import tempfile


def decode_bytes(encoded):
    result = bytearray()
    while encoded:
        if encoded.startswith('\\\\'):
            result.append(92)
            encoded = encoded[2:]
        elif encoded.startswith('\\'):
            result.append(int(encoded[1:3], 16))
            encoded = encoded[3:]
        else:
            result.append(ord(encoded[0]))
            encoded = encoded[1:]
    return bytes(result)


def measure(manifest, source, symbol, crate):
    with tempfile.TemporaryDirectory(prefix='cathedral-upstream-layout-') as directory:
        subprocess.run(['cargo', 'rustc', '--locked', '--manifest-path', str(manifest),
                        '--target', 'x86_64-unknown-uefi', '--target-dir', directory,
                        '--lib', '--', '--emit=llvm-ir'], check=True)
        matches = list(Path(directory).glob(f'x86_64-unknown-uefi/debug/deps/{crate}-*.ll'))
        if len(matches) != 1:
            raise ValueError('expected one upstream LLVM artifact')
        artifact = matches[0].read_bytes()
        text = artifact.decode()
        if 'target triple = "x86_64-unknown-windows-msvc"' not in text:
            raise ValueError('unexpected LLVM target for x86_64-unknown-uefi')
        constant = re.search('@' + re.escape(symbol) + r' = .*?c"([^"\n]*)"', text)
        if constant is None:
            raise ValueError(f'exported measurement constant absent: {symbol}')
        data = decode_bytes(constant[1])
        keys = re.findall(r' as u64, // (\S+)', source.read_text())
        if len(set(keys)) != len(keys) or len(data) != len(keys) * 8:
            raise ValueError('measurement key/byte count differs or duplicate keys')
        return {key: int.from_bytes(data[index * 8:index * 8 + 8], 'little')
                for index, key in enumerate(keys)}, artifact
