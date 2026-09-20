#!/usr/bin/env python3
"""Audit transport macro fields and actual pinned Rust layouts/word encodings."""
import hashlib
from pathlib import Path
import re
import subprocess
import sys
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
sys.path.insert(0,str(HERE.parent))
import inventory, rust_layout, vectors
package=ROOT/'source/libraries/virtio'
up=ROOT/'reference_code/rust-osdev/virtio-spec-rs/src'
rows=inventory.read_json(HERE/'mmio-registers.json')
actual=re.findall(r'#\[offset\((0x[0-9a-f]+)\)\]\s*#\[access\((\w+)\)\]\s*(\w+):\s*([^,]+),',(up/'mmio.rs').read_text())
wanted=[(int(offset,16),access,name,ty) for offset,access,name,ty in actual]
observed=[(r['offset'],r['access'],r['name'],r['source_type']) for r in rows]
if wanted!=observed:raise ValueError('MMIO macro source coverage differs')
for module,metadata in [('mmio','mmio-registers.json'),('configuration_fields','pci-common-fields.json')]:
    source=(package/f'{module}.omg').read_text()
    for row in inventory.read_json(HERE/metadata):
        name=row['name'].upper()
        if module=='mmio' and name=='MAGIC_VALUE':name+='_REGISTER'
        body=re.search(r'pub const '+name+r': RegisterField = RegisterField \{([^}]+)\}',source)[1]
        data={k:v.strip() for k,v in re.findall(r'(\w+):\s*([^,]+)',body)}
        if int(data['offset'],0)!=row['offset'] or int(data['bytes'])!=row['bytes']:raise ValueError(name+': offset/width mismatch')
        if data['readable']!=str(row['access']!='WriteOnly').lower() or data['writable']!=str(row['access']!='ReadOnly').lower():raise ValueError(name+': access metadata mismatch')
body=(up/'pci.rs').read_text().split('pub struct CommonCfg {',1)[1].split('\n}',1)[0]
fields=[];access='ReadWrite'
for line in body.splitlines():
    if '#[access(' in line:access=re.search(r'access\((\w+)\)',line)[1]
    match=re.match(r'\s*(\w+): (\w+),',line)
    if match:fields.append((match[1],match[2],access));access='ReadWrite'
if fields!=[(r['name'],r['source_type'],r['access']) for r in inventory.read_json(HERE/'pci-common-fields.json')]:raise ValueError('CommonCfg access/source coverage differs')
print('All30 MMIO macro fields and23 PCI common fields retain source access/width/offset metadata.',flush=True)
expected=vectors.validate(inventory.read_json(package/'transport.vectors.json'))
measured,artifact=rust_layout.measure(HERE/'Cargo.toml',HERE/'src/transport_probe.rs','CATHEDRAL_VIRTIO_TRANSPORT','cathedral_virtio_layout_probe')
if set(measured)!=set(expected['measurements']):raise ValueError('transport measurement coverage differs')
for key,value in measured.items():
    if expected['measurements'][key]['value']!=value:raise ValueError(key+': actual Rust target geometry differs')
print(f'{len(measured)} actual transport geometry measurements agree; LLVM SHA-256: '+hashlib.sha256(artifact).hexdigest(),flush=True)
subprocess.run(['cargo','run','--locked','--manifest-path',str(HERE/'Cargo.toml')],cwd=ROOT,check=True)
print('All operations remain data plans. Omega ABI/native device integration NOT RUN.')
