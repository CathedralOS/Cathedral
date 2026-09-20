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
    'facts': ('unknown.raw == 255', 'unknown.raw == 254'),
    'requirements': ('Features { low: 0x3801 }', 'Features { low: 0x3800 }'),
    'feature-words': ('selected.value.high == 64', 'selected.value.high == 63'),
    'notifications': ('block_queue_count(0, 2) == 512', 'block_queue_count(0, 2) == 256'),
    'layouts': ('layout_case(257, true, 520, 2064, 2062)', 'layout_case(257, true, 520, 2062, 2062)'),
    'ring-arithmetic': ('index_distance(0, 65535) == 1', 'index_distance(0, 65535) == 0'),
    'ring-views': ('event.offset == 2052', 'event.offset == 2054'),
    'descriptors': ('bytes[0] == 0xef', 'bytes[0] == 0xfe'),
}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega', type=Path, default=ROOT.parent/'Omega/target/release/omega')
    args = parser.parse_args()
    compiler = args.omega.resolve()
    print('Omega binary SHA-256:', hashlib.sha256(compiler.read_bytes()).hexdigest(), flush=True)
    subprocess.run([str(compiler), '--check', str(HERE/'main.omg')], cwd=ROOT, check=True)
    source = (HERE/'main.omg').read_text()
    build = (HERE/'build.omg').read_text().replace('../../../source/libraries/virtio', str(ROOT/'source/libraries/virtio'))
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
    print('All 8 groups passed; all 8 body-mutating negative controls rejected.')
    print('Semantic evaluation only: native execution and DMA and device access NOT RUN.')

if __name__ == '__main__':
    main()
