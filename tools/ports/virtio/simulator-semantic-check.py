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
    'negotiation': ('ready.session.status == 15', 'ready.session.status == 11'),
    'split': ('done1.completion.bytes == 64', 'done1.completion.bytes == 63'),
    'packed': ('next.queue.descriptors[0].flags == 32768', 'next.queue.descriptors[0].flags == 128'),
    'counter': ('submit.queue.produced == 0', 'submit.queue.produced == 65535'),
    'notification': ('!not_crossed.notification', 'not_crossed.notification'),
    'validation': ('reset.inflight == 0', 'reset.inflight == 1'),
}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega', type=Path, default=ROOT.parent/'Omega/target/release/omega')
    args = parser.parse_args()
    compiler = args.omega.resolve()
    print('Omega binary SHA-256:', hashlib.sha256(compiler.read_bytes()).hexdigest(), flush=True)
    subprocess.run([str(compiler), '--check', str(HERE/'simulator/main.omg')], cwd=ROOT, check=True)
    source = (HERE/'simulator/main.omg').read_text()
    build = (HERE/'simulator/build.omg').read_text().replace('../../../../source/libraries/virtio', str(ROOT/'source/libraries/virtio'))
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
    print('All 6 simulator groups passed; all 6 body-mutating negative controls rejected.')
    print('Semantic evaluation only: native execution and DMA and device access NOT RUN.')

if __name__ == '__main__':
    main()
