#!/usr/bin/env python3
"""Inventory extracted components without marking the enclosing live APIs callable."""
from pathlib import Path
import importlib.util,json,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');i=importlib.util.module_from_spec(spec);spec.loader.exec_module(i)
files=['src/registers/control.rs','src/registers/model_specific.rs','src/registers/rflags.rs','src/registers/xcontrol.rs','src/instructions/tlb.rs'];checkout=ROOT/'reference_code/rust-osdev/x86_64'
m=i.snapshot(checkout,'cc35c876d3badb57df54a66e22f7768a52be95f2',files,'https://github.com/rust-osdev/x86_64')
helper='source/libraries/x86_64/register_operands.omg'
maps={
files[0]:{'266:write':'register_merge','519:write':'register_merge','340:read':'cr3_observed_flags','348:read_raw':'cr3_observed','364:read_pcid':'cr3_observed_pcid','376:write':'cr3_flags_operand','390:write_pcid':'cr3_pcid_operand','405:write_pcid_no_flush':'cr3_pcid_operand','418:write_raw':'cr3_raw_operand','423:write_raw_impl':'cr3_raw_operand','570:read':'cr8_observed','588:write':'cr8_operand'},
files[1]:{'280:write':'register_merge','411:read':'star_expand','605:read':'cet_observed','617:write':'cet_operand','651:read':'cet_observed','663:write':'cet_operand','682:read':'apic_observed','689:read_raw':'apic_observed','705:write':'apic_preserving_operand','723:write_raw':'apic_raw_operand'},
files[2]:{'100:write':'register_merge'},files[3]:{'90:write':'xcr0_plan'},files[4]:{}}
for f,row in m['files'].items():
 row.update(disposition='translated',reason='Complete source binding for a bounded extracted numeric recipe or exact PCID mirror; instruction execution and surrounding live APIs are not translated.',targets=[{'path':helper,'anchor':'module register_operands;'}])
 assert set(maps[f])<=set(row['symbols']),(f,set(maps[f])-set(row['symbols']))
 for key,s in row['symbols'].items():
  if key in maps[f]:s.update(disposition='translated',reason='Only detached observation/operand computation is implemented. CPU read/write/feature/lifetime/invalidation authority deliberately remains outside this result.',targets=[{'path':helper,'anchor':'pub machine '+maps[f][key]+'('}])
  elif f==files[4] and key in ('64:Pcid','69:new','78:value'):
   s.update(disposition='translated',reason='Reuses prior bounded PCID representation; exact Rust type/constructor/getter copied only into the host pure-body witness to avoid architecture-gated instruction modules.',targets=[{'path':'source/drivers/facts/x86_tlb_operands.omg','anchor':'pub data Pcid'}])
  else:s.update(disposition='omitted',reason='Outside this numeric fragment slice: live instruction/state/closure, Rust scaffolding, or previously inventoried representation/algorithm. See baseline and other slice inventories; this is not a missing-language claim.')
m['scope']={'closed_audit_families':['register-merge','cr3-cr8-operands','xcr0-validation','msr-composition'],'boundary':'Pure components only; ordinary copied values never current-CPU facts or write authority.','omega_module':helper,'reference':'tools/ports/x86_64-register-operands/src/pinned.rs','reference_kind':'Exact extracted Rust fragments with observation inputs and returned operands; wrappers use actual pinned flag/address/frame/page/selector/priority APIs. No live register APIs execute.','deviations':['Explicit rejection for numeric inputs that cannot construct typed upstream frames/pages.','STAR selector overflow is rejected in every build; reference runs with overflow checks enabled.','XCR0 ordered assertion failures become semantic cases; successful plan is not complete CPU validation.','Default physical52 profile only; ambient memory encryption is excluded.','Forged ordinary PriorityClass payloads are revalidated before operand composition.']}
m['artifacts']={'cases':'tools/ports/x86_64-register-operands/cases.json','observations':'tools/ports/x86_64-register-operands/observations.json','fixtures':'tools/ports/x86_64-register-operands/fixtures.json','count':1323}
out=ROOT/'source/libraries/x86_64/register-operands-inventory.json';text=json.dumps(m,indent=2)+'\n'
if '--check' in sys.argv:
 if not out.exists() or out.read_text()!=text:raise SystemExit('inventory drift')
else:out.write_text(text)
print(i.check(m,checkout))
