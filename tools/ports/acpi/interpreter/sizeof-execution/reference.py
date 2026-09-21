#!/usr/bin/env python3
"""Actual pinned public load_table/evaluate observations for query AML."""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import fixtures

HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
SHARED=ROOT/'tools/ports/acpi/aml-public-execution'
LOCK=ROOT/'tools/ports/acpi/field-access/reference.Cargo.lock'

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def source_inputs():return [HERE/'reference.py',HERE/'fixtures.py',HERE.parent/'execution/fixtures.py',SHARED/'src/main.rs',LOCK]
def public_cases():return [row for row in fixtures.cases() if row['public'] or row['name']=='type_predefined_scope']

def validate_observed(row,observed):
    assert observed['forbidden_calls']=='0',(row['name'],observed)
    assert observed['created_mutexes']==('2' if row['name'].endswith('_mutex') else '1'),(row['name'],observed)
    assert observed.get('load')=='ok',(row['name'],observed)
    assert observed.get('panic')==('true' if row['name']=='type_root' else None),(row['name'],observed)
    for name,expected in row['public_after'].items():assert observed['after:'+name]==expected,(row['name'],observed)
    gaps={
        'type_created_scope':('typeless_scope_missing','error:ObjectDoesNotExist(AmlName([Segment("SCP0")]))'),
        'type_predefined_scope':('typeless_scope_missing','error:ObjectDoesNotExist(AmlName([Root, Segment("_SB_")]))'),
        'type_root':('root_scope_panic',None),
        'nearest_scope_wins':('scope_shadow_skipped','integer:1'),
        'type_truncated':('incomplete_operand_uninitialized','uninitialized'),
        'size_truncated':('incomplete_operand_uninitialized','uninitialized'),
    }
    if row['name'] in gaps:
        category,expected=gaps[row['name']]
        assert observed.get('result')==expected,(row['name'],observed)
    elif row['expected'] is None:
        category='explicit_rejection';assert observed['result'].startswith('error:'),(row['name'],observed)
    else:
        category='agreement';assert observed['result']=='integer:'+str(row['expected']),(row['name'],observed)
    return category

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--write',action='store_true');p.add_argument('--cargo',type=Path,default=Path('cargo'));p.add_argument('--target-dir',type=Path,default=Path(tempfile.gettempdir())/'cathedral-sizeof-opcodes-public');args=p.parse_args()
    upstream=ROOT/'reference_code/rust-osdev/acpi'
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=upstream,text=True).strip()==PIN
    paths=sorted((upstream/'src').rglob('*.rs'))+[upstream/name for name in ['Cargo.toml','LICENCE-MIT','LICENCE-APACHE']]
    provenance={str(path.relative_to(upstream)):sha(path) for path in paths}
    for path,digest in provenance.items():assert hashlib.sha256(subprocess.check_output(['git','show',PIN+':'+path],cwd=upstream)).hexdigest()==digest
    before={str(path.relative_to(ROOT)):sha(path) for path in source_inputs()};target=args.target_dir.resolve();observations=[]
    with tempfile.TemporaryDirectory(prefix='sizeof-opcodes-public-') as directory:
        work=Path(directory)
        (work/'Cargo.toml').write_text(f'[package]\nname="cathedral-field-access-reference"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nacpi={{path="{upstream}"}}\n[[bin]]\nname="reference"\npath="{SHARED}/src/main.rs"\n')
        (work/'Cargo.lock').write_bytes(LOCK.read_bytes())
        subprocess.run([str(args.cargo),'build','--release','--offline','--locked','--manifest-path',str(work/'Cargo.toml'),'--target-dir',str(target)],check=True)
        binary=target/'release'/('reference.exe' if os.name=='nt' else 'reference')
        for row in public_cases():
            path=work/'table.aml';path.write_bytes(bytes.fromhex(row['aml_hex']))
            run=subprocess.run([str(binary),str(path),str(row['revision']),'',*row['public_after']],capture_output=True,text=True,check=True,timeout=5)
            observed=dict(line.split('\t',1) for line in run.stdout.splitlines())
            category=validate_observed(row,observed)
            observations.append(dict(name=row['name'],aml_hex=row['aml_hex'],revision=row['revision'],expected=row['expected'],profile_outcome=row['error'],category=category,observed=observed))
        record=dict(stage='actual public pinned Interpreter load_table/evaluate; unchanged shared Rust harness; zero device/time/debug callbacks',pin=PIN,
                    upstream_sha256=provenance,source_sha256=before,binary_sha256=sha(binary),counts=dict(sorted(Counter(row['category'] for row in observations).items())),rows=observations)
    assert before=={str(path.relative_to(ROOT)):sha(path) for path in source_inputs()},'inputs changed during public observation'
    path=HERE/'reference-verification.json'
    if args.write:path.write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
    else:assert json.loads(path.read_text())==record
    print('PASS',len(observations),'public SizeOf/query opcode observations',record['counts'])
if __name__=='__main__':main()
