#!/usr/bin/env python3
"""Read-only historical input validation, not fresh execution of either suite."""
import hashlib,json,tarfile
from pathlib import Path
HERE=Path(__file__).resolve().parent
archive=HERE/'pre-scalar-canonical.tar.gz';manifest=json.loads((HERE/'manifest.json').read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha(archive.read_bytes())==manifest['archive_sha256']
with tarfile.open(archive,'r:gz')as tar:
 assert all(m.isfile()for m in tar.getmembers());files={m.name:tar.extractfile(m).read()for m in tar.getmembers()}
assert {p:sha(d)for p,d in files.items()}==manifest['files']
prefix='tools/ports/acpi/aml/named-value-store/';checked=json.loads(files[prefix+'verification.json']);public=json.loads(files[prefix+'reference-verification.json'])
assert checked['positive_count']==checked['control_count']==305 and len(checked['constant_proofs'])==6
assert len(public['rows'])==297
for p,want in checked['input_sha256'].items():assert sha(files[p])==want,p
for p,want in public['production_mapping_sha256'].items():assert sha(files[p])==want,p
for p,want in public['source_sha256'].items():assert sha(files[prefix+p])==want,p
for p,want in public['upstream_sha256'].items():assert sha(files['reference_code/rust-osdev/acpi/'+p])==want,p
for r in [checked,public]:assert sha(r['build_text'].encode())==r['build_sha256']
assert checked['execution_root']==public['execution_root']=='/Users/zcanann/Documents/projects/Cathedral'
print('PASS historical 305 checked/3 const +297 public receipt input closures,',len(files),'exact files; no fresh execution claim')
