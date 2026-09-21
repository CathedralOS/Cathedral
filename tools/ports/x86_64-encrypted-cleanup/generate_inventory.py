#!/usr/bin/env python3
"""Full-file pinned anchors, scoped to explicit-profile detached cleanup."""
import hashlib,importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];UP=ROOT/'reference_code/rust-osdev/x86_64'
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');i=importlib.util.module_from_spec(spec);spec.loader.exec_module(i)
files=[f'src/structures/paging/mapper/{mode}_page_table.rs'for mode in ['mapped','recursive']]
m=i.snapshot(UP,'cc35c876d3badb57df54a66e22f7768a52be95f2',files,'https://github.com/rust-osdev/x86_64')
for name,row in m['files'].items():
 target='source/libraries/x86_64/encrypted_cleanup_recursive.omg' if 'recursive' in name else 'source/libraries/x86_64/encrypted_cleanup_ranges.omg'
 row.update(disposition='translated',reason='Explicit-profile cleanup branch/cursor composition only; other mapper bodies and live authority remain outside this slice.',targets=[dict(path=target,anchor='pub machine step(')])
 for key,symbol in row['symbols'].items():
  if key.split(':',1)[1] in ['clean_up','clean_up_addr_range']:
   symbol.update(disposition='translated',reason='Detached branch, bounded complete range and recursive self-slot exclusion with accumulated PTE address mask; raw-word emptiness and ordered retirement preserved.',targets=[dict(path=target,anchor='pub machine step('),dict(path='source/libraries/x86_64/encrypted_cleanup_branch.omg',anchor='pub machine plan(')])
  else:symbol.update(disposition='omitted',reason='Separate mapper/default/profile slices or live boundary, outside the cleanup-only scope. This is not whole-file completion.')
m['mask_basis']=[dict(path=f,sha256=hashlib.sha256((UP/f).read_bytes()).hexdigest())for f in ['src/addr.rs','src/structures/paging/page_table.rs','src/structures/mem_encrypt.rs']]
m['canonical_dependencies']=['memory_encryption::State','cleanup_branch::CleanupPlan','cleanup_branch::CleanupOutcome','cleanup_ranges::Cursor','cleanup_ranges::RangeStatus','cleanup_ranges::RangeStep','recursive_cleanup::RecursiveCursor','recursive_cleanup::RecursiveStep']
cases=json.loads((HERE/'cases.json').read_text());worlds=cases['scenarios']
m['reference']={'profiles':11,'whole_tree_cases':len(worlds),'actual_public_mapped_calls':sum(not c['recursive']for c in worlds),'private_recursive_body_mirrors':sum(c['recursive']for c in worlds),'omega_step_bodies':len(cases['steps']),'omega_extra_bodies':6,'source':'tools/ports/x86_64-encrypted-cleanup/probe.rs.in','extraction':'tools/ports/x86_64-encrypted-cleanup/generate_reference.py','adaptations':['Mapped calls execute actual public CleanUp::clean_up_addr_range on stable disjoint owned initialized tables and a checked numeric registry.','Recursive private clean_up body preserves source loops/flags/zero tests/retirement ordering; pointer resolution alone is replaced by the same owned registry with explicit parameter plumbing.','Private VirtAddr::forward_checked_impl uses public Step::forward_checked whose exact pinned body delegates to it.','Recursive self-link is never resolved or retired; its raw word remains part of whole-root occupancy.','Raw entries are stored through set_addr(PhysAddr::zero(), PageTableFlags::from_bits_retain(word)); no pre-normalization erases encryption-only occupancy.','Each profile is configured in a fresh process before constructing typed addresses or tables; all backing allocations remain owned until final complete-tree observation.'],'not_claimed':['Actual public recursive mapper construction or execution','Live hierarchy access, invalidation, custody release or deallocation authority','Native Omega execution or ABI evidence','Universal proof beyond the finite corpus','CPU-valid encryption positions or provenance of a caller-supplied profile/capture']}
text=json.dumps(m,indent=2)+'\n';path=ROOT/'source/libraries/x86_64/encrypted-cleanup-inventory.json'
if '--check' in sys.argv:assert path.read_text()==text,'inventory drift'
else:path.write_text(text)
print(i.check(m,UP))
