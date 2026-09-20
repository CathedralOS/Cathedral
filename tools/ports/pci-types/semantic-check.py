#!/usr/bin/env python3
"""Run pure PCI fixtures through Omega semantic evaluation, then mutate each group."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
MUTATIONS = {
    'address': ('address_segment(full.value) == 65535', 'address_segment(full.value) == 65534'),
    'snapshot': ('last.value == 0x78563412', 'last.value == 0x78563413'),
    'header': ('header.vendor_id == 0x8086', 'header.vendor_id == 0x8087'),
    'bridge': ('bridge.buses.primary == 2', 'bridge.buses.primary == 1'),
    'register-class': ('command.raw == 0x8005', 'command.raw == 0x8004'),
    'bar': ('probe.operations[5].value == 0x3450000c', 'probe.operations[5].value == 0x34500000'),
    'capability': ('cycle.status == 4', 'cycle.status == 3'),
    'extended': ('first.next_offset == 4092', 'first.next_offset == 4088'),
    'msi': ('control.operations[0].value == 0x1a6', 'control.operations[0].value == 0x186'),
    'msix': ('table.bytes == 32768', 'table.bytes == 32767'),
}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--omega', type=Path, default=ROOT.parent/'Omega/target/release/omega')
    args = parser.parse_args()
    compiler = args.omega.resolve()
    print('Omega binary SHA-256:', hashlib.sha256(compiler.read_bytes()).hexdigest(), flush=True)
    subprocess.run([str(compiler), '--check', str(HERE/'main.omg')], cwd=ROOT, check=True)
    source = (HERE/'main.omg').read_text()
    build = (HERE/'build.omg').read_text().replace('../../../source/libraries/pci', str(ROOT/'source/libraries/pci'))
    with tempfile.TemporaryDirectory(prefix='cathedral-pci-semantic-') as temporary:
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
    print('All 10 groups passed; all 10 body-mutating negative controls rejected.')
    print('Semantic evaluation only: native execution and physical PCI access NOT RUN.')

if __name__ == '__main__':
    main()
