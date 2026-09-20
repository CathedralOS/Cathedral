from pathlib import Path
import json
H=Path(__file__).resolve().parent;C=H/'cases';C.mkdir(exist_ok=True)
base='''// SPDX-License-Identifier: MIT OR Apache-2.0
// Original Cathedral fixtures; no firmware or external uACPI fixture content.
'''
model=['Outcome','Read','Envelope','Span','Path','PathResult','Value','Object','Namespace','NamespaceResult','Lookup','LevelKind','ValueState','LoadState','ReferenceState']
imports=''.join(f'use aml::model::{x};\n' for x in model)
modules={'bytes':['read_le','pkg_length','envelope','opcode'],'names':['namestring','resolve','root','segment_at','path_equal','parent'],'namespace':['empty','child','insert','alias','get','search','add_level','object_at','allocate','entry_at'],'values':['parse_value','integer','atom'],'loader':['load'],'references':['resolve_reference']}
helpers='''
machine single(segment:u32,absolute:bool)->Path {let mut out:Path=Path {absolute:absolute,count:1};out.segments[0]=segment;out}
machine integer_is(space:Namespace,id:u64,expected:u64)->bool {
 let object:Object=object_at(space,id);
 transition object.value {Value::Integer {number} -> compare(number,expected) _ -> (false)}
 state compare(number:u64,expected:u64)->bool {number==expected}
}
machine package_first(space:Namespace,id:u64)->u64 {
 let object:Object=object_at(space,id);
 transition object.value {Value::Package {first,count} -> (first) _ -> (64)}
}
machine package_is(space:Namespace,id:u64,expected:u64)->bool {
 let object:Object=object_at(space,id);
 transition object.value {Value::Package {first,count} -> compare(count,expected) _ -> (false)}
 state compare(count:u64,expected:u64)->bool {count==expected}
}
machine reference_is(space:Namespace,id:u64,scope:Path)->bool {
 let object:Object=object_at(space,id);
 transition object.value {Value::NameReference {name,scope} -> compare(scope) _ -> (false)}
 state compare(scope:Path)->bool {scope.absolute && scope.count==1 && scope.segments[0]==0x30564544}
}
machine method_is(space:Namespace,id:u64,unit:u64,start:u64,end:u64)->bool {
 let object:Object=object_at(space,id);
 transition object.value {Value::Method {flags,body} -> compare(flags,body,unit,start,end) _ -> (false)}
 state compare(flags:u8,body:Span,unit:u64,start:u64,end:u64)->bool {flags==0x39 && body.unit==unit && body.start==start && body.end==end}
}
'''
def arr(b,name='input'):
 return f'let mut {name}:[u8;1024];\n'+''.join(f'{name}[{i}]={v};\n' for i,v in enumerate(b))
def pkg(op,payload):
 for n in range(1,5):
  length=len(payload)+n
  if length < (64 if n==1 else 1<<(4+8*(n-1))):break
 if n==1:enc=bytes([length])
 else:enc=bytes([((n-1)<<6)|(length&15)])+bytes((length>>(4+8*j))&255 for j in range(n-1))
 return bytes(op)+enc+payload
def name(n,value):return b'\x08'+n.encode()+value
def integer(n):return b'\x0a'+bytes([n])
def scope(n,body,op=b'\x10'):return pkg(op,n.encode()+body)
def finish(name_,body,expr,mutation,groups,description):
 src=base+imports+''.join(f'use aml::{m}::{f};\n' for m in groups for f in modules[m])+ (helpers if 'namespace' in groups else '')
 src+='\nmachine test()->u64 {\n'+body+'\n transition '+expr+' {true -> (0) _ -> (1)}\n}\nconst TEST_RESULT:u64=test();\nmachine require_ok(result:u64) requires result==0; {}\ndata Main {}\nmachine Main::main(&mut self){require_ok(TEST_RESULT);}\n'
 assert src.count(mutation[0])==1,(name_,mutation)
 (C/(name_+'.omg')).write_text(src)
 cases[name_]={'description':description,'mutation':list(mutation),'groups':groups}
cases={}
finish('bytes-endian',arr([0x12,0x34,0x56,0x78])+'''let one:Read=read_le(&input,4,0,4);let truncated:Read=read_le(&input,3,0,4);''','one.outcome==Outcome::Success && one.value==0x78563412 && one.next==4 && truncated.outcome==Outcome::Truncated',('one.value==0x78563412','one.value==0x78563413'),['bytes'],'Little-endian initialized input, exact end, and truncated width')
finish('package-length',arr([0x41,4,0x71,4,0x80,0,1])+'''let length:Read=pkg_length(&input,7,0);let reserved:Read=pkg_length(&input,7,2);let larger:Read=pkg_length(&input,7,4);let crossing:Envelope=envelope(&input,7,0);''','length.value==65 && length.next==2 && reserved.outcome==Outcome::BadEncoding && larger.value==4096 && crossing.outcome==Outcome::BadEncoding',('length.value==65','length.value==64'),['bytes'],'Following-byte nibble encoding, reserved bits, three-byte length, enclosing bounds')
finish('opcode-prefix',arr([0x92,0x93,0x92,0x11,0x5b,0x82,0x92])+'''let a:Read=opcode(&input,7,0);let b:Read=opcode(&input,7,2);let c:Read=opcode(&input,7,4);let d:Read=opcode(&input,7,6);let e:Read=opcode(&input,5,4);''','a.value==0x9293 && a.next==2 && b.value==0x92 && b.next==3 && c.value==0x5b82 && d.value==0x92 && d.next==7 && e.outcome==Outcome::Truncated',('a.value==0x9293','a.value==0x9294'),['bytes'],'LNot composite lookahead, standalone final LNot and truncated extended opcode')
finish('names-forms',arr(b'\\\x2eDEV0OBJ0^^\x2f\x02AAAABBBB')+'''let a:PathResult=namestring(&input,22,0);let b:PathResult=namestring(&input,22,10);let r:Path=root();let invalid:PathResult=resolve(b.value,r);''','a.outcome==Outcome::Success && a.value.absolute && a.value.count==2 && a.next==10 && b.outcome==Outcome::Success && b.value.parents==2 && b.value.count==2 && invalid.outcome==Outcome::AboveRoot',('a.next==10','a.next==9'),['bytes','names'],'Absolute dual and parent-prefixed multi names; above-root resolution rejected')
finish('names-invalid',arr(b'1BAD\x2f\x00')+'''let invalid:PathResult=namestring(&input,6,0);let zero:PathResult=namestring(&input,6,4);let short:PathResult=namestring(&input,3,0);''','invalid.outcome==Outcome::InvalidName && invalid.offset==0 && zero.outcome==Outcome::BadEncoding && short.outcome==Outcome::Truncated',('invalid.offset==0','invalid.offset==1'),['bytes','names'],'Invalid lead, zero multi-count and incomplete NameSeg are explicit errors')
finish('namespace-alias', '''let start:Namespace=empty();let a:Path=single(0x41414141,true);let b:Path=single(0x42424242,true);
let old:Value=Value::Integer {number:42};let new:Value=Value::Integer {number:7};
let first:NamespaceResult=insert(start,a,old);let aliasing:NamespaceResult=alias(first.space,a,b);let second:NamespaceResult=insert(aliasing.space,a,new);
let original:Lookup=get(second.space,a);let copied:Lookup=get(second.space,b);let collision:NamespaceResult=alias(second.space,a,b);let unchanged:Lookup=get(collision.space,b);
let old_ok:bool=integer_is(second.space,copied.object,42);let new_ok:bool=integer_is(second.space,original.object,7);''','first.outcome==Outcome::Success && aliasing.outcome==Outcome::Success && original.object!=copied.object && old_ok && new_ok && collision.outcome==Outcome::NameCollision && unchanged.object==copied.object && second.space.object_count==2',('second.space.object_count==2','second.space.object_count==3'),['names','namespace'],'Alias retains original object identity across source rebinding; collision is transactional')
finish('namespace-shadow', '''let start:Namespace=empty();let r:Path=root();let d:Path=single(0x30564544,true);let a:Path=single(0x41414141,true);let rel:Path=single(0x41414141,false);
let subtree:NamespaceResult=add_level(start,d,LevelKind::Device);let root_value:Value=Value::Integer {number:5};let local_value:Value=Value::Integer {number:9};
let root_insert:NamespaceResult=insert(subtree.space,a,root_value);let local:PathResult=child(d,0x41414141);let local_insert:NamespaceResult=insert(root_insert.space,local.value,local_value);
let shadow:Lookup=search(local_insert.space,rel,d);let explicit:Lookup=search(local_insert.space,a,d);let fallback:Lookup=search(root_insert.space,rel,d);
let shadow_ok:bool=integer_is(local_insert.space,shadow.object,9);let root_ok:bool=integer_is(local_insert.space,explicit.object,5);let fallback_ok:bool=integer_is(root_insert.space,fallback.object,5);''','shadow.outcome==Outcome::Success && shadow_ok && root_ok && fallback_ok && shadow.path.count==2',('shadow.path.count==2','shadow.path.count==1'),['names','namespace'],'Nearest single-segment shadow, explicit root lookup, ancestor fallback')
finish('namespace-errors', '''let start:Namespace=empty();let relative:Path=single(0x41414141,false);let absolute:Path=single(0x41414141,true);let value:Value=Value::Integer {number:3};
let get_bad:Lookup=get(start,relative);let level_bad:NamespaceResult=add_level(start,relative,LevelKind::Scope);let insert_bad:NamespaceResult=insert(start,relative,value);
let resolved:PathResult=resolve(absolute,relative);let r:Path=root();let parent_bad:PathResult=parent(r);''','get_bad.outcome==Outcome::NotAbsolute && level_bad.outcome==Outcome::NotAbsolute && insert_bad.outcome==Outcome::NotAbsolute && resolved.outcome==Outcome::NotAbsolute && parent_bad.outcome==Outcome::AboveRoot && insert_bad.space.object_count==0',('insert_bad.space.object_count==0','insert_bad.space.object_count==1'),['names','namespace'],'Relative namespace paths and starting scopes rejected; root has no parent; failed insertion rolls back allocation')
# Nested package with underfilled inner package, then outer integer.
b=pkg(b'\x12',bytes([2])+pkg(b'\x12',bytes([2])+b'\x01')+integer(9))
finish('values-nested',arr(b)+f'''let r:Path=root();let start:Namespace=empty();let parsed:ValueState=parse_value(&input,{len(b)},0,17,r,start,false,10);
let first:u64=package_first(parsed.space,parsed.object);let inner:Object=object_at(parsed.space,first);let leaf:u64=package_first(parsed.space,first);let one_ok:bool=integer_is(parsed.space,leaf,1);let inner_ok:bool=package_is(parsed.space,first,2);let outer_ok:bool=package_is(parsed.space,parsed.object,2);''','parsed.outcome==Outcome::Success && parsed.done && parsed.at=='+str(len(b))+' && parsed.space.object_count==5 && inner_ok && outer_ok && one_ok && inner.has_next',('parsed.space.object_count==5','parsed.space.object_count==4'),['bytes','names','namespace','values'],'Nested packages, uninitialized padding, sibling links and exact envelope end')
b=pkg(b'\x12',b'\x01\x01\x00')
finish('values-extra',arr(b)+f'''let r:Path=root();let start:Namespace=empty();let parsed:ValueState=parse_value(&input,{len(b)},0,1,r,start,false,5);''','parsed.outcome==Outcome::BadEncoding && parsed.offset==4',('parsed.offset==4','parsed.offset==3'),['bytes','names','namespace','values'],'Extra fixed-package elements yield error instead of pinned assertion panic')
b=pkg(b'\x11',integer(2)+bytes([0xaa,0xbb,0xcc,0xdd]))
finish('values-buffer',arr(b)+f'''let r:Path=root();let start:Namespace=empty();let parsed:ValueState=parse_value(&input,{len(b)},0,19,r,start,false,2);let object:Object=object_at(parsed.space,parsed.object);
let buffer_ok:bool=buffer_is(object.value);''','parsed.outcome==Outcome::Success && buffer_ok && parsed.at=='+str(len(b)),('parsed.at=='+str(len(b)),'parsed.at=='+str(len(b)-1)),['bytes','names','namespace','values'],'Buffer retains declared length and initializer span without copy_from_slice panic')
s=(C/'values-buffer.omg').read_text()+'''\nmachine buffer_is(value:Value)->bool {transition value {Value::Buffer {size,initializer} -> check(size,initializer) _ -> (false)} state check(size:u64,initializer:Span)->bool {size==2 && initializer.unit==19 && initializer.start==4 && initializer.end==8}}\n''';(C/'values-buffer.omg').write_text(s)
# Loader integration.
b=name('AAAA',integer(42))+b'\x06AAAABBBB'+name('AAAA',integer(7))
finish('load-alias',arr(b)+f'''let start:Namespace=empty();let parsed:LoadState=load(&input,{len(b)},77,start,4,2);let a:Path=single(0x41414141,true);let b:Path=single(0x42424242,true);let aa:Lookup=get(parsed.space,a);let bb:Lookup=get(parsed.space,b);let a_ok:bool=integer_is(parsed.space,aa.object,7);let b_ok:bool=integer_is(parsed.space,bb.object,42);''','parsed.outcome==Outcome::Success && parsed.done && parsed.at=='+str(len(b))+' && a_ok && b_ok && aa.object!=bb.object',('parsed.at=='+str(len(b)),'parsed.at=='+str(len(b)-1)),['bytes','names','namespace','values','loader'],'Load Name/Alias/rebinding from real AML byte stream')
b=pkg(b'\x14',b'MTHD'+bytes([0x39,0xfe,0xa4,0x01]))+name('AAAA',b'\x01')
finish('load-method',arr(b)+f'''let start:Namespace=empty();let parsed:LoadState=load(&input,{len(b)},88,start,3,2);let path:Path=single(0x4448544d,true);let found:Lookup=get(parsed.space,path);let retained:bool=method_is(parsed.space,found.object,88,7,10);''','parsed.outcome==Outcome::Success && retained && parsed.space.object_count==2',('parsed.space.object_count==2','parsed.space.object_count==1'),['bytes','names','namespace','values','loader'],'Unknown executable byte inside Method is retained, never executed at namespace load')
body=name('PKG0',pkg(b'\x12',b'\x01FWD0'))+name('FWD0',integer(23))
b=name('FWD0',integer(3))+scope('DEV0',body,b'\x5b\x82')
finish('load-forward-reference',arr(b)+f'''let start:Namespace=empty();let parsed:LoadState=load(&input,{len(b)},4,start,6,5);let d:Path=single(0x30564544,true);let p:PathResult=child(d,0x30474b50);let found:Lookup=get(parsed.space,p.value);let item:u64=package_first(parsed.space,found.object);let deferred:bool=reference_is(parsed.space,item,d);let resolved:ReferenceState=resolve_reference(parsed.space,item,3);let correct:bool=integer_is(parsed.space,resolved.object,23);''','parsed.outcome==Outcome::Success && deferred && resolved.outcome==Outcome::Success && correct && parsed.space.count==5',('parsed.space.count==5','parsed.space.count==4'),['bytes','names','namespace','values','loader','references'],'Package forward reference resolves after load from declaration Device scope, shadowing root object')
b=name('AAAA',b'\x01')+bytes([0x72,0,1,0])
finish('load-rollback',arr(b)+f'''let start:Namespace=empty();let parsed:LoadState=load(&input,{len(b)},1,start,3,2);''','parsed.outcome==Outcome::UnsupportedSyntax && parsed.offset==6 && parsed.space.count==1 && parsed.space.object_count==0',('parsed.offset==6','parsed.offset==7'),['bytes','names','namespace','values','loader'],'Unsupported executable Add rejects whole load and restores original namespace')
b=scope('\\\x00',name('AAAA',b'\x01'))
finish('load-root-scope',arr(b)+f'''let start:Namespace=empty();let parsed:LoadState=load(&input,{len(b)},1,start,4,2);let path:Path=single(0x41414141,true);let found:Lookup=get(parsed.space,path);let correct:bool=integer_is(parsed.space,found.object,1);''','parsed.outcome==Outcome::Success && correct && parsed.depth==0 && parsed.space.count==2',('parsed.space.count==2','parsed.space.count==3'),['bytes','names','namespace','values','loader'],'Scope of root NullName works and restores outer scope')

b=b'\x01'
for _ in range(9):b=pkg(b'\x12',b'\x01'+b)
finish('values-depth',arr(b)+f"let r:Path=root();let start:Namespace=empty();let parsed:ValueState=parse_value(&input,{len(b)},0,1,r,start,false,12);",'parsed.outcome==Outcome::Depth && parsed.depth==8',('parsed.depth==8','parsed.depth==7'),['bytes','names','namespace','values'],'Ninth nested package exceeds explicit eight-frame resource profile')
b=pkg(b'\x12',b'\x40')
finish('values-capacity-work',arr(b)+f"let r:Path=root();let start:Namespace=empty();let full:ValueState=parse_value(&input,{len(b)},0,1,r,start,false,2);let budget:ValueState=parse_value(&input,{len(b)},0,1,r,start,false,0);",'full.outcome==Outcome::Capacity && full.space.object_count==0 && budget.outcome==Outcome::WorkLimit',('full.space.object_count==0','full.space.object_count==1'),['bytes','names','namespace','values'],'Declared package count cannot exceed object arena; zero work returns WorkLimit')
finish('reference-cycle', """let start:Namespace=empty();let r:Path=root();let a:Path=single(0x41414141,true);let b:Path=single(0x42424242,true);
let to_b:Value=Value::NameReference {name:b,scope:r};let to_a:Value=Value::NameReference {name:a,scope:r};
let first:NamespaceResult=insert(start,a,to_b);let missing:ReferenceState=resolve_reference(first.space,first.object,3);
let second:NamespaceResult=insert(first.space,b,to_a);let cycle:ReferenceState=resolve_reference(second.space,first.object,4);
let exhausted:ReferenceState=resolve_reference(second.space,first.object,1);""",'missing.outcome==Outcome::MissingObject && cycle.outcome==Outcome::ReferenceCycle && cycle.object==0 && exhausted.outcome==Outcome::WorkLimit',('cycle.object==0','cycle.object==1'),['names','namespace','references'],'Missing forward references, exact identity-cycle detection, and caller work budget remain distinct')
b=b'\x15EXT0'+bytes([8,2])+b'\x5b\x01MUT0'+bytes([3])+b'\x5b\x02EVT0'+b'\x5b\x80REG0'+bytes([0])+integer(0x20)+integer(8)
finish('load-inert-objects',arr(b)+f"let start:Namespace=empty();let parsed:LoadState=load(&input,{len(b)},1,start,5,2);let external_path:Path=single(0x30545845,true);let absent:Lookup=get(parsed.space,external_path);",'parsed.outcome==Outcome::Success && parsed.external_count==1 && parsed.externals[0].argument_count==2 && absent.outcome==Outcome::MissingObject && parsed.space.object_count==3',('parsed.space.object_count==3','parsed.space.object_count==4'),['bytes','names','namespace','values','loader'],'External retained only as metadata; Mutex/Event/literal OpRegion have inert objects and no handlers')
b=scope('CPU0',bytes([7,0x34,0x12,0,0,6]),b'\x5b\x83')+scope('PWR0',bytes([4,0x78,0x56]),b'\x5b\x84')+scope('THM0',b'',b'\x5b\x85')
finish('load-scoped-records',arr(b)+f"let start:Namespace=empty();let parsed:LoadState=load(&input,{len(b)},1,start,7,2);let cpu_path:Path=single(0x30555043,true);let found:Lookup=get(parsed.space,cpu_path);let cpu:Object=object_at(parsed.space,found.object);let correct:bool=processor_is(cpu.value);",'parsed.outcome==Outcome::Success && correct && parsed.space.count==4 && parsed.space.object_count==3',('parsed.space.count==4','parsed.space.count==3'),['bytes','names','namespace','values','loader'],'Processor/Pblk fields, PowerResource and ThermalZone scopes create both level and object slots')
source=(C/'load-scoped-records.omg').read_text()+"\nmachine processor_is(value:Value)->bool {transition value {Value::Processor {id,address,length} -> check(id,address,length) _ -> (false)} state check(id:u8,address:u32,length:u8)->bool {id==7 && address==0x1234 && length==6}}\n";(C/'load-scoped-records.omg').write_text(source)
b=b''
for _ in range(9):b=scope('AAAA',b)
finish('load-depth',arr(b)+f"let start:Namespace=empty();let parsed:LoadState=load(&input,{len(b)},1,start,11,2);",'parsed.outcome==Outcome::Depth && parsed.space.count==1 && parsed.depth==8',('parsed.depth==8','parsed.depth==7'),['bytes','names','namespace','values','loader'],'Scope nesting limit is a recoverable resource result and namespace transaction rolls back')
b=name('AAAA',b'\x01')
finish('load-capacity-work',arr(b)+f"let mut start:Namespace=empty();start.object_count=64;let full:LoadState=load(&input,{len(b)},1,start,2,2);let exhausted:LoadState=load(&input,{len(b)},1,start,0,2);",'full.outcome==Outcome::Capacity && full.space.object_count==64 && exhausted.outcome==Outcome::WorkLimit && exhausted.space.count==1',('full.space.object_count==64','full.space.object_count==63'),['bytes','names','namespace','values','loader'],'Object arena capacity and whole-loader work budget are distinct and preserve supplied namespace')


b=bytes([0xff,0x0e])+bytes.fromhex('1032547698badcfe')+bytes([0x5b,0x30])
finish('integer-widths',arr(b)+"let ones:Read=integer(&input,12,0);let qword:Read=integer(&input,12,1);let revision:Read=integer(&input,12,10);let short:Read=integer(&input,9,1);",'ones.value==(0xffffffffffffffff as u64) && qword.value==(0xfedcba9876543210 as u64) && qword.next==10 && revision.value==2 && short.outcome==Outcome::Truncated',('qword.next==10','qword.next==9'),['bytes','names','namespace','values'],'Ones and high-bit QWord retain full64 bits; Revision is2; short QWord is truncated')
b=bytes([13,65,0,13,66,13,128,0,13,0])
finish('string-validation',arr(b)+"let r:Path=root();let start:Namespace=empty();let good:ValueState=parse_value(&input,3,0,3,r,start,false,2);let short:ValueState=parse_value(&input,5,3,3,r,start,false,2);let bad:ValueState=parse_value(&input,8,5,3,r,start,false,2);let empty_string:ValueState=parse_value(&input,10,8,3,r,start,false,2);let object:Object=object_at(good.space,good.object);let correct:bool=string_is(object.value);",'good.outcome==Outcome::Success && correct && short.outcome==Outcome::Truncated && bad.outcome==Outcome::BadEncoding && empty_string.outcome==Outcome::Success && good.at==3',('good.at==3','good.at==2'),['bytes','names','namespace','values'],'ASCII span without terminator, unterminated input and non-ASCII bytes have distinct outcomes')
source=(C/'string-validation.omg').read_text()+"\nmachine string_is(value:Value)->bool {transition value {Value::String {bytes} -> check(bytes) _ -> (false)} state check(bytes:Span)->bool {bytes.unit==3 && bytes.start==1 && bytes.end==2}}\n";(C/'string-validation.omg').write_text(source)
b=pkg(b'\x13',integer(3)+b'\x01')
finish('variable-package',arr(b)+f"let r:Path=root();let start:Namespace=empty();let parsed:ValueState=parse_value(&input,{len(b)},0,1,r,start,false,7);let right_count:bool=package_is(parsed.space,parsed.object,3);let first:u64=package_first(parsed.space,parsed.object);let one:bool=integer_is(parsed.space,first,1);",'parsed.outcome==Outcome::Success && right_count && one && parsed.space.object_count==4 && parsed.at=='+str(len(b)),('parsed.space.object_count==4','parsed.space.object_count==3'),['bytes','names','namespace','values'],'Literal-count VarPackage pads each missing element as a distinct uninitialized object')
b=bytes([0x2f,16])+b''.join(('A%03d'%i).encode() for i in range(16))+b'^'*17+b'AAAA'+bytes([0x2f,17])
finish('name-capacity',arr(b)+f"let at_limit:PathResult=namestring(&input,{len(b)},0);let parents:PathResult=namestring(&input,{len(b)},66);let segments:PathResult=namestring(&input,{len(b)},{len(b)-2});",'at_limit.outcome==Outcome::Success && at_limit.value.count==16 && at_limit.next==66 && parents.outcome==Outcome::Capacity && segments.outcome==Outcome::Capacity',('at_limit.next==66','at_limit.next==65'),['bytes','names'],'Sixteen segments fit; seventeenth parent prefix or MultiName count returns Capacity')

(H/'cases.json').write_text(json.dumps(cases,indent=2,sort_keys=True)+'\n')
print(len(cases),'cases written')
