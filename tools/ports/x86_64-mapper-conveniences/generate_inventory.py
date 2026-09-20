#!/usr/bin/env python3
"""Bounded mapper defaults/views overlay; existing route/translation types remain canonical."""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('i',ROOT/'tools/ports/inventory.py');i=importlib.util.module_from_spec(spec);spec.loader.exec_module(i)
checkout=ROOT/'reference_code/rust-osdev/x86_64';path='src/structures/paging/mapper/mod.rs'
m=i.snapshot(checkout,'cc35c876d3badb57df54a66e22f7768a52be95f2',[path],'https://github.com/rust-osdev/x86_64')
helper='source/libraries/x86_64/mapper_conveniences.omg';existing='source/libraries/x86_64/translation.omg'
anchors={28:(helper,'pub machine translate_address_from_words'),36:(existing,'pub machine translate_words'),45:(helper,'pub machine translated_address_pinned'),58:(existing,'pub data Translation'),80:(helper,'pub data MappedFrame'),91:(helper,'pub machine frame_start'),100:(helper,'pub machine frame_size'),179:(helper,'pub machine default_map('),353:(helper,'pub machine translate_page('),362:(helper,'pub machine identity_map(')}
file=m['files'][path];file.update(disposition='translated',reason='Bounded defaults and numeric view overlay. Remaining interfaces already implemented elsewhere or deliberately excluded from this wrapper API.',targets=[{'path':helper,'anchor':'module mapper_conveniences;'}])
for key,row in file['symbols'].items():
 line,name=key.split(':',1);n=int(line);target=anchors.get(n)
 if target:
  reason='Pure wrapper/view over existing numeric route or translation implementation; no mapped-frame, memory, flush or custody authority.'
  if n==45:reason='Exact default addition permits offset>=size on a valid typed-frame shape; physical overflow/panic becomes Rejected. Existing captured-result strict helper remains unchanged.'
  if n==58:reason='Existing Translation remains canonical; local InvalidVirtual/InvalidRootHugePage errors remain explicit. Generic InvalidFrameAddress payload is not separately reintroduced; optional-address projection maps all nonmapped outcomes to Rejected.'
  if n==362:reason='Validates typed-frame geometry and strict canonical identity address before default route delegation. No manufactured flush receipt.'
  row.update(disposition='translated',reason=reason,targets=[{'path':target[0],'anchor':target[1]}])
 else:
  reason='Existing underlying mapping routes/plans or other mapper implementation, Rust trait/format scaffolding, and cleanup are outside this bounded convenience overlay; no new blocker claim.'
  if 390<=n<=448:reason='MapperFlush/MapperFlushAll construction, page projection, flush and public ignore are deliberately excluded as authority APIs. Numeric route edits do not settle invalidation or grant the ability to discard a debt.'
  row.update(disposition='omitted',reason=reason)
m['semantic_variants']={'MappedFrame':['Size4KiB(address)','Size2MiB(address)','Size1GiB(address)'],'meaning':'Numeric size alternatives only; checked constructor uses existing physical frame geometry. No native enum layout or backing authority.'}
m['reused_implementations']=['mapping_routes::plan','mapping_plans::default_parent_flags','pages::start_valid','addresses::virtual_valid','addresses::physical_add','translation::translate_words','translation::translated_address']
p=ROOT/'source/libraries/x86_64/mapper-conveniences-inventory.json';text=json.dumps(m,indent=2)+'\n'
if '--check' in sys.argv:
 if not p.exists() or p.read_text()!=text:raise SystemExit('inventory drift')
else:p.write_text(text)
print(i.check(m,checkout))
