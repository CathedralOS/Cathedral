#!/usr/bin/env python3
"""Check named layout policy declarations independently; this does not select storage ABI."""
import argparse
from pathlib import Path
import subprocess
import tempfile
ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--omega', type=Path, default=ROOT.parent/'Omega/target/release/omega')
parser.add_argument('--module', choices=['layouts', 'packed_layouts'], default='layouts')
parser.add_argument('--combined', action='store_true', help='reproduce current full-fixture/core-layout trait-resolution failure')
args = parser.parse_args()
with tempfile.TemporaryDirectory(prefix='cathedral-virtio-layout-') as directory:
    folder = Path(directory)
    (folder/'build.omg').write_text((HERE/'build.omg').read_text().replace('../../../source/libraries/virtio', str(ROOT/'source/libraries/virtio')))
    source = (HERE/'main.omg').read_text() if args.combined else 'data Main {}\nmachine Main::main(&mut self) {}\n'
    (folder/'main.omg').write_text('use cathedral_virtio::'+args.module+';\n'+source)
    result = subprocess.run([str(args.omega.resolve()), '--check', str(folder/'main.omg')], cwd=ROOT)
    raise SystemExit(result.returncode)
