#!/usr/bin/env python3
"""Bind the completed migration artifacts after all current receipts verify."""
import hashlib,json,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
subprocess.run(['python3',str(HERE/'verify_record.py')],cwd=ROOT,check=True)
paths=set(p for p in HERE.rglob('*')if p.is_file()and'__pycache__'not in p.parts and p.name!='manifest.json')
paths.update(ROOT/'source/libraries/acpi/aml'/n for n in ['model.omg','values.omg','object_references.omg','build.omg','byte_storage.omg','byte-storage.PORT.md','byte-storage-inventory.json','object-references.PORT.md','PORT.md'])
paths.update(ROOT/n for n in ['source/libraries/acpi/pipeline/program.omg','tools/ports/acpi/aml/fixtures.py','tools/ports/acpi/aml/cases/string-validation.omg','tools/ports/acpi/aml/cases/values-buffer.omg','tools/ports/acpi/pipeline/fixtures.py','tools/ports/acpi/pipeline/cases.json','tools/ports/acpi/pipeline/main.omg'])
current={}
for p in [HERE/'verification.json',HERE/'extras-verification.json',HERE/'const-verification.json',*sorted((HERE/'regressions').glob('*.json'))]:
 for rel,digest in json.loads(p.read_text())['source_sha256'].items():
  if rel in current:assert current[rel]==digest,rel
  current[rel]=digest
source_closure={p:h for p,h in sorted(current.items())if p.startswith('source/')and p.endswith('.omg')}
record={'format':'cathedral-aml-owned-byte-migration-v1','status':'bounded canonical storage kernels and owned Program integration; generic byte opcode execution pending','upstream_pin':'257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5','omega_revision':'eaa7993a23623cd8fabf45350340479c5c9c7879','checked_scenarios':{'main_storage':128,'composition':10,'parser_regression':27,'pipeline_regression':22,'reference_regression':160,'integer_execution_regression':79,'field_syntax_regression':18},'all_changed_body_controls':444,'constant_pairs':8,'rust_observations':{'actual_public_read_clone':13,'exact_private_copy_bits_mirror':8},'source_files_sha256':source_closure,'source_closure_sha256':hashlib.sha256(json.dumps(source_closure,sort_keys=True,separators=(',',':')).encode()).hexdigest(),'artifact_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(paths)},'native_abi_hardware':'not run'}
(HERE/'manifest.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');print('Manifest SHA256',hashlib.sha256((HERE/'manifest.json').read_bytes()).hexdigest());print('Current source closure SHA256',record['source_closure_sha256'])
