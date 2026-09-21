#!/usr/bin/env python3
"""Compact initialized-data fixtures; shared actual Omega comparisons."""
import generate
OPS=['from_start','containing','from_pfn','pfn','arithmetic','difference','range_count','range_bytes','range_at','iterator']
HEAD=generate.IMPORTS+generate.HELPERS+'''data Fixture [copy] {op:u64;start:u64;end:u64;size:u64;index:u64;inclusive:bool;reverse:bool;ok:bool;value:u64;failed:bool;out_start:u64;out_end:u64;}
data FixtureRows [copy] {rows:[Fixture;48];}
machine ef_compact_case(profile:State,row:Fixture)->bool {
 transition row.op {
'''
for i,op in enumerate(OPS):HEAD+=f'{i} -> case_{i}(profile,row)\n'
HEAD+=' _ -> (false)}\n'
for i,op in enumerate(OPS):
 a='row.start';b='row.end';size='row.size';n='row.index';inc='row.inclusive';rev='row.reverse'
 if op in OPS[:4]:call=f'{op}(profile,{a},{size})'
 elif op=='arithmetic':call=f'arithmetic(profile,{a},{size},{n},{rev})'
 elif op=='difference':call=f'difference(profile,{b},{a},{size})'
 elif op in ['range_count','range_bytes']:call=f'{op}(profile,{a},{b},{size},{inc})'
 else:call=f'{"iterator_step"if op=="iterator"else"range_at"}(profile,{a},{b},{size},{inc},{n},{rev})'
 HEAD+=f'state case_{i}(profile:State,row:Fixture)->bool {{\n'
 if op=='iterator':HEAD+=f'let result:IteratorResult=encrypted_frames::{call};let good:bool=ef_fixture_iterator(result,row.ok,row.value,row.failed,row.out_start,row.out_end);good\n'
 else:
  HEAD+=f'let result:NumberResult=encrypted_frames::{call};let good:bool=ef_fixture_number(result,row.ok,row.value);\n'
  if op=='arithmetic':HEAD+='let step:OverflowingStep=encrypted_frames::arithmetic_overflowing(profile,row.start,row.size,row.index,row.reverse);\ntransition row.ok {true -> (good && !step.overflow && step.value==row.value) _ -> (good && step.overflow && step.value==row.start)}\n'
  else:HEAD+='good\n'
 HEAD+='}\n'
HEAD+='''}
machine ef_compact_rows(profile:State,rows:&FixtureRows,index:u64,count:u64,good:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {
 let next:bool=ef_compact_at(profile,rows,index,count,good);
 transition index<count {true -> ef_compact_rows(profile,rows,index+1,count,next) _ -> (good)}
}
machine ef_compact_at(profile:State,rows:&FixtureRows,index:u64,count:u64,good:bool)->bool {
 transition index<count && index<48 {true -> item(profile,rows.rows[index],good) _ -> (good)}
 state item(profile:State,row:Fixture,good:bool)->bool {let result:bool=ef_compact_case(profile,row);good && result}
}
data Suite{}
'''
def render(g,control):
 name=g['name']+('_control'if control else'_positive');text='machine Suite::'+name+'(&mut self)->i32 {let mut profile:State=initial();\nlet mut all_configured:bool=true;\n'
 if g['profile']!='disabled':
  for j,part in enumerate(g['profile'].split('_')):
   kind='EncryptedBit'if part[0]=='e'else'SharedBit';text+=f'let configured{j}:bool=configure(&mut profile,Configuration::{kind} {{position:{part[1:]}}});all_configured=all_configured && configured{j};\n'
 text+='let mut rows:FixtureRows=FixtureRows {};\n'
 for i,r in enumerate(g['cases']):
  row={k:v for k,v in r.items()if k in ['start','end','size','index','inclusive','reverse','ok','value','failed','out_start','out_end']};row['op']=OPS.index(r['op'])
  if control and i==0:
   key='failed'if r['op']=='iterator'else'ok';row[key]=not row[key]
  text+=f'rows.rows[{i}]=Fixture {{'+','.join(f'{k}:{str(v).lower()}'for k,v in row.items())+'};\n'
 text+=f'let good:bool=ef_compact_rows(profile,&rows,0,{len(g["cases"])},all_configured);transition good {{true -> (0) _ -> (1)}}\n}}\n'
 return text
