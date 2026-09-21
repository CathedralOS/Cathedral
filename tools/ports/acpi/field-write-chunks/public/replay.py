#!/usr/bin/env python3
"""Replay the original public Rust observations, retaining the rebuilt identity."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
BASE = HERE.parent.parent/'field-writes'
sys.path.insert(0, str(BASE))
import reference


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inputs():
    names = ['reference.py', 'reference.rs', 'aml_encoding.py', 'vectors.py',
             'geometry_vectors.py', 'reference.Cargo.lock', 'reference-verification.json']
    paths = [BASE/name for name in names] + [Path(__file__).resolve(), HERE.parent/'toolchain.json']
    return {str(path.relative_to(ROOT)):sha(path) for path in paths}


def verify(record):
    prior = json.loads((BASE/'reference-verification.json').read_text())
    assert record['pin'] == prior['pin'] == reference.PIN
    assert record['input_sha256'] == inputs()
    assert record['rows'] == prior['rows']
    assert record['upstream_sha256'] == prior['upstream_sha256']
    expected = reference.cases()
    assert len(record['rows']) == len(expected)
    for row, case in zip(record['rows'], expected):
        assert {key:row[key] for key in case} == case
        assert row['aml_hex'] == reference.table(case['offset'],case['length'],case['flags'],case['region'],case['source']).hex()
        assert row['observed']['forbidden_calls'] == '0'
    assert record['build_exit_code'] == 0 and record['source_unchanged']
    print('PASS',len(expected),'exact public observations; rebuilt binary identity retained')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository',type=Path,default=ROOT)
    parser.add_argument('--toolchain',type=Path)
    parser.add_argument('--target-dir',type=Path,default=Path('/tmp/cathedral-field-write-chunks-public-target'))
    parser.add_argument('--record',type=Path,default=HERE/'verification.json')
    parser.add_argument('--verify',action='store_true')
    parser.add_argument('--require-binary',action='store_true')
    args=parser.parse_args()
    if args.verify:
        record=json.loads(args.record.read_text())
        verify(record)
        if args.require_binary:
            assert sha(Path(record['binary_path'])) == record['binary_sha256']
        return
    upstream=args.repository.resolve()/'reference_code/rust-osdev/acpi'
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=upstream,text=True).strip()==reference.PIN
    source={str(path.relative_to(upstream)):sha(path) for path in sorted((upstream/'src').rglob('*.rs'))}
    for name in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']:
        source[name]=sha(upstream/name)
    for name,digest in source.items():
        assert hashlib.sha256(subprocess.check_output(['git','show',reference.PIN+':'+name],cwd=upstream)).hexdigest()==digest
    toolchain=args.toolchain or Path(json.loads((HERE.parent/'toolchain.json').read_text())['rust_toolchain'])
    cargo,rustc=toolchain/'cargo',toolchain/'rustc'
    environment=dict(os.environ,RUSTC=str(rustc))
    before=inputs()
    with tempfile.TemporaryDirectory(prefix='cathedral-write-chunk-public-') as directory:
        work=Path(directory)
        manifest='[package]\nname="cathedral-field-writes-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={path='+json.dumps(str(upstream))+'}\n[[bin]]\nname="reference"\npath='+json.dumps(str(BASE/'reference.rs'))+'\n'
        (work/'Cargo.toml').write_text(manifest)
        (work/'Cargo.lock').write_bytes((BASE/'reference.Cargo.lock').read_bytes())
        command=[str(cargo),'build','--release','--offline','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(args.target_dir)]
        built=subprocess.run(command,capture_output=True,text=True,env=environment)
        assert built.returncode==0,built.stdout+built.stderr
        binary=args.target_dir/'release/reference'
        rows=[]
        for case in reference.cases():
            data=reference.table(case['offset'],case['length'],case['flags'],case['region'],case['source'])
            path=work/'table.aml'
            path.write_bytes(data)
            run=subprocess.run([str(binary),str(path),str(case['revision'])],capture_output=True,text=True,timeout=5,check=True)
            observed=dict(line.split('\t',1) for line in run.stdout.splitlines())
            rows.append(dict(**case,aml_hex=data.hex(),observed=observed))
    record=dict(stage='replay of unchanged public pinned Rust Field write observations; no new API/native Omega claim',
                pin=reference.PIN,input_sha256=before,upstream_sha256=source,rows=rows,source_unchanged=before==inputs(),
                binary_path=str(binary.resolve()),binary_sha256=sha(binary),
                cargo_sha256=sha(cargo),rustc_sha256=sha(rustc),
                cargo_version=subprocess.check_output([str(cargo),'--version'],text=True).strip(),
                rustc_version=subprocess.check_output([str(rustc),'-vV'],text=True).strip(),
                manifest=manifest,build_command=command,build_exit_code=built.returncode,build_output=built.stdout+built.stderr)
    args.record.parent.mkdir(parents=True,exist_ok=True)
    args.record.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
    verify(record)


if __name__=='__main__':
    main()
