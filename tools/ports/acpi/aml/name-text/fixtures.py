#!/usr/bin/env python3
"""Original bounded ASL text vectors, with explicit primary/pin differences."""
import json,re
from pathlib import Path
import reference,guard_fixtures
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
IMPORTS='''use aml::name_text;
use aml::names;
use aml::name_text::TextResult;
use aml::model::Outcome;
use aml::model::Path;
use aml::model::PathResult;
'''
HELPERS='''machine equal_path(a:Path,b:Path)->bool {
 let rows:bool=equal_segments(a,b,0,16,true);
 a.absolute==b.absolute && a.parents==b.parents && a.count==b.count && rows
}
machine equal_segments(a:Path,b:Path,index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let good:bool=segment_equal(a,b,index);transition index<count {true -> equal_segments(a,b,index+1,count,prior && good) _ -> (prior)}}
machine segment_equal(a:Path,b:Path,index:u64)->bool {transition index<16 {true -> (a.segments[index]==b.segments[index]) _ -> (true)}}
machine equal_bytes(a:&[u8;256],b:&[u8;256],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let good:bool=byte_equal(a,b,index);transition index<count {true -> equal_bytes(a,b,index+1,count,prior && good) _ -> (prior)}}
machine byte_equal(a:&[u8;256],b:&[u8;256],index:u64)->bool {transition index<256 {true -> (a[index]==b[index]) _ -> (true)}}
machine text_good(result:TextResult,expected:&[u8;256],size:u64)->bool {
 transition result {TextResult::Text {length,bytes} -> compare(length,&bytes,expected,size) _ -> (false)}
 state compare(length:u64,bytes:&[u8;256],expected:&[u8;256],size:u64)->bool {let same:bool=equal_bytes(bytes,expected,0,256,true);length==size && same}
}
machine text_bad(result:TextResult,expected:Outcome)->bool {transition result {TextResult::Failure {error} -> (error==expected) _ -> (false)}}
machine parse_good(input:&[u8;256],size:u64,expected:Path,text:&[u8;256],text_size:u64)->bool {
 let parsed:PathResult=name_text::parse(input,size);let same:bool=equal_path(parsed.value,expected);
 let rendered:TextResult=name_text::format(parsed.value);let formatted:bool=text_good(rendered,text,text_size);
 let again:PathResult=name_text::parse(text,text_size);let back:bool=equal_path(again.value,expected);
 parsed.outcome==Outcome::Success && parsed.next==size && parsed.offset==0 && same && formatted && again.outcome==Outcome::Success && back
}
machine parse_bad(input:&[u8;256],size:u64,expected:Outcome,offset:u64)->bool {
 let parsed:PathResult=name_text::parse(input,size);let empty:bool=equal_path(parsed.value,Path {});
 parsed.outcome==expected && parsed.next==0 && parsed.offset==offset && empty
}
'''
def expectation(data,length=None):
 length=len(data) if length is None else length
 if length>256:return dict(error='Truncated',offset=0)
 if not length:return dict(error='InvalidName',offset=0)
 data=data[:length];absolute=data.startswith(b'\\');at=int(absolute);parents=0
 while at<len(data) and data[at]==94:
  if absolute:return dict(error='InvalidName',offset=at)
  if parents==16:return dict(error='Capacity',offset=at)
  parents+=1;at+=1
 words=[]
 if at<len(data):
  for part in data[at:].split(b'.'):
   if not part:return dict(error='InvalidName',offset=at)
   for i,byte in enumerate(part):
    allowed=byte==95 or 65<=byte<=90 or 97<=byte<=122 or (i>0 and 48<=byte<=57)
    if i>=4 or not allowed:return dict(error='InvalidName',offset=at+i)
   at+=len(part)
   if len(words)==16:return dict(error='Capacity',offset=at)
   words.append(part.upper().ljust(4,b'_'));at+=1
 text=(b'\\' if absolute else b'^'*parents)+b'.'.join(words)
 return dict(absolute=absolute,parents=parents,words=[int.from_bytes(word,'little')for word in words],text_hex=text.hex())
def cases():
 rows=[];seen=set()
 values=[bytes.fromhex(row['text_hex']) for row in reference.cases() if row['kind']=='path']
 values += [bytes([x]) for x in [1,31,47,57,58,64,90,91,95,96,122,123,127,128,255]]
 values += [b'A'+bytes([x]) for x in [1,31,47,48,57,58,64,90,91,95,96,122,123,127,128,255]]
 values += [b'A'*256,b'^'*256,b'A\0B',b'A.B\0',b'\\a.b0._c.dEf']
 for raw in values:
  if raw in seen:continue
  seen.add(raw);expected=expectation(raw);rows.append(dict(name=f'parse_{len(rows):03}',kind='parse',input_hex=raw.hex(),length=len(raw),expected=expected))
 for size in [257,2**64-1]:rows.append(dict(name='length_'+str(size),kind='parse',input_hex='',length=size,expected=expectation(b'',size)))
 for name,absolute,parents,count,words in [('empty',False,0,0,[]),('root_parent',True,1,0,[]),('count17',False,0,17,[]),('parents17',False,17,0,[]),('count_max',False,0,2**64-1,[]),('parents_max',False,2**64-1,0,[]),('zero_segment',False,0,1,[0]),('digit_lead',False,0,1,[int.from_bytes(b'0ABC','little')]),('lowercase',False,0,1,[int.from_bytes(b'aBCD','little')]),('bad_tail',False,0,1,[int.from_bytes(b'ABC!','little')])]:
  rows.append(dict(name='format_'+name,kind='format_bad',absolute=absolute,parents=parents,count=count,words=words))
 # Every valid count/prefix combination is covered as direct formatter input,
 # not only a value constructed by parse. Distinct segments expose indexing.
 for count in [0,1,2,16]:
  for absolute,parents in [(True,0),(False,1),(False,16)]:
   words=[f'A{n:03}'.encode() for n in range(count)]
   text=(b'\\'if absolute else b'^'*parents)+b'.'.join(words)
   rows.append(dict(name=f'format_{int(absolute)}_{parents}_{count}',kind='format_good',absolute=absolute,parents=parents,count=count,words=[int.from_bytes(x,'little')for x in words],text_hex=text.hex()))
 rows += [dict(row,name='guard_'+row['name'],kind='guard')for row in guard_fixtures.cases()]
 return rows

def array(name,data):return f'let mut {name}:[u8;256];'+''.join(f'{name}[{i}]={x};'for i,x in enumerate(data))
def path(name,absolute,parents,count,words):return f'let mut {name}:Path=Path {{absolute:{str(absolute).lower()},parents:{parents},count:{count}}};'+''.join(f'{name}.segments[{i}]={x};'for i,x in enumerate(words))
def body(row,control=False):
 if row['kind']=='guard':return row['control'if control else'body']+'\n'
 if row['kind']=='parse':
  setup=array('input',bytes.fromhex(row['input_hex']));expected=row['expected']
  if 'error'in expected:
   error='Success'if control else expected['error'];check=f'parse_bad(&input,{row["length"]},Outcome::{error},{expected["offset"]})'
  else:
   text=bytearray.fromhex(expected['text_hex']);setup+=path('expected',expected['absolute'],expected['parents'],len(expected['words']),expected['words'])
   # Mutate an actual expected output byte, including zero tail for root-only.
   if control:text[0]^=1
   setup+=array('text',text);check=f'parse_good(&input,{row["length"]},expected,&text,{len(text)})'
 else:
  setup=path('value',row['absolute'],row['parents'],row['count'],row['words'])+'let result:TextResult=name_text::format(value);'
  if row['kind']=='format_bad':check='text_bad(result,Outcome::'+('Success'if control else'InvalidName')+')'
  else:
   text=bytearray.fromhex(row['text_hex']);size=len(text)
   # Check the full zero tail: control changes byte255 while logical text stays identical.
   setup+=array('expected',text)+('expected[255]=1;'if control else'');check=f'text_good(result,&expected,{size})'
 return setup+'\nlet good:bool='+check+';transition good {true -> (0) _ -> (1)}\n'
def render(rows):
 text=IMPORTS+HELPERS+'data Suite {}\n';names=[]
 for row in rows:
  for control in [False,True]:
   name='Suite::'+row['name']+('_control'if control else'_positive');text+='machine '+name+'(&mut self)->i32 {\n'+body(row,control)+'}\n';names.append(name+'='+str(int(control)))
 return text,names
if __name__=='__main__':
 rows=cases();(HERE/'cases.json').write_text(json.dumps(rows,indent=2,sort_keys=True)+'\n');print(len(rows),'name text pairs')
