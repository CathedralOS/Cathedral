#!/usr/bin/env python3
"""Compare existing/additive UART facts and Rust-only divisor ordinals to the pin."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sys.path.insert(0,str(HERE.parent))
import inventory
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--write',action='store_true');args=parser.parse_args()
PIN='653455f17e92c67a67e44b4c4b6eaba22411e57b'
inventory.sources(ROOT/'reference_code/rust-osdev/uart_16550',PIN,['src','tests/api.rs'])
output=subprocess.check_output(['cargo','run','--quiet','--locked','--manifest-path',str(HERE/'Cargo.toml')],text=True)
values=dict((k,int(v)) for k,v in (line.split('=') for line in output.splitlines()))
facts=(ROOT/'source/drivers/facts/uart_16550.omg').read_text()
ordinals=(ROOT/'source/drivers/uart_16550/enum_ordinals.omg').read_text()
for name,value in values.items():
 source=ordinals if name.startswith('DIVISOR') else facts
 match=re.search(r'pub const '+name+r'\s*:\s*\w+\s*=\s*(0x[0-9a-fA-F]+|[0-9]+);',source)
 assert match and int(match[1],0)==value,(name,value)
rows=json.loads((HERE/'facts.json').read_text())
assert set(values)=={r['name'] for r in rows}|set(re.findall(r'pub const (DIVISOR\d+)',ordinals))
print(f'{len(values)} upstream constants/ordinals verified; no hardware access or Omega execution.')

document={'format':'cathedral-port-vectors-v1', 'target':{'abi':'uart-register-values-and-rust-enum-identities','pointer_bits':64,'endian':'little'},'provenance':{'kind':'upstream','revision':PIN,'sources':['src/spec.rs'],'description':'Independently compiled pinned Rust constants and enum ordinals; no device IO or Omega representation observation.','rustc':subprocess.check_output(['rustc','--version'],text=True).strip()},'measurements':{name:{'kind':'value','value':value} for name,value in values.items()}}
path=ROOT/'source/drivers/uart_16550/vectors.json'
if args.write:path.write_text(json.dumps(document,indent=2)+'\n')
elif json.loads(path.read_text())['measurements']!=document['measurements']:raise ValueError('vector disagreement; review before --write')
