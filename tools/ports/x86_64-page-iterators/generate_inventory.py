#!/usr/bin/env python3
"""Pinned PageRange transition anchors; existing numeric/trait exclusions kept."""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];UP=ROOT/'reference_code/rust-osdev/x86_64'
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');i=importlib.util.module_from_spec(spec);spec.loader.exec_module(i)
m=i.snapshot(UP,'cc35c876d3badb57df54a66e22f7768a52be95f2',['src/structures/paging/page.rs'],'https://github.com/rust-osdev/x86_64')
target='source/libraries/x86_64/page_iterators.omg'
for row in m['files'].values():
 row.update(disposition='translated',reason='Detached virtual PageRange/Inclusive next/nth/back cursor transitions, including partial updates before panic; existing bounded selection stays separate.',targets=[dict(path=target,anchor='pub machine step(')])
 for key,s in row['symbols'].items():
  if key.split(':',1)[1]in ['next','nth','next_back','nth_back']:
   s.update(disposition='translated',reason='Actual public Rust iterator observations compared to detached numeric start/end/selection/failure transitions, 64-bit usize and three canonical page sizes.',targets=[dict(path=target,anchor='pub machine step(')])
  else:s.update(disposition='omitted',reason='Existing pages/address algorithms or explicit nominal, unchecked, formatting, trait, native or universal-proof boundary; outside this stateful-iterator slice.')
m['canonical_dependencies']=['encrypted_frames::IteratorResult','addresses::NumberResult','pages::start_valid','pages::difference','addresses::virtual_add','addresses::virtual_sub','pages::steps_between','pages::containing']
m['reference']={'actual_public_observations':1626,'omega_added_geometry_cases':4,'public_calls':['PageRange::next','PageRange::nth','PageRange::next_back','PageRange::nth_back','PageRangeInclusive::next','PageRangeInclusive::nth','PageRangeInclusive::next_back','PageRangeInclusive::nth_back'],'source':'tools/ports/x86_64-page-iterators/probe.rs.in','observation':'Both cursor fields are read after caught panics as well as success/exhaustion; no private-body mirror. Invalid addresses are compared at actual public typed construction.','profile':'64-bit usize, canonical 48-bit virtual addresses, Size4KiB/Size2MiB/Size1GiB; no memory-encryption dependency in virtual iterator arithmetic.','not_claimed':['Native Omega execution or layout','Live mapping/page authority','Unchecked invalid Rust values','32-bit usize saturation','Universal Kani proofs']}
p=ROOT/'source/libraries/x86_64/page-iterators-inventory.json';s=json.dumps(m,indent=2)+'\n'
if '--check'in sys.argv:assert p.read_text()==s,'inventory drift'
else:p.write_text(s)
print(i.check(m,UP))
