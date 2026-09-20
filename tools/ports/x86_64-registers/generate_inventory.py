#!/usr/bin/env python3
"""Bound the register-only translation, leaving live operations explicitly omitted."""
from pathlib import Path
import sys
CHECK="--check" in sys.argv
def emit(path,text):
 if CHECK:
  if not path.exists() or path.read_text()!=text:raise SystemExit("generated artifact differs: "+str(path))
 else:path.write_text(text)
import importlib.util
import json
import re
ROOT=Path(__file__).resolve().parents[3];HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('shared_inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
schema=json.loads((HERE/'schema.json').read_text());checkout=ROOT/'reference_code/rust-osdev/x86_64'
manifest=shared.snapshot(checkout,schema['revision'],schema['files'],'https://github.com/rust-osdev/x86_64')
facts='source/drivers/facts/x86_registers.omg';helpers='source/libraries/x86_64/registers.omg'
constant_by_line={(c['path'],c['line']):c['name'] for group in schema['flags'] for c in group['members']}
constant_by_line.update({(m['path'],m['line']):m['name'] for m in schema['msrs']})
types={f['name'] for f in schema['flags']}|set(schema['enums'])|{'SegmentSelector','Msr'}
# Exact source method starts disambiguate same-named impl methods/private helpers.
methods={
'src/lib.rs':{60:'privilege_from_u16'},
'src/registers/control.rs':{207:'priority_class_valid'},
'src/registers/debug.rs':{83:'debug_register_number_valid',94:'debug_register_number_valid',155:'dr6_trap',218:'dr7_local_enable',228:'dr7_global_enable',257:'breakpoint_condition_valid',267:'dr7_condition',292:'breakpoint_size_from_bytes',303:'breakpoint_size_bits_valid',313:'dr7_size',329:'dr7_from_bits_truncate',335:'dr7_bits_valid',343:'dr7_bits_valid',353:'dr7_from_bits_truncate',365:'dr7_bits_valid',371:'dr7_bits_valid',377:'dr7_flags',383:'dr7_insert_flags',389:'dr7_remove_flags',395:'dr7_toggle_flags',401:'dr7_set_flags',410:'dr7_condition',416:'dr7_set_condition',422:'dr7_size',428:'dr7_set_size'},
'src/registers/model_specific.rs':{19:'msr_new',397:'star_sysret_base',411:'star_syscall_base',446:'star_pack_raw',468:'star_plan',503:'pub data StarPlan'},
'src/registers/mxcsr.rs':{53:'mxcsr_default'},
'src/registers/segmentation.rs':{78:'selector_new',84:'selector_new',88:'selector_index',94:'selector_rpl',100:'selector_set_rpl'},
}
for path,file in manifest['files'].items():
 file.update(disposition='translated',reason='Claimed pure register slice translated; live instruction methods, unrelated root modules and Rust-only boilerplate deliberately omitted with per-anchor reasons.',targets=[{'path':facts,'anchor':'module x86_registers;'}])
 for key,row in file['symbols'].items():
  number=int(key.split(':',1)[0]);name=key.split(':',1)[1];target=None
  if (path,number) in constant_by_line:target=(facts,'pub const '+constant_by_line[(path,number)])
  elif name in types:target=(facts,'pub data '+name+' [copy]')
  elif path=='src/registers/debug.rs' and name=='Dr7Value':target=(facts,'pub data Dr7Bits [copy]')
  elif number in methods.get(path,{}):target=(helpers,methods[path][number])
  elif path=='src/lib.rs' and name in {'Ring0','Ring1','Ring2','Ring3'}:target=(facts,'PRIVILEGE_LEVEL_RING'+name[-1])
  elif path=='src/registers/control.rs' and name=='PriorityClass1':target=(facts,'PRIORITY_CLASS_CLASS1')
  elif path=='src/registers/debug.rs' and name in {'InstructionExecution','DataWrites','IoReadsWrites','DataReadsWrites','Length1B','Length2B','Length8B','Length4B'}:
   target=(facts,'pub data BreakpointCondition' if not name.startswith('Length') else 'pub data BreakpointSize')
  if target:
   row.update(disposition='translated',reason='Inert raw value, validity predicate, field projection, or detached transformation only. STAR read/write mappings cover extracted bit/validation work, never the live instruction. Unchecked Dr7 construction is explicit raw Dr7Bits, not a validity proof.',targets=[{'path':target[0],'anchor':target[1]}])
  else:
   row.update(disposition='omitted',reason='Outside this bounded pure register slice: live register/CPU observation or mutation, zero-sized instruction proxy, Rust formatting/trait/scaffolding, hardware-dependent tests, or root modules inventoried in source/libraries/x86_64/inventory.json. No operation is claimed callable or blocked merely because omitted here.')
# Preserve every implicit enum variant from the full source-bound supplement,
# with explicit targets to the complete raw code-constant family.
supplement=json.loads((ROOT/'source/libraries/x86_64/lexical-supplement.json').read_text())
manifest['supplemental_enum_mappings']=[]
for path,row in supplement['files'].items():
 if path not in manifest['files']:continue
 for key,entry in row['symbols'].items():
  parts=key.split(':')
  if ':variant::' in key and parts[3] in schema['enums']:
   name=parts[3]
   manifest['supplemental_enum_mappings'].append({'source':path,'key':key,'anchor':entry['anchor'],'target':{'path':facts,'anchor':'pub data '+name+' [copy]'},'reason':'Complete implicit/explicit numeric code family appears beside this raw carrier; constants and discriminants independently measured in vectors.'})
path=ROOT/'source/drivers/facts/x86_registers-inventory.json';emit(path,json.dumps(manifest,indent=2)+'\n')
print(shared.check(manifest,checkout))
