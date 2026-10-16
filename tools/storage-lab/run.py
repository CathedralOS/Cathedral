"""Run the host chunk-layout baseline and save measurements with their provenance."""
import csv
import hashlib
import io
import json
import platform
import subprocess
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def capture(*command):
    return subprocess.run(command, cwd=HERE, check=True, capture_output=True, text=True).stdout


def main():
    output = capture('cargo', 'run', '--locked', '--release', '--quiet')
    rows = list(csv.DictReader(io.StringIO(output)))
    assert len(rows) == 48, 'incomplete layout comparison'
    assert len({(r['chunk_bytes'], r['inline_limit'], r['workload']) for r in rows}) == 48
    destination = ROOT / 'build' / 'storage-lab'
    destination.mkdir(parents=True, exist_ok=True)
    sources = [*HERE.glob('*.rs'), *HERE.glob('*.toml'), HERE / 'Cargo.lock', HERE / 'run.py']
    context = {
        'utc': datetime.now(timezone.utc).isoformat(),
        'platform': platform.platform(),
        'processor': platform.processor(),
        'rustc': capture('rustc', '-Vv').strip(),
        'revision': capture('git', 'rev-parse', 'HEAD').strip(),
        'tracked_changes': capture('git', 'diff', '--name-only', 'HEAD').splitlines(),
        'untracked_lab_files': capture('git', 'ls-files', '--others', '--exclude-standard', '.').splitlines(),
        'source_sha256': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(sources)},
        'limits': 'One in-memory sample per case; elapsed time excludes readback and collection. No device IO, engine or durability measurement.',
    }
    (destination / 'measurements.csv').write_bytes(output.encode('utf-8'))
    (destination / 'context.json').write_bytes((json.dumps(context, indent=2) + '\n').encode('utf-8'))
    print(f'Verified {len(rows)} cases. Results: {destination}')


if __name__ == '__main__':
    main()
