"""Focused ordinary Path and physical-index guard regressions."""
def cases():
 rows=[]
 def add(name,setup,check,token,changed):
  body=setup+'let good:bool='+check+';transition good {true -> (0) _ -> (1)}';assert body.count(token)==1
  rows.append(dict(name=name,body=body,control=body.replace(token,changed)))
 for value in [17,2**63,2**64-1]:
  for field in ['count','parents']:
   setup=f'let value:Path=Path {{absolute:true,{field}:{value}}};'
   add(f'valid_{field}_{value}',setup+'let actual:bool=names::path_valid(value);','actual==false','actual==false','actual==true')
   setup=f'let value:Path=Path {{{field}:{value}}};'
   good='true'if field=='parents'else'false'
   add(f'equal_{field}_{value}',setup+'let actual:bool=names::path_equal(value,value);','actual=='+good,'actual=='+good,'actual=='+('false'if good=='true'else'true'))
   setup=f'let value:Path=Path {{absolute:true,{field}:{value}}};'
   add(f'parent_{field}_{value}',setup+'let actual:PathResult=names::parent(value);','actual.outcome==Outcome::NotAbsolute','Outcome::NotAbsolute','Outcome::Success')
   setup=f'let value:Path=Path {{{field}:{value}}};let scope:Path=Path {{absolute:true}};'
   add(f'resolve_value_{field}_{value}',setup+'let actual:PathResult=names::resolve(value,scope);','actual.outcome==Outcome::InvalidName','Outcome::InvalidName','Outcome::Success')
   setup=f'let value:Path=Path {{}};let scope:Path=Path {{absolute:true,{field}:{value}}};'
   add(f'resolve_scope_{field}_{value}',setup+'let actual:PathResult=names::resolve(value,scope);','actual.outcome==Outcome::InvalidName','Outcome::InvalidName','Outcome::Success')
 for index in [16,17,2**63,2**64-1]:
  setup='let mut value:Path=Path {};value.segments[0]=0x44434241;'+f'let actual:u32=names::segment_at(value,{index});let out:Path=names::segment_set(value,{index},7);'
  add('physical_'+str(index),setup,'actual==0 && equal_path(value,out)','actual==0','actual==1')
 # Keep path_equal comparison policy, including asymmetric invalid counts/parents.
 add('equal_parent_mismatch','let a:Path=Path {parents:18446744073709551615};let b:Path=Path {parents:0};let actual:bool=names::path_equal(a,b);','actual==false','actual==false','actual==true')
 return rows
