#!/usr/bin/env python3
"""Verify exact AML pin/source/license metadata and raw opcode facts."""
import hashlib
import json
from pathlib import Path
import re
import sys
ROOT=Path(__file__).resolve().parents[4]
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]))
import inventory

def main():
 checkout=ROOT/'reference_code/rust-osdev/acpi';package=ROOT/'source/libraries/acpi/aml'
 manifest=inventory.read_json(package/'inventory.json')
 print(json.dumps(inventory.check(manifest,checkout,ROOT),indent=2))
 provenance=inventory.read_json(HERE/'provenance.json')
 assert provenance['revision']==manifest['upstream']['revision']
 for path,digest in provenance['license_and_manifest_sha256'].items():
  assert hashlib.sha256((checkout/path).read_bytes()).hexdigest()==digest,path
 for path,entry in provenance['test_assets'].items():
  assert hashlib.sha256((checkout/path).read_bytes()).hexdigest()==entry['sha256'],path
 text=(checkout/'src/aml/mod.rs').read_text();body=text[text.index('    fn opcode('):text.index('    fn pkglength(')]
 expected={name:int(raw,16) for raw,name in re.findall(r'^\s*(0x[0-9a-f]+) => Opcode::(\w+)',body,re.M)}
 expected.pop('NameChar',None)
 expected.update(LNot=0x92,LNotEqual=0x9293,LLessEqual=0x9294,LGreaterEqual=0x9295,DigitFirst=0x30,DigitLast=0x39,NameCharFirst=0x41,NameCharLast=0x5a,NameCharUnderscore=0x5f,LocalFirst=0x60,LocalLast=0x67,ArgFirst=0x68,ArgLast=0x6e)
 assert expected==inventory.read_json(HERE/'opcodes.json')
 actual={name:int(value,16) for name,value in re.findall(r'pub const OP_(\w+):u16=(0x[0-9a-f]+);',(package/'opcodes.omg').read_text())}
 assert actual==expected
 coverage=inventory.read_json(HERE/'coverage.json')['tokens']
 assert set(coverage)==set(expected)
 for name,item in coverage.items():
  assert item['wire']==expected[name]
  assert item['reason']
  if 'target' in item:
   target=item['target'];assert target['anchor'] in (ROOT/target['path']).read_text()
 cases=inventory.read_json(HERE/'cases.json')
 for name,case in cases.items():
  text=(HERE/'cases'/f'{name}.omg').read_text()
  assert text.count(case['mutation'][0])==1,name
 print(f'{len(expected)} opcode/range facts and {len(provenance["test_assets"])} metadata-only test assets audited; {len(cases)} unique body mutation controls.')
 print('Pending inventory entries remain pending. No full AML interpreter or ABI success claim.')
if __name__=='__main__':main()
