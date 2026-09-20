#!/usr/bin/env python3
"""Audit pinned topology sources, extraction mapping, licenses and fixture controls."""
import hashlib,json,subprocess,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sys.path.insert(0,str(ROOT/'tools/ports'))
import inventory
def main():
 checkout=ROOT/'reference_code/rust-osdev/x86_64';manifest=inventory.read_json(ROOT/'source/libraries/x86_64/mapper-topology-inventory.json')
 print(json.dumps(inventory.check(manifest,checkout,ROOT),indent=2))
 metadata=inventory.read_json(HERE/'provenance.json');assert metadata['revision']==manifest['upstream']['revision']
 for name,digest in metadata['license_and_manifest_sha256'].items():assert hashlib.sha256((checkout/name).read_bytes()).hexdigest()==digest,name
 subprocess.run([sys.executable,str(HERE/'generate_reference.py'),'--check'],cwd=ROOT,check=True)
 cases=inventory.read_json(HERE/'cases.json')
 for name,item in cases.items():assert (HERE/'cases'/f'{name}.omg').read_text().count(item['mutation'][0])==1,name
 assert sum(case['numeric_checks'] for case in cases.values())==3139
 print(f'{len(cases)} body controls; 3131 Rust numeric witnesses and eight explicit input policies.')
 print('No live constructor, pointer or CR3 instruction claim.')
if __name__=='__main__':main()
