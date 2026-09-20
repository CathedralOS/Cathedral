#!/usr/bin/env python3
"""Narrow owned GDT storage overlay; retain whole-file source identity."""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
checkout=ROOT/'reference_code/rust-osdev/x86_64';revision='cc35c876d3badb57df54a66e22f7768a52be95f2'
m=shared.snapshot(checkout,revision,['src/structures/gdt.rs'],'https://github.com/rust-osdev/x86_64')
source='source/libraries/x86_64/gdt_storage.omg';tests='tools/ports/x86_64-gdt-storage/src/main.rs'
anchors={108:'pub data Table',115:'pub machine initialize_default',122:'pub machine initialize_default',130:'pub machine initialize(',137:'let words: [u64; 8192]',161:'pub machine import_words',185:'pub machine entry(',197:'pub machine append(',250:'state write_user(',258:'pub machine limit('}
scenarios={555:'"mixed"',568:'"system_fits"',576:'"mixed"',589:'"full_import"',596:'"single_slot"',604:'"import"'}
for file in m['files'].values():
 file.update(disposition='translated',reason='Detached owned storage component only; other declarations explicitly outside this narrow overlay.',targets=[{'path':source,'anchor':'module gdt_storage;'}])
 for key,row in file['symbols'].items():
  line=int(key.split(':',1)[0]);target=None
  if line in anchors:target=(source,anchors[line])
  if line in scenarios:target=(tests,scenarios[line])
  if target:row.update(disposition='translated',reason='Ordinary owned numeric storage or retained storage test scenario. Fixed 8192-word backing plus checked logical capacity replaces const generics; no native ABI, atomics or live table loading.',targets=[{'path':target[0],'anchor':target[1]}])
  else:row.update(disposition='omitted',reason='Outside this storage overlay: canonical descriptor facts/codecs already translated separately, Rust scaffolding/formatting/traits, or live loading/pointer/atomic access.')
path=ROOT/'source/libraries/x86_64/gdt-storage-inventory.json';text=json.dumps(m,indent=2)+'\n'
if '--check' in sys.argv:assert path.read_text()==text,'stale GDT storage inventory'
else:path.write_text(text)
print(shared.check(m,checkout))
