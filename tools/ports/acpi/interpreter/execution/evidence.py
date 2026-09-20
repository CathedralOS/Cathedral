#!/usr/bin/env python3
"""Reproduce pinned complete source inventory and honest partial execution mappings."""
import argparse,hashlib,importlib.util,json,subprocess
from pathlib import Path
import fixtures
ROOT=fixtures.ROOT;UP=ROOT/'reference_code/rust-osdev/acpi';DEST=ROOT/'source/libraries/acpi/interpreter/execution'
PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
spec=importlib.util.spec_from_file_location('port_inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
def evidence():
 value=api.snapshot(UP,PIN,['src/aml/mod.rs','src/aml/object.rs','src/aml/namespace.rs'],'https://github.com/rust-osdev/acpi')
 value['scope']='ACPI005 bounded integer method execution. Complete source files and generic Rust methods remain pending; target mappings identify implemented suboperations only. No omission overlay.'
 mapping={'MethodContext':('execution_model.omg','pub data Frame'),'new_from_method':('frames.omg','pub machine new_frame'),'Block':('execution_model.omg','pub data Block'),'BlockKind':('control.omg','pub machine finish_block'),'OpInFlight':('execution_model.omg','pub data Operation'),'contribute_arg':('operands.omg','pub machine contribute_integer'),'start_new_block':('control.omg','pub machine enter_block'),'do_execute_method':('engine.omg','pub machine run_method'),'do_binary_maths':('retire.omg','pub machine retire_operation'),'do_unary_maths':('operator_specs.omg','pub machine compute_integer'),'do_logical_op':('operator_specs.omg','pub machine compute_integer'),'do_store':('targets.omg','pub machine store_target'),'do_copy_object':('targets.omg','pub machine copy_integer_target'),'do_from_bcd':('operator_specs.omg','pub machine compute_integer'),'do_to_bcd':('operator_specs.omg','pub machine compute_integer'),'opcode':('decode_execution.omg','pub machine decode_term'),'ResolveBehaviour':('decode_execution.omg','pub machine decode_target')}
 for path,file in value['files'].items():
  file['reason']='Bounded integer method profile only; complete generic interpreter/value/namespace semantics remain pending.'
  file['targets']=[{'path':'source/libraries/acpi/interpreter/execution/PORT.md','anchor':'## Source and behavior map'}]
  for key,row in file['symbols'].items():
   row['reason']='Outside bounded integer execution profile, or only partial generic behavior implemented. Remains future work.'
   if path=='src/aml/mod.rs'and key.split(':',1)[1]in mapping:
    target,anchor=mapping[key.split(':',1)[1]];row['targets']=[{'path':'source/libraries/acpi/interpreter/execution/'+target,'anchor':anchor}]
    row['reason']='Partial bounded integer/control/target translation; generic objects, references, dynamic namespace, package/field or external service behavior remains pending. See execution PORT source map.'
 value['test_sources']={}
 for path in ['tests/method.asl','tests/while.asl','tests/incdec.asl','tests/logical_not.asl']:
  blob=subprocess.check_output(['git','-C',str(UP),'show',PIN+':'+path])
  if (UP/path).read_bytes()!=blob:raise SystemExit('Test source differs from pin: '+path)
  value['test_sources'][path]=hashlib.sha256(blob).hexdigest()
 return value

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--check',action='store_true');a=p.parse_args();value=evidence();text=json.dumps(value,indent=2,sort_keys=True)+'\n';path=DEST/'inventory.json'
 if a.check:
  if not path.exists()or path.read_text()!=text:raise SystemExit('Execution inventory differs')
 else:path.write_text(text)
 print(api.check(value,UP))
if __name__=='__main__':main()
