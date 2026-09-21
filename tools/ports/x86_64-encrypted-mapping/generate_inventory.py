#!/usr/bin/env python3
from pathlib import Path
import importlib.util,json,hashlib,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];UP=ROOT/'reference_code/rust-osdev/x86_64';PIN='cc35c876d3badb57df54a66e22f7768a52be95f2'
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');i=importlib.util.module_from_spec(spec);spec.loader.exec_module(i)
files=[f'src/structures/paging/mapper/{mode}_page_table.rs' for mode in ['mapped','recursive']];m=i.snapshot(UP,PIN,files,'https://github.com/rust-osdev/x86_64')
methods={'map_to_with_table_flags','unmap','update_flags','set_flags_p4_entry','set_flags_p3_entry','set_flags_p2_entry','translate_page'}
for f,row in m['files'].items():
 target='source/libraries/x86_64/encrypted_recursive_routes.omg' if 'recursive' in f else 'source/libraries/x86_64/encrypted_mapping_routes.omg'
 row.update(disposition='translated',reason='Only explicit-encryption leaf/child/route arithmetic, reusing canonical types; no live mapper/allocator/flush interface.',targets=[{'path':target,'anchor':'pub machine plan('}])
 for key,s in row['symbols'].items():
  name=key.split(':',1)[1]
  if name in methods:t={'path':target,'anchor':'pub machine plan('}
  elif name in {'create_next_table','inner'}:t={'path':'source/libraries/x86_64/encrypted_mapping_plans.omg','anchor':'pub machine prepare_child('}
  elif name in {'next_table','next_table_mut'}:t={'path':target,'anchor':'state existing('}
  else:s.update(disposition='omitted',reason='Canonical default/numeric representation or separate generic translation, cleanup, topology, formatting and live authority boundary. This slice is not whole-file completion.');continue
  s.update(disposition='translated',reason='Explicit-profile component using accumulated PTE decoding and current physical admission, with exact source decisions and retained effects.',targets=[t])
m['mask_basis']=[{'path':f,'sha256':hashlib.sha256((UP/f).read_bytes()).hexdigest()} for f in ['src/addr.rs','src/structures/paging/page_table.rs','src/structures/mem_encrypt.rs']]
m['canonical_dependencies']=['memory_encryption::State','mapping_routes::Request','mapping_routes::CapturedPath','mapping_routes::AllocationInputs','mapping_routes::Edits','mapping_routes::Outcome','mapping_routes::RoutePlan','mapping_plans::FrameSupply','mapping_plans::ChildPlan','mapping_plans::LeafPlan']
m['reference']={'profiles':11,'routes':318,'actual_public_mapped_calls':159,'source_body_mirrors':318,'method_bodies':42,'public_reference':'tools/ports/x86_64-encrypted-mapping/src/public_mapped.rs','instrumented_references':['tools/ports/x86_64-encrypted-mapping/src/mapped.rs','tools/ports/x86_64-encrypted-mapping/src/recursive.rs'],'adaptations':['Actual public mapped calls use stable distinct owned initialized tables and a checked numeric ID registry, never installed as a root.','Copied mapped/recursive method bodies receive borrowed snapshots instead of self root and raw topology dereferences.','Instrumented entries execute actual pinned PTE APIs; complete clearing executes actual PageTable::zero.','Copied private child/walker branches and error conversions retained; unsafe allocator trait replaced only in mirrors by safe numeric observation queue.','Actual public mapped final words, zero observations, allocation calls and return/panic payloads cross-check their mirrors. Setter counts and deepest access are measured by mirrors only.','Each configured profile runs in a fresh process before construction of any typed address/table.'],'not_claimed':['Live recursive mapper','Hardware execution','Native Omega code or ABI','Caller profile/capture provenance','Exact public typed construction of rejected ordinary numeric inputs']}
m['extra_policies']=['Reject current-bit or malformed requested frames before route effects.','Reject invalid allocation observation at its consumption point, retaining earlier effects.','Capture mismatch after address-bit flag OR retains the preceding parent write.','New child snapshot ID follows the resulting accumulated-mask address, even with raw flag address contamination.']
p=ROOT/'source/libraries/x86_64/encrypted-mapping-inventory.json';s=json.dumps(m,indent=2)+'\n'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=s:raise SystemExit('inventory drift')
else:p.write_text(s)
print(i.check(m,UP))
