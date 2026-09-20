#!/usr/bin/env python3
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
checkout=ROOT/'reference_code/rust-osdev/x86_64';pin='cc35c876d3badb57df54a66e22f7768a52be95f2'
m=shared.snapshot(checkout,pin,['src/registers/model_specific.rs'],'https://github.com/rust-osdev/x86_64')
for file in m['files'].values():
 file.update(disposition='translated',reason='Only the MSR split/combine pure expressions; live instructions and other register families outside this overlay.',targets=[{'path':'source/libraries/x86_64/msr_words.omg','anchor':'module msr_words;'}])
 for key,row in file['symbols'].items():
  line=int(key.split(':',1)[0])
  if line in [223,243]:row.update(disposition='translated',reason='Pure low/high word transport expression only; actual RDMSR/WRMSR remains external instruction/provider work.',targets=[{'path':'source/libraries/x86_64/msr_words.omg','anchor':'pub machine combine' if line==223 else 'pub machine low_word'}])
  else:row.update(disposition='omitted',reason='Outside this two-expression overlay; existing register representations and recipes have separate inventories.')
path=ROOT/'source/libraries/x86_64/msr-words-inventory.json';text=json.dumps(m,indent=2)+'\n'
if '--check' in sys.argv:assert path.read_text()==text,'stale MSR words manifest'
else:path.write_text(text)
print(shared.check(m,checkout))
