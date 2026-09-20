#!/usr/bin/env python3
"""Explicit-state encryption overlay, preserving whole pinned source hashes."""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
spec=importlib.util.spec_from_file_location('inventory',ROOT/'tools/ports/inventory.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
checkout=ROOT/'reference_code/rust-osdev/x86_64';pin='cc35c876d3badb57df54a66e22f7768a52be95f2'
m=shared.snapshot(checkout,pin,['src/structures/mem_encrypt.rs','src/structures/paging/page_table.rs','src/addr.rs'],'https://github.com/rust-osdev/x86_64')
source='source/libraries/x86_64/memory_encryption.omg'
anchors={
 'src/structures/mem_encrypt.rs':{14:'bit_mask: u64',18:'reversed: bool',22:'pub data Configuration',44:'pub machine configure',59:'pub machine encryption_flag',74:'pub machine set_encrypted',80:'pub machine is_encrypted'},
 'src/structures/paging/page_table.rs':{28:'address_mask: u64',59:'pub machine entry_flags',65:'pub machine entry_address',77:'pub machine entry_frame',89:'pub machine set_address',96:'pub machine set_frame',103:'pub machine set_flags',115:'pub machine entry_address'},
 'src/addr.rs':{539:'pub machine physical_check',557:'pub machine physical_truncate',578:'pub machine physical_check'}}
for path,file in m['files'].items():
 file.update(disposition='translated' if 'mem_encrypt' in path else 'pending',reason='Explicit-state encryption component overlay; unrelated declarations remain in separate slices.',targets=[{'path':source,'anchor':'module memory_encryption;'}])
 for key,row in file['symbols'].items():
  line=int(key.split(':',1)[0]);anchor=anchors[path].get(line)
  if anchor:row.update(disposition='translated',reason='Detached explicit-state arithmetic only: global atomics and actual CPU/mapping configuration excluded; repeated-mask accumulation preserved.',targets=[{'path':source,'anchor':anchor}])
  else:row.update(disposition='omitted',reason='Outside this narrow encryption overlay: other address/PTE/table algorithms have their own slice mappings; Rust scaffolding and live atomic access are not recreated.')
m['configuration_cases']=[{'upstream':'MemoryEncryptionConfiguration::EncryptedBit(u8)','target':'Configuration::EncryptedBit(position: u8)'},{'upstream':'MemoryEncryptionConfiguration::SharedBit(u8)','target':'Configuration::SharedBit(position: u8)'}]
path=ROOT/'source/libraries/x86_64/memory-encryption-inventory.json';text=json.dumps(m,indent=2)+'\n'
if '--check' in sys.argv:assert path.read_text()==text,'stale memory encryption inventory'
else:path.write_text(text)
print(shared.check(m,checkout))
