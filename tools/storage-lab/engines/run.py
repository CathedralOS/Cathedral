"""Build the engine comparison, measure fresh files, and exercise process recovery."""
import csv
import hashlib
import io
import json
import platform
import queue
import subprocess
import threading
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
LAB = HERE.parent
ROOT = LAB.parents[1]


def run(*args, **kwargs):
    return subprocess.run(args, cwd=LAB, check=True, text=True, timeout=300, **kwargs)


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


def main():
    run('cargo', 'build', '--locked', '--release', '--features', 'engines')
    executable = LAB / 'target' / 'release' / ('storage-layout.exe' if platform.system() == 'Windows' else 'storage-layout')
    base = ROOT / 'build' / 'storage-engines'
    base.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    destination = base / stamp
    # Rust creates the directory and refuses overwrite. No previous DB is deleted.
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
    sources = [*LAB.glob('*.rs'), *LAB.glob('*.toml'), *LAB.glob('chunk_tree/*.rs'),
               *HERE.glob('*.rs'), HERE / 'run.py', LAB / 'Cargo.lock']
    context = {
        'utc': datetime.now(timezone.utc).isoformat(),
        'platform': platform.platform(),
        'rustc': run('rustc', '-Vv', capture_output=True).stdout.strip(),
        'engine_runtime': run(str(executable), 'engines', 'versions', capture_output=True).stdout.strip(),
        'revision': run('git', 'rev-parse', 'HEAD', capture_output=True).stdout.strip(),
        'tracked_changes': run('git', 'diff', '--name-only', 'HEAD', capture_output=True).stdout.splitlines(),
        'source_sha256': {p.relative_to(LAB).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources},
        'recovery_checks': checks,
        'limits': 'Single host sample; logical file extents, not physical media allocation. Process termination is not power loss. redb IO counters exclude initialization, checkpoint and close; SQLite counters unavailable.',
    }
    (destination / 'measurements.csv').write_bytes(result.stdout.encode('utf-8'))
    (destination / 'context.json').write_bytes((json.dumps(context, indent=2) + '\n').encode('utf-8'))
    print(f'PASS: {len(rows)} comparisons and {len(checks)} recovery/capacity checks. {destination}')


if __name__ == '__main__':
    main()
