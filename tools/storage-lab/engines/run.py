"""Build the engine comparison, measure fresh files, and exercise process recovery."""
import csv
import argparse
import hashlib
import io
import json
import platform
import queue
import subprocess
import threading
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
LAB = HERE.parent
ROOT = LAB.parents[1]


def run(*args, **kwargs):
    return subprocess.run(args, cwd=LAB, check=True, text=True, timeout=kwargs.pop('timeout', 300), **kwargs)


def recovery(executable, directory):
    checks = []
    for engine in ('redb', 'sqlite_delete', 'sqlite_wal'):
        for phase in ('staged', 'committed'):
            path = directory / f'crash-{engine}-{phase}.db'
            prefix = [str(executable), 'engines', 'recovery']
            run(*prefix, 'init', engine, str(path), phase, capture_output=True)
            child = subprocess.Popen([*prefix, 'writer', engine, str(path), phase],
                                     cwd=LAB, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                     stderr=subprocess.PIPE, text=True)
            ready = queue.Queue()
            threading.Thread(target=lambda: ready.put(child.stdout.readline()), daemon=True).start()
            try:
                line = ready.get(timeout=30)
                if line.strip() != 'READY':
                    child.kill()
                    child.wait(timeout=10)
                    raise RuntimeError(f'writer failed before gate: {line} {child.stderr.read()}')
                child.kill()
                child.wait(timeout=10)
            finally:
                if child.poll() is None:
                    child.kill()
                    child.wait(timeout=10)
                for stream in (child.stdin, child.stdout, child.stderr):
                    stream.close()
            result = run(*prefix, 'verify', engine, str(path), phase, capture_output=True)
            assert 'VERIFIED' in result.stdout
            checks.append(f'{engine}:{phase}')
        result = run(*prefix, 'full', engine, str(directory / f'full-{engine}.db'), 'unused', capture_output=True)
        assert 'VERIFIED' in result.stdout
        checks.append(f'{engine}:capacity')
    return checks


def compare(executable, destination):
    result = run(str(executable), 'engines', 'measure', str(destination), capture_output=True)
    rows = list(csv.DictReader(io.StringIO(result.stdout)))
    assert len(rows) == 48
    expected = {(e, l, str(c), w) for e in ('redb', 'sqlite_delete', 'sqlite_wal')
                for l in ('flat', 'tree_32') for c in (4096, 16384)
                for w in ('tiny', 'single', 'separate', 'batch')}
    indexed = {(r['engine'], r['layout'], r['chunk_bytes'], r['workload']): r for r in rows}
    assert set(indexed) == expected
    for key, row in indexed.items():
        for metric in ('submitted_bytes', 'transactions', 'records'):
            assert row[metric] == indexed[('redb', *key[1:])][metric]
    checks = recovery(executable, destination)
    (destination / 'measurements.csv').write_bytes(result.stdout.encode('utf-8'))
    return [f'{len(rows)} comparisons', *checks]


def stress(executable, destination):
    result = run(str(executable), 'engines', 'stress', str(destination), capture_output=True)
    (destination / 'checks.txt').write_bytes(result.stdout.encode('utf-8'))
    assert 'PASS:' in result.stdout
    def read(name):
        with (destination / name).open(newline='', encoding='utf-8') as file:
            return list(csv.DictReader(file))
    summary = read('contention-summary.csv')
    assert {(r['bulk_batch'], r['kind']) for r in summary} == {('64', 'small'), ('64', 'bulk'), ('1', 'small'), ('1', 'bulk')}
    assert len(summary) == 4
    assert len(read('contention.csv')) == 193
    assert all(int(r['reader_checks']) > 0 for r in summary)
    admission = read('admission-summary.csv')
    assert {(r['trial'], r['kind']) for r in admission} == {(str(t), k) for t in range(3) for k in ('small', 'bulk')}
    assert len(admission) == 6
    for row in admission:
        assert int(row['requests']) == 64
        assert int(row['peak_bytes']) <= 512 * 1024
        assert int(row['peak_requests']) <= 16
        assert int(row['client_peak_bytes']) <= 256 * 1024
        assert int(row['client_peak_requests']) <= 8
        assert row['final_bytes'] == row['final_requests'] == '0'
    admitted = read('admission.csv')
    assert len(admitted) == 384
    assert Counter((r['trial'], r['kind']) for r in admitted) == {(str(t), k): 64 for t in range(3) for k in ('small', 'bulk')}
    assert all(int(r['charge_bytes']) <= 128 * 1024 and 0 < int(r['objects']) <= 8 for r in admitted)
    faults = read('faults.csv')
    assert {(r['retire'], r['payload_bytes']) for r in faults} == {(a, b) for a in ('false', 'true') for b in ('4096', '2097152')}
    assert {r['kind'] for r in faults} == {'write', 'resize', 'sync'}
    groups = {}
    for row in faults:
        assert row['recovered'] in ('old', 'new')
        key = (row['retire'], row['payload_bytes'], row['operation'])
        groups.setdefault(key, []).append(row['cut'])
    assert all(sorted(cuts) == ['After', 'Before', 'Partial'] for cuts in groups.values())
    reclamation = read('reclamation.csv')
    assert {r['phase'] for r in reclamation} == {'unpinned', 'pinned', 'released', 'recovered'}
    limited = [r for r in reclamation if r['phase'] in ('released', 'recovered')]
    assert len({r['capacity_bytes'] for r in limited}) == 1
    assert all(int(r['extent_bytes']) <= int(r['capacity_bytes']) for r in limited)
    print(result.stdout, end='')
    return result.stdout.splitlines()


def volume(executable, destination):
    result = run(str(executable), 'engines', 'volume', 'prove', str(destination), capture_output=True, timeout=900)
    (destination / 'checks.txt').write_bytes(result.stdout.encode('utf-8'))
    assert 'PASS:' in result.stdout
    with (destination / 'volume-cuts.csv').open(newline='', encoding='utf-8') as file:
        rows = list(csv.DictReader(file))
    assert {r['phase'] for r in rows} == {'format', 'publish', 'shrink'}
    groups = {}
    for row in rows:
        assert row['kind'] in ('write', 'sync'), 'volume must not resize physical device'
        allowed = ('incomplete', 'new') if row['phase'] == 'format' else ('old', 'new')
        assert row['outcome'] in allowed
        groups.setdefault((row['phase'], row['operation']), []).append(row['cut'])
    assert all(sorted(cuts) == ['After', 'Before', 'Partial'] for cuts in groups.values())
    # Exercise the public commands as separate processes, including refusal to
    # replace an existing image and opening a missing path without creating it.
    prefix = [str(executable), 'engines', 'volume']
    path = destination / 'cli.raw'
    run(*prefix, 'format', str(path), '4194304', '1', capture_output=True)
    before = hashlib.sha256(path.read_bytes()).hexdigest()
    refused = subprocess.run([*prefix, 'format', str(path), '4194304', '2'], cwd=LAB, capture_output=True, timeout=30)
    assert refused.returncode != 0 and hashlib.sha256(path.read_bytes()).hexdigest() == before
    inspected = run(*prefix, 'inspect', str(path), '0', '4194304', capture_output=True)
    assert 'ready=true' in inspected.stdout
    assert hashlib.sha256(path.read_bytes()).hexdigest() == before
    run(*prefix, 'open', str(path), '0', '4194304', capture_output=True)
    missing = destination / 'missing.raw'
    refused = subprocess.run([*prefix, 'open', str(missing), '0', '4194304'], cwd=LAB, capture_output=True, timeout=30)
    assert refused.returncode != 0 and not missing.exists()
    print(result.stdout, end='')
    return [*result.stdout.splitlines(), 'VERIFIED: separate-process format/inspect/open and creation refusals']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument('--stress', action='store_true', help='run redb contention, crash-cut and reclamation experiments')
    modes.add_argument('--volume', action='store_true', help='run raw-image volume bootstrap and recovery experiments')
    args = parser.parse_args()
    run('cargo', 'build', '--locked', '--release', '--features', 'engines')
    executable = LAB / 'target' / 'release' / ('storage-layout.exe' if platform.system() == 'Windows' else 'storage-layout')
    mode = 'volume' if args.volume else 'stress' if args.stress else 'comparison'
    base = ROOT / 'build' / {'volume': 'storage-volume', 'stress': 'storage-stress', 'comparison': 'storage-engines'}[mode]
    base.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    destination = base / stamp
    # Rust creates the directory and refuses overwrite. No previous DB is deleted.
    checks = {'volume': volume, 'stress': stress, 'comparison': compare}[mode](executable, destination)
    sources = [*LAB.glob('*.rs'), *LAB.glob('*.toml'), *LAB.glob('chunk_tree/*.rs'),
               *HERE.glob('*.rs'), *HERE.glob('stress/**/*.rs'), *HERE.glob('volume/**/*.rs'), HERE / 'run.py', LAB / 'Cargo.lock']
    context = {
        'utc': datetime.now(timezone.utc).isoformat(),
        'platform': platform.platform(),
        'rustc': run('rustc', '-Vv', capture_output=True).stdout.strip(),
        'engine_runtime': run(str(executable), 'engines', 'versions', capture_output=True).stdout.strip(),
        'revision': run('git', 'rev-parse', 'HEAD', capture_output=True).stdout.strip(),
        'tracked_changes': run('git', 'diff', '--name-only', 'HEAD', capture_output=True).stdout.splitlines(),
        'source_sha256': {p.relative_to(LAB).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources},
        'mode': mode,
        'checks': checks,
        'limits': 'Single host sample; logical extents, not physical allocation. Stress uses a bounded simulated durability model, not exhaustive device failure coverage. Comparison process termination is not power loss. redb IO counters exclude initialization, checkpoint and close; SQLite counters unavailable.',
    }
    (destination / 'context.json').write_bytes((json.dumps(context, indent=2) + '\n').encode('utf-8'))
    print(f'PASS: {context["mode"]}. {destination}')


if __name__ == '__main__':
    main()
