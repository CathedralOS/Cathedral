#!/usr/bin/env python3
from pathlib import Path
import importlib.util,json,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];UP=ROOT/'reference_code/rust-osdev/x86_64'
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');i=importlib.util.module_from_spec(spec);spec.loader.exec_module(i)
f='src/structures/paging/mapper/recursive_page_table.rs';m=i.snapshot(UP,'cc35c876d3badb57df54a66e22f7768a52be95f2',[f],'https://github.com/rust-osdev/x86_64')
route='source/libraries/x86_64/recursive_routes.omg';translation='source/libraries/x86_64/recursive_translation.omg';row=m['files'][f]
row.update(disposition='translated',reason='Captured numeric route/translation component only. Constructors/topology/cleanup and live unsafe interfaces retain separate inventory boundaries.',targets=[{'path':route,'anchor':'pub machine plan('},{'path':translation,'anchor':'pub machine translate_words('}])
methods={'map_to_with_table_flags','unmap','update_flags','set_flags_p4_entry','set_flags_p3_entry','set_flags_p2_entry','translate_page'}
for key,s in row['symbols'].items():
 name=key.split(':',1)[1]
 if name in methods:target={'path':route,'anchor':'pub machine plan('}
 elif name in {'create_next_table','inner'}:target={'path':route,'anchor':'pub machine prepare_child('}
 elif name=='translate':target={'path':translation,'anchor':'pub machine translate_words('}
 else:
  s.update(disposition='omitted',reason='Separate topology/constructor or cleanup slice, Rust formatting, raw pointer access, or live RecursivePageTable representation. This bounded inventory does not classify ordinary remaining engineering as blocked.');continue
 s.update(disposition='translated',reason='Pinned decisions and mutation order extracted into numeric captured results; no recursive references, CPU state, hierarchy authority or invalidation settlement.',targets=[target])
m['reference']={'kind':'Adapted exact source-body mirror, not execution through an actual RecursivePageTable object.','source':'tools/ports/x86_64-recursive-routes/src/pinned.rs','adaptations':['Free functions receive disjoint borrowed initialized tables instead of self.p4 and raw recursive pointer resolutions.','p3/p2/p1 coordinate values become their corresponding supplied snapshot reference; topology arithmetic has a separate slice.','PageTableEntry and PageTable aliases are instrumented safe adapters executing actual pinned PTE operations and PageTable::zero.','Unsafe FrameAllocator trait replaced by safe numeric allocation-observation queue; no backing or uniqueness grant constructed.','crate flag imports point to x86_64; the unused Page constructor gets explicit Size4KiB after removal of pointer-driven type inference.','Branch conditions, typed errors, parent flag OR, PTE setter calls and zero ordering remain source-extracted.'],'scenarios':{'routes':232,'translations':90},'measurements':['Final selected words','Assignment and zero masks including redundant setters','Allocation attempts','Deepest accessed selected table','Typed return/errors and caught setter/translation panics','Nonselected dirty sentinel cleared by actual PageTable::zero'],'not_measured':['Live recursive mapping','Hardware writes/CPU permissions','Native Omega layout/execution','Numeric table ID provenance']}
m['capture_policy']={'ids':'Additional consistency check over supplied physical snapshot identities; equality confers no provenance. New zero requests update next identity from resulting parent address.','partial_effects':'Earlier parent writes remain on child error, allocation exhaustion, leaf error or capture mismatch.','defaults':'Canonical48 virtual and default physical52 masks.','reused_types':['mapping_routes::CapturedPath','mapping_routes::AllocationInputs','mapping_routes::Request','mapping_routes::Edits','mapping_routes::Outcome','mapping_routes::RoutePlan','mapping_plans::ChildPlan','mapping_plans::LeafPlan','translation::Translation']}
p=ROOT/'source/libraries/x86_64/recursive-routes-inventory.json';text=json.dumps(m,indent=2)+'\n'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=text:raise SystemExit('inventory drift')
else:p.write_text(text)
print(i.check(m,UP))
