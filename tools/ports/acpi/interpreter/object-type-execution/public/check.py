#!/usr/bin/env python3
"""Observe the same encoded ObjectType fixtures through the pinned public API."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import fixtures

ROOT = fixtures.ROOT
PIN = '257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
PROBE = ROOT / 'tools/ports/acpi/aml-public-execution'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--acpi-source', type=Path, default=ROOT/'reference_code/rust-osdev/acpi')
    parser.add_argument('--record', type=Path, default=HERE/'verification.json')
    args = parser.parse_args()
    upstream = args.acpi_source.resolve()
    assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=upstream, text=True).strip() == PIN
    assert not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=no'], cwd=upstream, text=True).strip()
    paths = [Path(__file__), HERE.parent/'fixtures.py', PROBE/'src/main.rs', PROBE/'Cargo.toml', PROBE/'Cargo.lock',
             fixtures.GENERIC/'fixtures.py', fixtures.GENERIC/'decoder_fixtures.py', fixtures.HERE.parent/'execution/fixtures.py']
    before = {str(path.relative_to(ROOT)): sha(path) for path in paths}
    upstream_before = {str(path.relative_to(upstream)): sha(path) for path in sorted((upstream/'src').rglob('*.rs'))}
    upstream_before['Cargo.toml'] = sha(upstream/'Cargo.toml')
    manifest = PROBE.joinpath('Cargo.toml').read_text().replace('../../../../reference_code/rust-osdev/acpi', str(upstream))
    observations, omitted = {}, {}
    with tempfile.TemporaryDirectory(prefix='cathedral-object-type-public-') as directory:
        work = Path(directory)
        (work/'src').mkdir()
        (work/'src/main.rs').write_bytes((PROBE/'src/main.rs').read_bytes())
        (work/'Cargo.toml').write_text(manifest)
        (work/'Cargo.lock').write_bytes((PROBE/'Cargo.lock').read_bytes())
        target = Path('/tmp/cathedral-object-type-public')
        subprocess.run(['cargo', '+nightly-2026-09-04', 'build', '--offline', '--locked', '--release',
                        '--manifest-path', str(work/'Cargo.toml'), '--target-dir', str(target)], check=True, cwd=upstream)
        binary = target/'release/cathedral-acpi-public-execution'
        binary_hash = sha(binary)
        for row in fixtures.execution_cases():
            if row['setup']:
                omitted[row['name']] = 'Direct canonical metadata edit has no corresponding public API fixture.'
                continue
            aml = work/(row['name']+'.aml')
            aml.write_bytes(bytes.fromhex(row['table']))
            command = [str(binary), str(aml), '1' if row['bits'] == 32 else '2', '', *row['after']]
            process = subprocess.run(command, capture_output=True, text=True, timeout=5,
                                     env=dict(os.environ, CATHEDRAL_GENERIC_DESCRIBE='1'))
            assert process.returncode == 0, (row['name'], process.stderr)
            values = dict(line.split('\t', 1) for line in process.stdout.splitlines())
            assert values['forbidden_calls'] == '0' and values['created_mutexes'] == '1', (row['name'], values)
            expected = {'result': 'integer:'+str(row['number']), **{'after:'+key:'integer:'+str(value) for key,value in row['after'].items()}}
            agrees = row['error'] == 'Success' and values.get('load') == 'ok' and all(values.get(key) == value for key,value in expected.items())
            observations[row['name']] = dict(table_hex=row['table'], bits=row['bits'], omega_outcome=row['error'],
                                             omega_expected=expected, observed=values, agrees_on_value_and_state=agrees)
            print(row['name'], json.dumps(values, sort_keys=True), flush=True)
        assert binary_hash == sha(binary)
    assert before == {str(path.relative_to(ROOT)): sha(path) for path in paths}
    assert all(sha(upstream/path) == digest for path,digest in upstream_before.items())
    record = dict(stage='Pinned public Rust Interpreter::load_table/evaluate; no Omega native or hardware claim',
                  upstream_revision=PIN, upstream_source=str(upstream), upstream_sha256=upstream_before,
                  source_sha256=before, source_unchanged=True, binary=str(binary), binary_sha256=binary_hash,
                  build_source=manifest, rustc=subprocess.check_output(['rustc', '+nightly-2026-09-04', '--version'], text=True).strip(),
                  rows=observations, omitted=omitted,
                  agreements=sum(row['agrees_on_value_and_state'] for row in observations.values()))
    args.record.write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
    print(len(observations), 'public observations;', record['agreements'], 'value/state agreements;', len(omitted), 'explicit omissions')


if __name__ == '__main__':
    main()
