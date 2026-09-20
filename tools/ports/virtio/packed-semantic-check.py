#!/usr/bin/env python3
"""Run pure VirtIO fixtures through Omega semantic evaluation, then mutate each group."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
MUTATIONS = {
    'cursor': ('delta.value == 2', 'delta.value == 1'),
    'flags': ('avail1.value == 131', 'avail1.value == 130'),
    'event': ('all.value.desc.raw == 65535', 'all.value.desc.raw == 32767'),
    'notify': ('!notify(next.value, 3, true, old, new).notify', 'notify(next.value, 3, true, old, new).notify'),
    'descriptor': ('bytes[12] == 0x54', 'bytes[12] == 0x55'),
}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega', type=Path, default=ROOT.parent/'Omega/target/release/omega')
    args = parser.parse_args()
    compiler = args.omega.resolve()
    print('Omega binary SHA-256:', hashlib.sha256(compiler.read_bytes()).hexdigest(), flush=True)
    subprocess.run([str(compiler), '--check', str(HERE/'packed/main.omg')], cwd=ROOT, check=True)
    source = (HERE/'packed/main.omg').read_text()
    build = (HERE/'packed/build.omg').read_text().replace('../../../../source/libraries/virtio', str(ROOT/'source/libraries/virtio'))
    with tempfile.TemporaryDirectory(prefix='cathedral-virtio-semantic-') as temporary:
        folder = Path(temporary)
        (folder/'build.omg').write_text(build)
        for name, (old, new) in MUTATIONS.items():
            if source.count(old) != 1:
                raise SystemExit(f'{name}: mutation marker is not unique')
            (folder/'main.omg').write_text(source.replace(old, new))
            result = subprocess.run([str(compiler), '--check', str(folder/'main.omg')], cwd=ROOT, capture_output=True, text=True)
            output = result.stdout + result.stderr
            if result.returncode == 0 or 'cannot prove requires contract' not in output or '1 == 0' not in output:
                raise SystemExit(f'{name}: changed behavior did not compute failure:\n{output}')
            print(f'{name}: changed expected behavior computed failure and was rejected', flush=True)
    print('All 5 packed groups passed; all 5 body-mutating negative controls rejected.')
    print('Semantic evaluation only: native execution and DMA and device access NOT RUN.')

if __name__ == '__main__':
    main()
