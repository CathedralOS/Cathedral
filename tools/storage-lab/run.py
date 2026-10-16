"""Compare flat/tree layouts on identical traces and save measurements/provenance."""
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
    validate(rows)
    destination = ROOT / 'build' / 'storage-lab'
    destination.mkdir(parents=True, exist_ok=True)
    sources = [*HERE.glob('*.rs'), *HERE.glob('chunk_tree/*.rs'),
               *HERE.glob('*.toml'), HERE / 'Cargo.lock', HERE / 'run.py']
    context = {
        'utc': datetime.now(timezone.utc).isoformat(),
        'platform': platform.platform(),
        'processor': platform.processor(),
        'rustc': capture('rustc', '-Vv').strip(),
        'revision': capture('git', 'rev-parse', 'HEAD').strip(),
        'tracked_changes': capture('git', 'diff', '--name-only', 'HEAD').splitlines(),
        'untracked_lab_files': capture('git', 'ls-files', '--others', '--exclude-standard', '.').splitlines(),
        'source_sha256': {p.relative_to(HERE).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(sources)},
        'limits': 'One in-memory sample per case; elapsed time excludes verification and collection. Read columns count unique packed records required with an initially cold cache, not device IO. No engine or durability measurement.',
    }
    (destination / 'measurements.csv').write_bytes(output.encode('utf-8'))
    (destination / 'context.json').write_bytes((json.dumps(context, indent=2) + '\n').encode('utf-8'))
    print(f'Verified {len(rows)} cases. Results: {destination}')


def validate(rows):
    workloads = {
        'tiny_4096x32', 'sequential_10mib', 'copy_100_content_only',
        'patch_1_aligned', 'patch_100_aligned', 'patch_100_boundary',
        'patch_64_keep_history', 'patch_64_latest_only', 'patch_64_batched',
        'local_64_bytes_separate', 'local_64_bytes_batched',
        'read_100_aligned', 'read_100_boundary', 'read_100_tail', 'insert_one_prefix',
    }
    layouts = ('flat', 'tree_32', 'tree_128')
    expected = {(layout, str(chunk), str(inline), name)
                for layout in layouts for chunk in (4096, 16384, 65536)
                for inline in (0, 128) for name in workloads}
    indexed = {(r['layout'], r['chunk_bytes'], r['inline_limit'], r['workload']): r for r in rows}
    assert len(rows) == len(expected) and set(indexed) == expected, 'incomplete/duplicate comparison'
    for key, row in indexed.items():
        reference = indexed[('flat', *key[1:])]
        for metric in ('logical_bytes', 'hashed_payload_bytes', 'new_payload_bytes',
                       'live_payload_bytes', 'live_chunks', 'live_versions',
                       'read_payload_bytes', 'read_payload_records'):
            assert row[metric] == reference[metric], (key, metric, 'payload mismatch')
        if key[-1] in ('patch_64_batched', 'local_64_bytes_batched'):
            separate = 'patch_64_latest_only' if key[-1] == 'patch_64_batched' else 'local_64_bytes_separate'
            baseline = indexed[(*key[:3], separate)]
            for metric in ('live_payload_bytes', 'live_manifest_bytes', 'live_chunks', 'live_versions', 'live_tree_nodes'):
                assert row[metric] == baseline[metric], (key, metric, 'final representation mismatch')


if __name__ == '__main__':
    main()
