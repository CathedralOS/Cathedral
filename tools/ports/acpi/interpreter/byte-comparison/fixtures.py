#!/usr/bin/env python3
"""Original comparison vectors; expectations distinguish primary and pinned ordering."""
import json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
PIN='257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5'
IMPORTS='''use helpers::byte_comparison;
use helpers::byte_comparison::Comparison;
use helpers::byte_comparison::BufferOrdering;
'''
HELPERS=''
def cases():
 pairs=[('empty',[],[]),('empty_left',[],[65]),('empty_right',[65],[]),('same',[65,66],[65,66]),('less',[65],[66]),('greater',[66],[65]),('prefix',[65],[65,66]),('prefix_reverse',[65,66],[65]),('opposed_length',[255],[0,0]),('opposed_ascii',[90],[65,65]),('opposed_reverse',[65,65],[90]),('later_less',[65,66,99],[65,67,0]),('later_greater',[65,67,0],[65,66,99]),('unsigned',[128],[127]),('zero',[0],[1]),('later_nul',[65,0],[66,67]),('later_nonascii',[65,255],[66,67]),('all256',[65]*256,[65]*256),('last256',[65]*255+[66],[65]*256),('first256',[66]+[65]*255,[65]*256),('length255',[65]*255,[65]*256),('empty_nonascii',[],[255])]
 rows=[]
 for name,left,right in pairs:
  for profile in ['Lexicographic','PinnedLengthFirst','String']:
   lexical=(left>right)-(left<right);pin=((len(left)>len(right))-(len(left)<len(right))) or lexical
   invalid=profile=='String' and any(x==0 or x>=128 for x in left+right)
   result='Encoding' if invalid else {-1:'Less',0:'Equal',1:'Greater'}[pin if profile=='PinnedLengthFirst' else lexical]
   rows.append(dict(name=name+'_'+profile,left=left,right=right,a=len(left),b=len(right),profile=profile,result=result))
 for profile in ['Lexicographic','PinnedLengthFirst','String']:
  for n,(a,b) in enumerate([(257,0),(0,257),(18446744073709551615,0),(0,18446744073709551615),(257,257)]):rows.append(dict(name=f'capacity_{profile}_{n}',left=[0],right=[255],a=a,b=b,profile=profile,result='Capacity'))
 return rows

def render(row,control=False,machine='test_result'):
 signature=machine+'(&mut self)'if'::'in machine else machine+'()'
 text=f'machine {signature}->i32 {{\nlet mut left:[u8;256];let mut right:[u8;256];\n'
 for var in ['left','right']:
  for index,value in enumerate(row[var]):
   if value:text+=f'{var}[{index}]={value};\n'
 args=f'&left,{row["a"]},&right,{row["b"]}'
 call=f'byte_comparison::compare_strings({args})'if row['profile']=='String'else f'byte_comparison::compare_buffers({args},BufferOrdering::{row["profile"]})'
 want=row['result'];want=('Equal'if want!='Equal'else'Less')if control else want
 text+=f'let observed:Comparison={call};transition observed {{Comparison::{want} -> (0) _ -> (1)}} }}\n'
 return text
if __name__=='__main__':
 paths={HERE/'cases.json':json.dumps(cases(),indent=2)+'\n',HERE/'main.omg':IMPORTS+render(cases()[0])+'''const TEST_RESULT:i32=test_result();
machine require_ok(value:i32) requires value==0;{}
data Main{}
machine Main::main(&mut self){require_ok(TEST_RESULT);}
'''}
 for path,text in paths.items():
  if '--check'in sys.argv:assert path.read_text()==text,path
  else:path.write_text(text)
 print(len(cases()),'comparison scenarios verified')
