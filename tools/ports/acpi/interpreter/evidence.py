#!/usr/bin/env python3
"""Complete two-file anchor inventory with partial operation mappings kept pending."""
import argparse,hashlib,importlib.util,json,subprocess
from pathlib import Path
import fixtures
ROOT=fixtures.ROOT;HERE=fixtures.HERE;DEST=ROOT/'source/libraries/acpi/interpreter';UP=ROOT/'reference_code/rust-osdev/acpi'
spec=importlib.util.spec_from_file_location('port_inventory',ROOT/'tools/ports/inventory.py');api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)

def target(file,anchor):return {'path':'source/libraries/acpi/interpreter/'+file,'anchor':anchor}
def evidence():
 inv=api.snapshot(UP,fixtures.PIN,['src/aml/mod.rs','src/aml/object.rs'],'https://github.com/rust-osdev/acpi')
 inv['scope']='ACPI005 pure integer and initialized-byte helper slice; pending methods retain missing object/context/namespace behavior. No whole interpreter or full source-file translation claimed.'
 partial={'do_binary_maths':('integers.omg','pub machine binary'),'do_unary_maths':('integers.omg','pub machine find_set_left'),'do_logical_op':('integers.omg','pub machine logical'),'do_from_bcd':('bcd.omg','pub machine from_bcd'),'do_to_bcd':('bcd.omg','pub machine to_bcd'),'do_to_buffer':('conversions.omg','pub machine integer_to_buffer'),'to_buffer':('conversions.omg','pub machine string_to_buffer'),'to_integer':('conversions.omg','pub machine buffer_to_integer'),'read_buffer_field':('buffer_fields.omg','pub machine field_to_integer'),'write_buffer_field':('buffer_fields.omg','pub machine copy_bits'),'do_to_string':('conversions.omg','pub machine buffer_to_string'),'do_mid':('conversions.omg','pub machine mid')}
 unit={'test_copy_bits':'upstream_copy_bits','buffer_to_integer':'upstream_buffer_to_integer','buffer_field_to_integer':'upstream_buffer_field_to_integer','buffer_field_to_4_byte_integer':'upstream_buffer_field_to_4_byte_integer','buffer_field_to_8_byte_integer':'upstream_buffer_field_to_8_byte_integer'}
 for path,file in inv['files'].items():
  file['reason']='Only bounded pure suboperations translated; complete interpreter/object behavior remains pending.';file['targets']=[target('PORT.md','## Source map')]
  for key,row in file['symbols'].items():
   line,name=key.split(':',1);line=int(line)
   row['reason']='Outside implemented pure-helper slice; later AML interpreter/value/object/context work remains pending.'
   mapping=None
   if path.endswith('/mod.rs') and name in ['IntegerSize','from_revision','FourBytes','EightBytes']:
    mapping=target('integers.omg',{'IntegerSize':'pub data IntegerSize','from_revision':'pub machine from_revision','FourBytes':'case FourBytes','EightBytes':'case EightBytes'}[name])
   elif path.endswith('/object.rs') and name=='copy_bits':mapping=target('buffer_fields.omg','pub machine copy_bits')
   elif path.endswith('/object.rs') and name in unit and line>=604:
    mapping={'path':'tools/ports/acpi/interpreter/main.omg','anchor':'machine test_'+unit[name]}
   if mapping:
    row.update(disposition='translated',targets=[mapping]);row.pop('reason',None)
   elif name in partial:
    row['targets']=[target(*partial[name])];row['reason']='Only pure scalar/byte suboperation implemented. Generic Object conversion, reference/token handling, target stores, method context and/or retirement remain pending; not a complete method translation.'
 inv['test_sources']={}
 for name in ['tests/to_integer.asl','tests/to_x.asl','tests/incdec.asl','tests/logical_not.asl']:
  blob=subprocess.check_output(['git','-C',str(UP),'show',fixtures.PIN+':'+name])
  if (UP/name).read_bytes()!=blob:raise SystemExit('Test source differs from pin: '+name)
  inv['test_sources'][name]=hashlib.sha256(blob).hexdigest()
 return inv

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--check',action='store_true');args=p.parse_args();value=evidence();text=json.dumps(value,indent=2,sort_keys=True)+'\n';path=DEST/'inventory.json'
 if args.check:
  if not path.exists()or path.read_text()!=text:raise SystemExit('Interpreter inventory differs')
 else:path.write_text(text)
 print('Pinned interpreter helper source map '+('verified'if args.check else'generated'));print(api.check(value,UP))
if __name__=='__main__':main()
