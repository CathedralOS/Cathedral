#!/usr/bin/env python3
"""Static pinned-source label provenance only: no Rust execution or mirror."""
import hashlib,json,re,subprocess
from pathlib import Path
import fixtures
CANONICAL=Path('/Users/zcanann/Documents/projects/Cathedral');UP=CANONICAL/'reference_code/rust-osdev/acpi';PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
def snapshot():
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=UP,text=True).strip()==PIN
 p=UP/'src/aml/mod.rs';source=p.read_text();start=source.index('        fn resolve_as_string(');end=source.index('\n        let result = match source1.typ()',start);body=source[start:end]
 rows=[]
 for kind,label in fixtures.LABELS.items():
  upstream='OpRegion'if kind=='OperationRegion'else kind
  found=re.search(r'Object::'+upstream+r'\b[^\n]*=> "([^"\n]+)"\.to_string\(\)',body)
  assert found and found.group(1)==label,(kind,found.group(1)if found else None)
  rows.append(dict(canonical_case=kind,upstream_case=upstream,text=label,logical_length=len(label),line=source[:start+found.start()].count('\n')+1))
 sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
 return dict(stage='static primary/pinned label provenance audit only; no Rust execution and no private algorithm mirror',upstream_revision=PIN,upstream_source_sha256=sha(p),fixtures_sha256=sha(fixtures.HERE/'fixtures.py'),audit_sha256=sha(Path(__file__)),primary='https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#concatenate-concatenate-data',primary_table='19.31',rows=rows)
def verify():
 assert json.loads((fixtures.HERE/'labels.json').read_text())==snapshot()
def main():
 import argparse
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args()
 if a.check:verify()
 else:(fixtures.HERE/'labels.json').write_text(json.dumps(snapshot(),indent=2,sort_keys=True)+'\n')
 print('11 exact pinned labels; static audit only')
if __name__=='__main__':main()
