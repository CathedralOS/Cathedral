#!/usr/bin/env python3
"""Reviewed mappings for the exact addr.rs slice; not a general translator."""
from pathlib import Path
import json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
p=ROOT/'source/libraries/x86_64/addresses-inventory.json';doc=json.loads(p.read_text())
source='source/libraries/x86_64/addresses.omg';fixture='tools/ports/x86_64-addresses/main.omg';extra='tools/ports/x86_64-addresses/extras.omg'
for key,row in doc['files']['src/addr.rs']['symbols'].items():
 number,name=key.split(':');number=int(number);target=None
 disposition='translated';reason='Checked numeric helper preserves pure behavior; errors become an explicit Rejected case, with no raw pointer or address authority.'
 if name in {'fmt','Output','any','tests','proofs','from_ptr','as_ptr','as_mut_ptr','test_from_ptr_array','new_unsafe','zero','as_u64','is_null'}:
  disposition='omitted';reason='Rust formatting/trait/newtype/proof-runner plumbing is not reproduced; numeric identity/zero/null are ordinary u64 values. Pointer conversions and unchecked construction are deliberately absent.'
 elif number>=1036:
  disposition='omitted';reason='Kani universal proof machinery is not translated or claimed. Its six semantic relations have finite Omega boundary instances in extras.omg; these are tests, not replacement proofs.';target=(extra,'machine relations(')
 elif number>=777:
  target=(extra,'pub machine run(') if name=='virtaddr_step_overflowing' else (fixture,'machine test_result(')
  reason='Pinned test expectations translated to deterministic Omega body checks; panic cases become Rejected. See generated vectors for the individual inputs and outputs.'
 elif name=='ADDRESS_SPACE_SIZE':target=(source,'pub const VIRTUAL_DENSE_MAX:');reason='Dense canonical ordinal maximum is one less than upstream ADDRESS_SPACE_SIZE; step bounds preserve the same 48-bit envelope.'
 elif name in {'VirtAddr','PhysAddr','VirtAddrNotValid','PhysAddrNotValid'}:
  target=(source,'pub data NumberResult');reason='Explicit u64 numeric inputs and case-bearing results replace Rust nominal address wrappers; every address-specific helper validates inputs. Rejection leaves the original input with its caller instead of duplicating it in an error wrapper.'
 else:
  physical=539<=number<750
  if name in {'new','try_new'}:machine='physical_check' if physical else 'virtual_check'
  elif name=='new_truncate':machine='physical_truncate' if physical else 'virtual_truncate'
  elif name in {'align_up','align_down','align_down_u64'}:machine=name if number>=750 else 'physical_align' if physical else 'virtual_align'
  elif name in {'is_aligned','is_aligned_u64'}:machine='physical_is_aligned' if physical else 'virtual_is_aligned'
  elif name in {'p1_index','p2_index','p3_index','p4_index','page_table_index'}:machine='page_index'
  elif name=='page_offset':machine=name
  elif name.startswith('steps_between'):machine='virtual_steps_between'
  elif name in {'forward_checked_impl','forward_checked_u64','backward_checked_u64','forward_checked','backward_checked'}:machine='virtual_step'
  elif name in {'forward_overflowing','backward_overflowing'}:machine='virtual_step_overflowing'
  elif name in {'add','add_assign'}:machine='physical_add' if physical else 'virtual_add'
  elif name in {'sub','sub_assign'}:machine='checked_sub' if number in {465,732} else 'physical_sub' if physical else 'virtual_sub'
  else:raise ValueError(key)
  target=(source,'machine '+machine+'(')
 row.update(disposition=disposition,reason=reason)
 if target:row['targets']=[dict(path=target[0],anchor=target[1])]
file=doc['files']['src/addr.rs'];file.update(disposition='translated',reason='Complete reviewed numeric addr.rs slice; Rust wrapper/formatting/unsafe pointer/proof infrastructure omissions are explicit per symbol.',targets=[dict(path=source,anchor='module addresses;')])
p.write_text(json.dumps(doc,indent=2)+'\n')
