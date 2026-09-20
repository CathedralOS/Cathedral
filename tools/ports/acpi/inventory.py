#!/usr/bin/env python3
"""Reproduce ACPI source partitions and test-scenario metadata without copying fixtures."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];DEST=ROOT/'source/libraries/acpi'
spec=importlib.util.spec_from_file_location('port_inventory',HERE.parent/'inventory.py');inventory=importlib.util.module_from_spec(spec);spec.loader.exec_module(inventory)
PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5';CHECKOUT=ROOT/'reference_code/rust-osdev/acpi'
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
source=inventory.sources(CHECKOUT,PIN,['src','tests','tools'])
doc=inventory.read_json(DEST/'inventory.json')
CATEGORIES={'byte_table_facts':'ACPI-001/002','table_discovery_parsing':'ACPI-001/002','platform_topology':'ACPI-003','aml_syntax_namespace':'ACPI-004','aml_execution':'ACPI-005','handler_boundary':'ACPI-007','allocator_concurrency':'ACPI-006','platform_policy':'ACPI-007','host_test_tool':'ACPI-004/005/006'}
def classify(path,name,line):
 if path.startswith('tools/'):return ['host_test_tool']
 if path.startswith('tests/'):
  if Path(path).stem in {'namespace_paths','name_search'}:return ['host_test_tool','aml_syntax_namespace']
  if Path(path).stem in {'global_lock'}:return ['host_test_tool','allocator_concurrency','handler_boundary']
  if Path(path).stem in {'operation_region','normal_fields','bank_fields','index_fields'}:return ['host_test_tool','aml_execution','handler_boundary']
  return ['host_test_tool','aml_execution']
 if path=='src/rsdp.rs':return ['handler_boundary','table_discovery_parsing'] if name in {'search_for_on_bios','find_search_areas'} else ['byte_table_facts','table_discovery_parsing']
 if path.startswith('src/sdt/'):return ['byte_table_facts','table_discovery_parsing']
 if path=='src/address.rs':return ['handler_boundary'] if name in {'MappedGas','map_gas','read','write'} else ['byte_table_facts','table_discovery_parsing']
 if path=='src/registers.rs':return ['handler_boundary','byte_table_facts']
 if path.startswith('src/platform/'):
  return ['handler_boundary','platform_policy'] if name in {'initialize_events','read_mode','enter_acpi_mode','wake_aps'} else ['platform_topology','allocator_concurrency']
 if path=='src/aml/op_region.rs':return ['handler_boundary','byte_table_facts']
 if path=='src/aml/resource.rs':return ['aml_syntax_namespace','byte_table_facts']
 if path=='src/aml/pci_routing.rs':return ['aml_execution','platform_topology']
 if path=='src/aml/namespace.rs':return ['aml_syntax_namespace','platform_policy','allocator_concurrency'] if name=='new' and line<200 else ['aml_syntax_namespace','allocator_concurrency']
 if path=='src/aml/object.rs':return ['aml_execution','allocator_concurrency']
 if path=='src/aml/mod.rs':
  if name in {'opcode','pkglength','namestring','next','next_u16','next_u32','next_u64','peek','parse_field_list','Opcode','IntegerSize','from_revision'}:return ['aml_syntax_namespace']
  if 'lock' in name or name in {'object_token','BaseInterpreter','NoSendInterpreter','Interpreter'}:return ['allocator_concurrency','handler_boundary','aml_execution']
  if name in {'install_region_handler','do_field_read','do_field_write','do_native_region_read','do_native_region_write','new_from_platform'}:return ['handler_boundary','aml_execution']
  return ['aml_execution','aml_syntax_namespace','allocator_concurrency']
 if path=='src/lib.rs':
  if line>=354 and line<759:return ['handler_boundary','allocator_concurrency']
  if name in {'AcpiError','AcpiQuirks','AmlTable'}:return ['byte_table_facts','platform_policy']
  return ['table_discovery_parsing','handler_boundary']
 raise ValueError(path)
partitions={};tests=[]
for path,(digest,anchors) in source.items():
 f=doc['files'][path];categories=set()
 for key,entry in f['symbols'].items():
  n,name=key.split(':',1);cats=classify(path,name,int(n));categories.update(cats);entry['partition']=cats;entry['queue_tasks']=sorted({CATEGORIES[c] for c in cats});
  if entry['disposition']=='pending':entry['reason']='Inventoried for '+', '.join(entry['queue_tasks'])+'; no translation/execution claimed.'
 f['partition']=sorted(categories or classify(path,'',0));f['reason']='Source partition complete; implementation status remains pending until named slices land.' if f['disposition']=='pending' else f['reason']
 text=(CHECKOUT/path).read_text()
 assumptions=[name for name,pattern in [('allocation',r'\b(?:Vec|Box|BTreeMap|String|Arc|Allocator|Global|SmallVec)\b'),('concurrency',r'\b(?:Atomic\w*|Mutex|Spinlock|UnsafeCell|ObjectToken|Send|Sync)\b'),('unsafe_boundary',r'\bunsafe\b'),('host_io',r'\b(?:std::|Command|File|TcpStream)') ] if re.search(pattern,text)]
 partitions[path]={'sha256':digest,'categories':f['partition'],'assumption_flags':assumptions,'queue_tasks':sorted({CATEGORIES[c] for c in f['partition']})}
 # #[test] plus intervening cfg/serial/ignore annotations; signatures only.
 for match in re.finditer(r'#\[test\]([\s\S]*?)\bfn\s+(\w+)\s*\(',text):
  between=match[1];name=match[2];n=text[:match.start()].count('\n')+1
  license_review='crate-license-reviewed'
  if path=='tests/uacpi_examples.rs' or path=='tests/global_lock.rs' and name=='uacpi_global_lock_test':license_review='external-uacpi-origin-review-required'
  tests.append({'path':path,'line':n,'name':name,'ignored':'#[ignore' in between,'execution':'not-run','translation':'pending','origin_review':license_review,'scenario_partition':classify(path,name,n)})
files=subprocess.check_output(['git','-C',str(CHECKOUT),'ls-tree','-r','--name-only',PIN],text=True).splitlines();assets=[]
for path in files:
 if not path.endswith(('.asl','.aml')):continue
 blob=subprocess.check_output(['git','-C',str(CHECKOUT),'show',PIN+':'+path]);assert (CHECKOUT/path).read_bytes()==blob
 text=blob.decode(errors='replace')
 assets.append({'path':path,'sha256':hashlib.sha256(blob).hexdigest(),'scenario':Path(path).stem,'named_methods':re.findall(r'\bMethod\s*\(\s*([A-Za-z_0-9]+)',text),'execution':'not-run','translation':'pending','origin_review':'firmware-capture-license-unresolved' if path.endswith('pc-bios_acpi-dsdt.asl') else 'crate-license-reviewed-no-external-origin-identified','copied_content':False})
license_paths=['Cargo.toml','README.md','LICENCE-MIT','LICENCE-APACHE']
licenses={p:hashlib.sha256((CHECKOUT/p).read_bytes()).hexdigest() for p in license_paths}
for p in license_paths:assert (CHECKOUT/p).read_bytes()==subprocess.check_output(['git','-C',str(CHECKOUT),'show',PIN+':'+p])
for path in ['LICENCE-MIT','LICENCE-APACHE']:assert (CHECKOUT/path).read_bytes()==(ROOT/'licenses/rust-osdev/acpi'/path).read_bytes()
documents={'inventory.json':doc,'partitions.json':{'upstream_revision':PIN,'categories':CATEGORIES,'files':partitions},'test-scenarios.json':{'upstream_revision':PIN,'rust_tests':tests,'bytecode_assets':assets,'external_suite':{'project':'uACPI','pin':'not pinned by this checkout','execution':'not-run','origin_review':'separate license/pin review before import'},'license_evidence':licenses}}
for name,value in documents.items():
 path=DEST/name
 if args.check:
  assert json.loads(path.read_text())==value,('stale inventory metadata',name)
 else:path.write_text(json.dumps(value,indent=2)+'\n')
print(f'{len(source)} Rust files, {sum(len(x[1]) for x in source.values())} anchors, {len(tests)} Rust tests ({sum(x["ignored"] for x in tests)} ignored), {len(assets)} ASL/AML assets; inventory only.')
