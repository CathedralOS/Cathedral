"""Combined canonical Field read, Mid and SizeOf interactions."""
import importlib.util
from pathlib import Path
HERE=Path(__file__).resolve().parent
BASE=HERE.parent/'field-reads/fixtures.py'
spec=importlib.util.spec_from_file_location('field_read_fixtures',BASE);base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base)
IMPORTS,HELPERS=base.IMPORTS,base.HELPERS
render=base.render

def cases():
 rows=[];memory=bytes((i*37+19)&255 for i in range(512));integer=base.integer
 def make(label,body,bits=64,field_bits=65,setup='',result=None,error='Success',objects=None,reads=None,quotas=None,extra=''):
  data=base.table(body,bits=field_bits)
  source=base.array('input',data)+f'let prepared:Prepared=prepare_program(input,{len(data)},7,128,1024);let mut program:Program=prepared.program;'+setup
  source+='let original_field:Value=program.store.space.objects[1].value;let mut path:Path=Path {absolute:true,count:1};path.segments[0]=1313423693;let mut arguments:[Value;7];'
  api='begin'if quotas is None else'begin_limited';limits=''if quotas is None else','+','.join(map(str,quotas))
  source+=f'let mut session:Session={api}(program,path,&arguments,0,IntegerSize::{"FourBytes"if bits==32 else"EightBytes"},1024,77{limits});'+base.array('memory',memory,512)
  source+=f'let mut trace:Trace;let outcome:ExecutionOutcome=fr_run(&mut session,&memory,&mut trace,4096,{base.MAX},0);'
  chunks=(field_bits+7)//8;reads=chunks if reads is None else reads
  source+=f'let log:bool=fr_trace(&trace,&memory,{reads},{chunks},0,1,0,1,0,0,512,true);let intact:bool=fr_field(session.program.store.space.objects[1].value,original_field);'
  if isinstance(result,bytes):source+=base.array('expected',result,256)+f'let result:bool=fr_buffer(&session,{len(result)},&expected);'
  elif result is not None:source+=f'let result:bool=fr_result(&session,{result});'
  else:source+='let result:bool=!session.result.has_value;'
  source+=extra
  check=f'prepared.outcome==Outcome::Success && outcome==ExecutionOutcome::{error} && session.state==State::{"Finished"if error=="Success"else"Failed"} && log && intact && result && session.accepted=={reads} && session.issued=={reads}'
  if objects is not None:check+=f' && session.program.store.space.object_count=={objects}'
  rows.append(dict(name=label,body=source,check=check,mutation=['log','!log'],description='Exact combined behavior, Field identity, read coordinates/values and zero trace tails'))
 for bits in [32,64]:
  make(f'sizeof_field_{bits}',b'\xa4\x87FLD_',bits,error='UnsupportedValue',reads=0,objects=3)
  make(f'sizeof_malformed_field_{bits}',b'\xa4\x87FLD_',bits,error='UnsupportedValue',reads=0,objects=3,setup='let changed:Value=fr_malformed(program.store.space.objects[1].value,5);program.store.space.objects[1].value=changed;')
  make(f'object_type_field_{bits}',b'\xa4\x8eFLD_',bits,result=5,reads=0,objects=3)
  make(f'mid_integer_field_{bits}',b'\xa4\x9eFLD_'+integer(1)+integer(1)+b'\0',bits,field_bits=16,result=memory[1:2],objects=4)
  make(f'mid_buffer_field_{bits}',b'\xa4\x9eFLD_'+integer(1)+integer(3)+b'\0',bits,result=memory[1:4],objects=5)
  make(f'sizeof_mid_local_{bits}',b'\x9eFLD_'+integer(1)+integer(3)+b'\x60\xa4\x87\x60',bits,result=3,objects=6)
 make('mid_read_consumes_last_slot',b'\xa4\x9eFLD_'+integer(1)+integer(3)+b'\0',setup='program.store.space.object_count=63;',error='Capacity',objects=64,extra='let retained:bool=session.program.store.bytes.blocks[63].initialized && session.program.store.bytes.blocks[63].length==9;')
 rows[-1]['check']+=' && retained'
 make('mid_field_target_after_read',b'\xa4\x9eFLD_'+integer(1)+integer(3)+b'FLD_',error='UnresolvedRegion',objects=4)
 make('mid_read_preflight_capacity',b'\xa4\x9eFLD_'+integer(1)+integer(3)+b'\0',setup='program.store.space.object_count=64;',error='Capacity',objects=64,reads=0)
 make('mid_result_quota_preserves_reads',b'\xa4\x9eFLD_'+integer(1)+integer(3)+b'\0',error='Capacity',objects=5,quotas=(64,2))
 return rows
