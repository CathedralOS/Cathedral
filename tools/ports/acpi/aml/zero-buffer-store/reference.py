#!/usr/bin/env python3
"""Actual pinned public Object observations using the existing component probe."""
import argparse
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
PROBE = HERE.parent/'buffer-target-values'
PIN = '257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def expected_rows():
    rows = []
    for kind, value, output in [
        ('Integer', '0', bytes(8)),
        ('Integer', '4294967296', (1 << 32).to_bytes(8, 'little')),
        ('Integer', '18446744073709551615', bytes([255])*8),
        ('String', '', b''),
        ('String', '41', b'A'),
        ('String', '41'*256, b'A'*256),
    ]:
        rows.append(dict(kind=kind, argument=value, target_extent=0,
                         output='result\tbuffer:'+str(len(output))+':'+output.hex()+'\n',
                         classification='zero-extent-match' if not output else 'pin-resizes-existing-zero-buffer'))
    return rows


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--checkout', type=Path, default=ROOT/'reference_code/rust-osdev/acpi')
    parser.add_argument('--record', type=Path, default=HERE/'reference-verification.json')
    parser.add_argument('--verify-record', action='store_true')
    args = parser.parse_args()
    upstream = args.checkout.resolve()
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=upstream,text=True).strip() == PIN
    files = [*sorted((upstream/'src').rglob('*.rs')),
             *[upstream/n for n in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']]]
    source = {str(p.relative_to(upstream)): sha(p) for p in files}
    for name, digest in source.items():
        assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+name],cwd=upstream)).hexdigest() == digest
    input_paths = [Path(__file__), PROBE/'reference.rs', PROBE/'reference.Cargo.lock',
                   ROOT/'source/libraries/acpi/aml/named_value_store.omg',
                   ROOT/'source/libraries/acpi/aml/buffer_target_values.omg']
    inputs = {str(p.relative_to(ROOT)): sha(p) for p in input_paths}
    target = Path('/tmp/cathedral-zero-buffer-store-public')
    if args.verify_record:
        receipt = json.loads(args.record.read_text())
        assert receipt['pin'] == PIN and receipt['execution_root'] == str(ROOT)
        assert receipt['checkout'] == str(upstream) and receipt['upstream_sha256'] == source
        assert receipt['input_sha256'] == inputs and receipt['rows'] == expected_rows()
        assert receipt['binary_sha256'] == sha(target/'release/reference')
        print('PASS 6 public observations; exact current inputs and probe binary')
        return
    with tempfile.TemporaryDirectory(prefix='zero-buffer-store-public-') as directory:
        work = Path(directory)
        manifest = ('[package]\nname="cathedral-buffer-target-values-reference"\nversion="0.1.0"\nedition="2024"\n'
                    '[dependencies]\nacpi={path="'+str(upstream)+'"}\n[[bin]]\nname="reference"\npath="'+str(PROBE/'reference.rs')+'"\n')
        (work/'Cargo.toml').write_text(manifest)
        (work/'Cargo.lock').write_bytes((PROBE/'reference.Cargo.lock').read_bytes())
        command = ['cargo','+nightly-2026-09-04','build','--release','--locked',
                   '--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)]
        subprocess.run(command,check=True)
        binary = target/'release/reference'
        rows = []
        for row in expected_rows():
            result = subprocess.run([str(binary),row['kind'],row['argument'],str(row['target_extent'])],capture_output=True,text=True,check=True)
            assert result.stdout == row['output'] and not result.stderr
            rows.append(dict(row, output=result.stdout))
        assert inputs == {str(p.relative_to(ROOT)): sha(p) for p in input_paths}
        assert source == {str(p.relative_to(upstream)): sha(p) for p in files}
        receipt = dict(pin=PIN, execution_root=str(ROOT), checkout=str(upstream), build_text=manifest,
                       build_command=command, probe='actual public Object::replace_with_implicit_casting on ordinary owned objects',
                       upstream_sha256=source,input_sha256=inputs,binary_sha256=sha(binary),rows=rows,
                       notes=['No interpreter, source/target aliasing, unsafe ObjectToken, provider or device calls.',
                              'These observations document the pinned replacement API; its resize behavior differs from primary existing-target conversion geometry.'])
        args.record.write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n')
    print('PASS',len(rows),'actual public pinned zero-destination replacement observations')


if __name__ == '__main__':
    main()
