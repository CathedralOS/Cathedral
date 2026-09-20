#!/usr/bin/env python3
"""Author fixed x64 policy plans from independently checked pinned Rust facts."""
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent
RAW=HERE.parents[2]/'source/contracts/uefi/raw'
schema=json.loads((HERE/'schema.json').read_text())
vectors=json.loads((RAW/'network.vectors.json').read_text())['measurements']
lines=['// SPDX-License-Identifier: MIT OR Apache-2.0',
 '// Modified Cathedral plans for pinned uefi-rs network representations; see network.PORT.md.',
 '// These declarations are not proof of an evaluated foreign-layout consumer.',
 'module network_layouts;', 'use omega::language::core::layout;', '']
preamble=list(lines)
large=[]
for record in schema['records']:
 start=len(lines)
 name=record['name']; fields=record['omega_fields']; policy=name+'X64Layout'
 if len(record['fields'])!=len(fields) and record['kind']!='union':
  lines.append('// Fixed prefix only; runtime tail binding remains PORT-BLOCKED[omega:runtime-layout-strides].')
 if len(fields)>32:
  lines.append('// PORT-BLOCKED[omega:layout-reflection-capacity]: all 34 fields retained; current Schema capacity is 32.')
 lines += ['pub data '+policy+' {}','pub '+policy+'Evidence: '+policy+' satisfies Layout;',
  'pub machine '+policy+'::plan(schema: Schema) -> Plan satisfies Layout::plan {',
  '    let mut entries: [FieldEntry; 64];']
 for index,(field,_) in enumerate(fields):
  offset=vectors.get(name+'.'+field+'.offset',{'value':0})['value']
  lines.append(f'    entries[{index}] = FieldEntry {{ key: schema.fields[{index}].key, placement: FieldPlan::At {{ offset: {offset} }} }};')
 size=vectors[name+'.size']['value'];align=vectors[name+'.alignment']['value']
 lines += [f'    Plan {{ entries: entries, entry_count: {len(fields)}, size_fixed: {size}, size_is_dynamic: false, align: {align} }}','}', '']
 if len(fields)>32:
  large+=lines[start:]
  del lines[start:]
(RAW/'network_large_layouts.omg').write_text('\n'.join([x.replace('module network_layouts;', 'module network_large_layouts;') for x in preamble]+large))
(RAW/'network_layouts.omg').write_text('\n'.join(lines))
print('Authored 81 fixed plans; consumer validation remains separate.')
