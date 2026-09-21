#!/usr/bin/env python3
"""Actual AML normal-Field reads using explicit initialized synthetic responses."""
MAX = (1 << 64) - 1


def integer(value):
    if value in (0, 1):
        return bytes([value])
    for width, opcode in ((1, 0x0A), (2, 0x0B), (4, 0x0C), (8, 0x0E)):
        if value < 1 << (width * 8):
            return bytes([opcode]) + value.to_bytes(width, 'little')
    raise ValueError(value)


def encoded_length(value):
    if value < 64:
        return bytes([value])
    for following in range(1, 4):
        if value < 1 << (4 + 8 * following):
            return bytes([(following << 6) | (value & 15)]) + (value >> 4).to_bytes(following, 'little')
    raise ValueError(value)


def package(opcode, body):
    for count in range(1, 5):
        length = encoded_length(len(body) + count)
        if len(length) == count:
            return bytes(opcode) + length + body
    raise ValueError(len(body))


def method(label, body, flags=0):
    return package([0x14], label.encode('ascii') + bytes([flags]) + body)


def table(body, *, bits=8, offset=0, width=1, locked=False, base=0, region_length=512,
          space=0, prefix=b'', extra=b'', flags=0):
    region = b'\x5b\x80REG0' + bytes([space]) + integer(base) + integer(region_length)
    access = {1: 1, 2: 2, 4: 3, 8: 4}[width] | (16 if locked else 0)
    members = (b'\0' + encoded_length(offset) if offset else b'') + b'FLD_' + encoded_length(bits)
    field = package([0x5B, 0x81], b'REG0' + bytes([access]) + members)
    return prefix + region + field + extra + method('MAIN', body, flags)


def array(name, data, size=1024):
    assert len(data) <= size
    return f'let mut {name}:[u8;{size}];' + ''.join(
        f'{name}[{i}]={value};' for i, value in enumerate(data) if value) + '\n'


IMPORTS = '''use aml::model::Value;
use aml::model::FieldBinding;
use aml::model::Path;
use aml::model::ObjectStore;
use aml::model::BufferStorage;
use aml::model::ReferenceKind;
use aml::model::Outcome;
use aml::field_model::DeclarationKind;
use aml::field_model::Declaration;
use aml::field_model::Field;
use aml::field_model::Access;
use aml::field_model::Connection;
use aml::model::Span;
use pipeline::program::Program;
use pipeline::program::Prepared;
use pipeline::program::prepare_program;
use pipeline::program::run_program;
use pipeline::field_reads::Session;
use pipeline::field_reads::State;
use pipeline::field_reads::Progress;
use pipeline::field_reads::NativeRead;
use pipeline::field_reads::Completion;
use pipeline::field_reads::begin;
use pipeline::field_reads::begin_limited;
use pipeline::field_reads::advance;
use pipeline::field_reads::progress;
use pipeline::field_reads::complete;
use pipeline::field_reads::same_request;
use execution::engine::ExecutionResult;
use execution::execution_model::ExecutionOutcome;
use execution::execution_model::Operand;
use execution::execution_model::DeferredRead;
use integer_helpers::integers::IntegerSize;
'''

HELPERS = '''data Trace {count:u64;requests:[NativeRead;512];values:[u64;512];}
machine fr_word(memory:&[u8;512],start:u64,width:u64,index:u64,value:u64)
terminates by(index,width)->Nat::BoundedDistance;
->u64 {let next:u64=fr_byte(memory,start,width,index,value);transition index<width {true -> fr_word(memory,start,width,index+1,next) _ -> (value)}}
machine fr_byte(memory:&[u8;512],start:u64,width:u64,index:u64,value:u64)->u64 {
 transition index<width && index<8 && start<512 && index<512-start {true -> read(memory,start,index,value) _ -> (value)}
 state read(memory:&[u8;512],start:u64,index:u64,value:u64)->u64 {
  let at:u64=start+index;transition at<512 && index<8 {true -> selected(memory,at,index,value) _ -> (value)}
 }
 state selected(memory:&[u8;512],at:u64,index:u64,value:u64)->u64 {let byte:u64=memory[at] as u64;value | (byte << (index*8))}
}
machine fr_run(session:&mut Session,memory:&[u8;512],trace:&mut Trace,limit:u64,fail_at:u64,mode:u64)->ExecutionOutcome {
 let result:ExecutionOutcome=fr_drive(session,memory,trace,0,limit,fail_at,mode);result
}
machine fr_drive(session:&mut Session,memory:&[u8;512],trace:&mut Trace,index:u64,limit:u64,fail_at:u64,mode:u64)
terminates by(index,limit)->Nat::BoundedDistance;
->ExecutionOutcome {
 let result:ExecutionOutcome=fr_turn(session,memory,trace,index,limit,fail_at,mode);
 transition {session.state==State::Finished || session.state==State::Failed -> (result)
  index<limit -> fr_drive(session,memory,trace,index+1,limit,fail_at,mode) _ -> (result)}
}
machine fr_turn(session:&mut Session,memory:&[u8;512],trace:&mut Trace,index:u64,limit:u64,fail_at:u64,mode:u64)->ExecutionOutcome {
 transition index<limit {true -> poll(session,memory,trace,fail_at,mode) _ -> (ExecutionOutcome::WorkLimit)}
 state poll(session:&mut Session,memory:&[u8;512],trace:&mut Trace,fail_at:u64,mode:u64)->ExecutionOutcome {
  let result:Progress=advance(session,1);
  transition result {
   Progress::Read {request} -> request(session,memory,trace,request,fail_at,mode)
   Progress::Failure {outcome} -> (outcome)
   _ -> (ExecutionOutcome::Success)
  }
 }
 state request(session:&mut Session,memory:&[u8;512],trace:&mut Trace,request:NativeRead,fail_at:u64,mode:u64)->ExecutionOutcome {
  let count:u64=trace.count;
  transition count<512 && request.base<=512 && request.offset<=512-request.base && request.width<=512-request.base-request.offset {
   true -> observe(session,memory,trace,request,fail_at,mode,count) _ -> (ExecutionOutcome::Capacity)
  }
 }
 state observe(session:&mut Session,memory:&[u8;512],trace:&mut Trace,request:NativeRead,fail_at:u64,mode:u64,index:u64)->ExecutionOutcome {
  let raw:u64=fr_word(memory,request.base+request.offset,request.width,0,0);
  let word:u64=fr_response(raw,index,mode);
  transition index<512 {true -> retain(session,trace,request,fail_at,index,word) _ -> (ExecutionOutcome::Capacity)}
 }
 state retain(session:&mut Session,trace:&mut Trace,request:NativeRead,fail_at:u64,index:u64,word:u64)->ExecutionOutcome {
  trace.requests[index]=request;trace.values[index]=word;trace.count=index+1;
  transition index==fail_at {true -> failure(session,request) _ -> completed(session,request,word)}
 }
 state failure(session:&mut Session,request:NativeRead)->ExecutionOutcome {let result:ExecutionOutcome=complete(session,Completion::Failure {request:request,outcome:ExecutionOutcome::UnresolvedService});result}
 state completed(session:&mut Session,request:NativeRead,value:u64)->ExecutionOutcome {let result:ExecutionOutcome=complete(session,Completion::Word {request:request,value:value});result}
}
machine fr_response(raw:u64,index:u64,mode:u64)->u64 {
 transition {mode==1 && index<2 -> (1) mode==1 -> (0) mode==2 -> (1) _ -> (raw)}
}
machine fr_trace(trace:&Trace,memory:&[u8;512],count:u64,chunks:u64,start:u64,width:u64,base:u64,field:u64,mode:u64,index:u64,limit:u64,prior:bool)
terminates by(index,limit)->Nat::BoundedDistance;
->bool {let good:bool=fr_event(trace,memory,count,chunks,start,width,base,field,mode,index);transition index<limit {true -> fr_trace(trace,memory,count,chunks,start,width,base,field,mode,index+1,limit,prior && good) _ -> (prior && trace.count==count)}}
machine fr_event(trace:&Trace,memory:&[u8;512],count:u64,chunks:u64,start:u64,width:u64,base:u64,field:u64,mode:u64,index:u64)->bool {
 transition index<512 {true -> selected(trace,memory,count,chunks,start,width,base,field,mode,index) _ -> (true)}
 state selected(trace:&Trace,memory:&[u8;512],count:u64,chunks:u64,start:u64,width:u64,base:u64,field:u64,mode:u64,index:u64)->bool {
  transition index<count && chunks>0 {true -> ordinal(trace,memory,chunks,start,width,base,field,mode,index) _ -> tail(trace,index)}
 }
 state ordinal(trace:&Trace,memory:&[u8;512],chunks:u64,start:u64,width:u64,base:u64,field:u64,mode:u64,index:u64)->bool {
  let ordinal:u64=index%chunks;
  transition ordinal<257 && start<=512 && width<=8 && base<=512 && field>0 && index<512 {
   true -> live(trace,memory,start,width,base,field,mode,index,ordinal) _ -> (false)
  }
 }
 state live(trace:&Trace,memory:&[u8;512],start:u64,width:u64,base:u64,field:u64,mode:u64,index:u64,ordinal:u64)->bool {
  let offset:u64=start+ordinal*width;
  let expected:NativeRead=NativeRead {session:77,serial:index+1,unit:7,field:field,region:field-1,base:base,offset:offset,width:width};
  let same:bool=same_request(trace.requests[index],expected);
  let raw:u64=fr_word(memory,base+offset,width,0,0);let value:u64=fr_response(raw,index,mode);
  transition index<512 {true -> compared(trace,index,same,value) _ -> (false)}
 }
 state compared(trace:&Trace,index:u64,same:bool,value:u64)->bool {same && trace.values[index]==value}
 state tail(trace:&Trace,index:u64)->bool {let same:bool=same_request(trace.requests[index],NativeRead {});same && trace.values[index]==0}
}
machine fr_integer(value:Value,expected:u64)->bool {transition value {Value::Integer {number} -> (number==expected) _ -> (false)}}
machine fr_span(a:Span,b:Span)->bool {a.unit==b.unit && a.start==b.start && a.end==b.end}
machine fr_path(a:Path,b:Path)->bool {let same:bool=fr_segments(&a.segments,&b.segments,0,16,true);a.absolute==b.absolute && a.parents==b.parents && a.count==b.count && same}
machine fr_segments(a:&[u32;16],b:&[u32;16],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=fr_segment(a,b,index);transition index<count {true -> fr_segments(a,b,index+1,count,prior && same) _ -> (prior)}}
machine fr_segment(a:&[u32;16],b:&[u32;16],index:u64)->bool {transition index<16 {true -> (a[index]==b[index]) _ -> (true)}}
machine fr_access(a:Access,b:Access)->bool {a.flags==b.flags && a.kind==b.kind && a.locked==b.locked && a.update==b.update && a.attribute==b.attribute && a.attribute_mode==b.attribute_mode && a.extended==b.extended && a.access_length==b.access_length}
machine fr_connection(a:Connection,b:Connection)->bool {
 transition a {Connection::None -> none(b) Connection::Name {connection_name} -> name(connection_name,b) Connection::Buffer {buffer_encoding,size_known,declared_size,initializer} -> buffer(buffer_encoding,size_known,declared_size,initializer,b)}
 state none(b:Connection)->bool {transition b {Connection::None -> (true) _ -> (false)}}
 state name(a:Path,b:Connection)->bool {transition b {Connection::Name {connection_name} -> path(a,connection_name) _ -> (false)}}
 state path(a:Path,b:Path)->bool {let same:bool=fr_path(a,b);same}
 state buffer(a:Span,known:bool,size:u64,a_initializer:Span,b:Connection)->bool {transition b {Connection::Buffer {buffer_encoding,size_known,declared_size,initializer} -> spans(a,known,size,a_initializer,buffer_encoding,size_known,declared_size,initializer) _ -> (false)}}
 state spans(a:Span,known:bool,size:u64,initializer:Span,b:Span,other_known:bool,other_size:u64,other_initializer:Span)->bool {let encoding:bool=fr_span(a,b);let source:bool=fr_span(initializer,other_initializer);encoding && source && known==other_known && size==other_size}
}
machine fr_declaration(a:Declaration,b:Declaration)->bool {
 let scope:bool=fr_path(a.scope,b.scope);let primary:bool=fr_path(a.primary_name,b.primary_name);let secondary:bool=fr_path(a.secondary_name,b.secondary_name);let source:bool=fr_span(a.source,b.source);let list:bool=fr_span(a.field_list,b.field_list);
 scope && primary && secondary && source && list && a.kind==b.kind && a.bank_value==b.bank_value && a.flags==b.flags
}
machine fr_metadata(a:Field,b:Field)->bool {
 let name:bool=fr_path(a.name,b.name);let access:bool=fr_access(a.access,b.access);let connection:bool=fr_connection(a.connection,b.connection);let source:bool=fr_span(a.source,b.source);
 name && access && connection && source && a.bit_offset==b.bit_offset && a.bit_length==b.bit_length
}
machine fr_binding(a:FieldBinding,b:FieldBinding)->bool {
 transition a {FieldBinding::Region {region_object} -> region(region_object,b) FieldBinding::Bank {region_object,selector_object} -> bank(region_object,selector_object,b) FieldBinding::Index {index_object,data_object} -> index(index_object,data_object,b)}
 state region(id:u64,b:FieldBinding)->bool {transition b {FieldBinding::Region {region_object} -> (id==region_object) _ -> (false)}}
 state bank(region:u64,selector:u64,b:FieldBinding)->bool {transition b {FieldBinding::Bank {region_object,selector_object} -> (region==region_object && selector==selector_object) _ -> (false)}}
 state index(index:u64,data:u64,b:FieldBinding)->bool {transition b {FieldBinding::Index {index_object,data_object} -> (index==index_object && data==data_object) _ -> (false)}}
}
machine fr_field(value:Value,expected:Value)->bool {
 transition value {Value::FieldUnit {binding,declaration,field} -> other(binding,declaration,field,expected) _ -> (false)}
 state other(a:FieldBinding,b:Declaration,c:Field,expected:Value)->bool {transition expected {Value::FieldUnit {binding,declaration,field} -> compare(a,b,c,binding,declaration,field) _ -> (false)}}
 state compare(a:FieldBinding,b:Declaration,c:Field,d:FieldBinding,e:Declaration,f:Field)->bool {let binding:bool=fr_binding(a,d);let declaration:bool=fr_declaration(b,e);let field:bool=fr_metadata(c,f);binding && declaration && field}
}
machine fr_malformed(value:Value,mode:u64)->Value {
 transition value {Value::FieldUnit {binding,declaration,field} -> changed(binding,declaration,field,mode) _ -> (value)}
 state changed(binding:FieldBinding,declaration:Declaration,field:Field,mode:u64)->Value {
  transition {mode==1 -> (Value::FieldUnit {binding:FieldBinding::Region {region_object:18446744073709551615},declaration:declaration,field:field}) mode==2 -> source(binding,declaration,field) mode==3 -> bank(declaration,field) mode==4 -> index(declaration,field) mode==6 -> declaration_source(binding,declaration,field) mode==7 -> (Value::FieldUnit {binding:FieldBinding::Index {index_object:0,data_object:1},declaration:declaration,field:field}) _ -> (Value::FieldUnit {binding:FieldBinding::Bank {region_object:0,selector_object:1},declaration:declaration,field:field})}
 }
 state declaration_source(binding:FieldBinding,declaration:Declaration,field:Field)->Value {let mut d:Declaration=declaration;d.source.unit=8;Value::FieldUnit {binding:binding,declaration:d,field:field}}
 state source(binding:FieldBinding,declaration:Declaration,field:Field)->Value {let mut f:Field=field;f.source.unit=8;Value::FieldUnit {binding:binding,declaration:declaration,field:f}}
 state bank(declaration:Declaration,field:Field)->Value {let mut d:Declaration=declaration;d.kind=DeclarationKind::Bank;Value::FieldUnit {binding:FieldBinding::Bank {region_object:0,selector_object:1},declaration:d,field:field}}
 state index(declaration:Declaration,field:Field)->Value {let mut d:Declaration=declaration;d.kind=DeclarationKind::Index;Value::FieldUnit {binding:FieldBinding::Index {index_object:0,data_object:1},declaration:d,field:field}}
}
machine fr_no_deferred(value:DeferredRead)->bool {transition value {DeferredRead::None -> (true) _ -> (false)}}
machine fr_result(session:&Session,expected:u64)->bool {transition session.result.operand {Operand::Integer {number} -> (number==expected && session.result.has_value) _ -> (false)}}
machine fr_bytes(a:&[u8;256],b:&[u8;256],index:u64,count:u64,prior:bool)
terminates by(index,count)->Nat::BoundedDistance;
->bool {let same:bool=fr_byte_equal(a,b,index);transition index<count {true -> fr_bytes(a,b,index+1,count,prior && same) _ -> (prior)}}
machine fr_byte_equal(a:&[u8;256],b:&[u8;256],index:u64)->bool {transition index<256 {true -> (a[index]==b[index]) _ -> (true)}}
machine fr_buffer(session:&Session,length:u64,expected:&[u8;256])->bool {
 transition session.result.operand {Operand::Object {object_id} -> identity(session,length,expected,object_id) _ -> (false)}
 state identity(session:&Session,length:u64,expected:&[u8;256],object:u64)->bool {
  transition object<session.program.store.space.object_count && object<64 {true -> value(session,length,expected,object) _ -> (false)}
 }
 state value(session:&Session,length:u64,expected:&[u8;256],object:u64)->bool {
  transition session.program.store.space.objects[object].value {Value::Buffer {buffer_storage} -> owner(session,length,expected,object,buffer_storage) _ -> (false)}
 }
 state owner(session:&Session,length:u64,expected:&[u8;256],object:u64,storage:BufferStorage)->bool {
  transition storage {BufferStorage::Owned {buffer_owner} -> bytes(session,length,expected,object,buffer_owner) _ -> (false)}
 }
 state bytes(session:&Session,length:u64,expected:&[u8;256],object:u64,owner:u64)->bool {
  let same:bool=fr_bytes(&session.program.store.bytes.blocks[object].bytes,expected,0,256,true);
  transition object<64 {true -> compared(session,length,object,owner,same) _ -> (false)}
 }
 state compared(session:&Session,length:u64,object:u64,owner:u64,same:bool)->bool {
  same && object==owner && session.program.store.bytes.blocks[object].initialized && session.program.store.bytes.blocks[object].length==length && session.result.has_value
 }
}
machine fr_progress_error(result:Progress,expected:ExecutionOutcome)->bool {transition result {Progress::Failure {outcome} -> (outcome==expected) _ -> (false)}}
machine fr_progress_read(result:Progress,expected:NativeRead)->bool {
 transition result {Progress::Read {request} -> matched(request,expected) _ -> (false)}
 state matched(value:NativeRead,expected:NativeRead)->bool {let same:bool=same_request(value,expected);same}
}
machine fr_reference_result(session:&Session,expected:ReferenceKind)->bool {
 transition session.runtime.result {Operand::Object {object_id} -> object(session,expected,object_id) _ -> (false)}
 state object(session:&Session,expected:ReferenceKind,index:u64)->bool {
  transition index<64 && index<session.program.store.space.object_count {true -> value(session,expected,index) _ -> (false)}
 }
 state value(session:&Session,expected:ReferenceKind,index:u64)->bool {
  transition session.program.store.space.objects[index].value {Value::Reference {kind,object_id} -> (kind==expected && object_id==1) _ -> (false)}
 }
}
'''


def cases():
    rows = []
    memory = bytes((i * 37 + 19) & 255 for i in range(512))

    def add(name, body, check, old, new, description):
        assert check.count(old) == 1, (name, old)
        rows.append(dict(name=name, body=body, check=check, mutation=[old, new], description=description))

    def start(data, bits=64, setup='', arguments='', flags=0, fuel=1024, quotas=None, field=1):
        body = array('input', data)
        body += f'let prepared:Prepared=prepare_program(input,{len(data)},7,128,1024);let mut program:Program=prepared.program;'
        body += setup
        body += 'let mut path:Path=Path {absolute:true,count:1};path.segments[0]=1313423693;let mut arguments:[Value;7];'  # MAIN
        body += arguments
        body += f'let original_field:Value=program.store.space.objects[{field}].value;'
        api = 'begin' if quotas is None else 'begin_limited'
        limits = '' if quotas is None else ',' + ','.join(map(str, quotas))
        body += f'let mut session:Session={api}(program,path,&arguments,{flags},IntegerSize::{"FourBytes" if bits==32 else "EightBytes"},{fuel},77{limits});'
        return body

    def run_case(name, data, expected, *, bits=64, field_bits=8, offset=0, width=1,
                 base=0, field=1, passes=1, mode=0, fail_at=MAX, error='Success',
                 setup='', after='', extra_check='', arguments='', flags=0):
        native_bits = width * 8
        start_offset = (offset // native_bits) * width
        chunks = ((offset + field_bits + native_bits - 1) // native_bits) - offset // native_bits
        count = chunks * passes if fail_at == MAX else fail_at + 1
        body = start(data, bits, setup, arguments, flags, field=field) + array('memory', memory, 512)
        body += f'let mut trace:Trace;let outcome:ExecutionOutcome=fr_run(&mut session,&memory,&mut trace,4096,{fail_at},{mode});'
        body += f'let log:bool=fr_trace(&trace,&memory,{count},{chunks},{start_offset},{width},{base},{field},{mode},0,512,true);'
        body += f'let intact:bool=fr_field(session.program.store.space.objects[{field}].value,original_field);'
        if isinstance(expected, bytes):
            body += array('expected', expected, 256)
            body += f'let result:bool=fr_buffer(&session,{len(expected)},&expected);'
        else:
            body += f'let result:bool=fr_result(&session,{expected});'
        body += after
        if error == 'Success':
            check = 'prepared.outcome==Outcome::Success && session.state==State::Finished && outcome==ExecutionOutcome::Success && result && log && intact'
        else:
            check = f'prepared.outcome==Outcome::Success && session.state==State::Failed && outcome==ExecutionOutcome::{error} && !session.runtime.has_result && log && intact'
        if extra_check:
            check += ' && ' + extra_check
        add(name, body, check, 'log', '!log', 'Actual AML/session execution with exact synthetic request order, words and zero trace tail')

    for revision in (32, 64):
        for width in (1, 2, 4, 8):
            offset = width * 8 - 3
            count = min(revision, 29)
            raw = int.from_bytes(memory, 'little') >> offset
            expected = raw & ((1 << count) - 1)
            run_case(f'integer_{revision}_width_{width}', table(b'\xa4FLD_', bits=count, offset=offset, width=width),
                     expected, bits=revision, field_bits=count, offset=offset, width=width)
        expected = (int.from_bytes(memory, 'little') >> 5) & ((1 << revision) - 1)
        run_case(f'integer_full_{revision}', table(b'\xa4FLD_', bits=revision, offset=5, width=4),
                 expected, bits=revision, field_bits=revision, offset=5, width=4)
        for count in (revision + 1, 2048):
            offset = 3
            expected = ((int.from_bytes(memory, 'little') >> offset) & ((1 << count) - 1)).to_bytes((count + 7) // 8, 'little')
            run_case(f'buffer_{revision}_{count}', table(b'\xa4FLD_', bits=count, offset=offset),
                     expected, bits=revision, field_bits=count, offset=offset,
                     extra_check='session.program.store.space.object_count==4')

    run_case('arithmetic_operand', table(b'\xa4\x72FLD_\x01\x00'), memory[0] + 1)
    run_case('to_integer_operand', table(b'\xa4\x99FLD_\x00'), memory[0])
    run_case('two_operands', table(b'\xa4\x72FLD_FLD_\x00'), memory[0] * 2, passes=2)
    run_case('nested_argument', table(b'\xa4GIVEFLD_', extra=method('GIVE', b'\xa4\x68', 1)), memory[0])
    run_case('nested_callee_read', table(b'\xa4GIVE', extra=method('GIVE', b'\xa4FLD_')), memory[0])
    run_case('explicit_argument_read', table(b'\xa4\x68', flags=1), memory[0], flags=1,
             arguments='arguments[0]=Value::Reference {kind:ReferenceKind::RefOf,object_id:1};')
    run_case('statement_read', table(b'FLD_\xa4\x01'), 1)
    run_case('branch', table(package([0xA0], b'FLD_\xa4\x0a\x2a') + b'\xa4\x0a\x09'), 42)
    keep = b'\x08KEEP\x00'
    run_case('loop', table(package([0xA2], b'FLD_\x75KEEP') + b'\xa4KEEP', extra=keep),
             2, mode=1, passes=3,
             after='let kept:bool=fr_integer(session.program.store.space.objects[2].value,2);', extra_check='kept')
    run_case('named_store_source', table(b'\x70FLD_KEEP\xa4KEEP', extra=keep), memory[0],
             after=f'let kept:bool=fr_integer(session.program.store.space.objects[2].value,{memory[0]});', extra_check='kept')
    snapshot_table = table(b'\xa4FLD_')
    run_case('source_snapshot', snapshot_table, memory[0],
             setup=f'input[0]=0;input[{len(snapshot_table)-1}]=255;',
             after=f'let retained:bool=session.program.source[0]==0x5b && session.program.source[{len(snapshot_table)-1}]==95;', extra_check='retained')
    run_case('nonzero_base', table(b'\xa4FLD_', bits=32, offset=5, width=4, base=16),
             (int.from_bytes(memory[16:], 'little') >> 5) & ((1 << 32) - 1),
             field_bits=32, offset=5, width=4, base=16)

    for fail_at in (0, 2):
        code = b'\x70\x0a\x2aKEEP\xa4FLD_'
        run_case(f'failed_read_{fail_at}', table(code, bits=24, extra=keep), 0,
                 field_bits=24, fail_at=fail_at, error='UnresolvedService',
                 after='let kept:bool=fr_integer(session.program.store.space.objects[2].value,42);',
                 extra_check=f'kept && session.transfer.received=={fail_at} && session.accepted=={fail_at} && session.program.store.space.object_count==4 && session.transfer.words[0]=={memory[0] if fail_at else 0} && session.transfer.words[1]=={memory[1] if fail_at else 0}')
    run_case('failed_wide_last', table(code, bits=65, extra=keep), 0,
             field_bits=65, fail_at=8, error='UnresolvedService',
             after='let kept:bool=fr_integer(session.program.store.space.objects[2].value,42);',
             extra_check=f'kept && session.transfer.received==8 && session.accepted==8 && session.program.store.space.object_count==4 && session.transfer.words[0]=={memory[0]} && session.transfer.words[7]=={memory[7]} && session.transfer.words[8]==0')
    run_case('writes_still_unresolved', table(b'\x70\x01FLD_\xa4\x01'), 0, passes=0, error='UnresolvedRegion')
    run_case('increment_still_unresolved', table(b'\x75FLD_\xa4\x01'), 0, passes=0, error='UnresolvedRegion')

    for name, options, setup, error in [
        ('unsupported_space', {'space': 1}, '', 'UnresolvedRegion'),
        ('lock', {'locked': True}, '', 'UnresolvedSynchronization'),
        ('alignment', {'width': 2, 'base': 1}, '', 'UnresolvedRegion'),
        ('address_overflow', {'base': MAX}, '', 'Overflow'),
        ('outside_region', {'bits': 16, 'region_length': 1}, '', 'UnresolvedRegion'),
        ('zero_width', {'bits': 0}, '', 'UnresolvedRegion'),
        ('wide_capacity', {'bits': 65}, 'program.store.space.object_count=64;', 'Capacity'),
        ('bad_identity', {}, 'let changed:Value=fr_malformed(program.store.space.objects[1].value,1);program.store.space.objects[1].value=changed;', 'InvalidState'),
        ('source_unit', {}, 'let changed:Value=fr_malformed(program.store.space.objects[1].value,2);program.store.space.objects[1].value=changed;', 'InvalidState'),
        ('bank_kind', {}, 'let changed:Value=fr_malformed(program.store.space.objects[1].value,3);program.store.space.objects[1].value=changed;', 'UnresolvedRegion'),
        ('index_kind', {}, 'let changed:Value=fr_malformed(program.store.space.objects[1].value,4);program.store.space.objects[1].value=changed;', 'UnresolvedRegion'),
        ('declaration_unit', {}, 'let changed:Value=fr_malformed(program.store.space.objects[1].value,6);program.store.space.objects[1].value=changed;', 'InvalidState'),
        ('normal_index_binding', {}, 'let changed:Value=fr_malformed(program.store.space.objects[1].value,7);program.store.space.objects[1].value=changed;', 'InvalidState'),
        ('normal_bank_binding', {}, 'let changed:Value=fr_malformed(program.store.space.objects[1].value,5);program.store.space.objects[1].value=changed;', 'InvalidState'),
    ]:
        data = table(b'\xa4FLD_', **options)
        body = start(data, setup=setup) + 'let status:Progress=advance(&mut session,64);'
        check = f'session.state==State::Failed && session.outcome==ExecutionOutcome::{error} && session.issued==0 && session.accepted==0'
        add('admission_' + name, body, check, f'session.outcome==ExecutionOutcome::{error}',
            'session.outcome==ExecutionOutcome::Success', 'Complete field/region/profile/capacity admission rejects before any request')

    # One outstanding multi-chunk read: rejecting any coordinate must preserve
    # it so the original completion remains acceptable afterward.
    for coordinate in ('session', 'serial', 'unit', 'field', 'region', 'space', 'base', 'offset', 'width'):
        body = start(table(b'\xa4FLD_', bits=16))
        body += 'let status:Progress=advance(&mut session,64);let expected:NativeRead=session.transfer.request;'
        body += 'let steps:u64=session.runtime.steps;let pc:u64=session.runtime.frames[0].pc;let operands:u64=session.runtime.frames[0].operations[0].count;'
        body += f'let mut wrong:NativeRead=expected;wrong.{coordinate}=wrong.{coordinate}^1;'
        body += 'let rejected:ExecutionOutcome=complete(&mut session,Completion::Word {request:wrong,value:255});'
        body += 'let same:bool=same_request(expected,session.transfer.request);let pending:bool=session.state==State::AwaitingRead && same && session.accepted==0 && session.transfer.received==0 && session.issued==1 && session.runtime.steps==steps && session.runtime.frames[0].pc==pc && session.runtime.frames[0].operations[0].count==operands;'
        body += 'let accepted:ExecutionOutcome=complete(&mut session,Completion::Word {request:expected,value:42});'
        check = 'rejected==ExecutionOutcome::InvalidState && pending && accepted==ExecutionOutcome::Success && session.accepted==1 && session.transfer.words[0]==42 && session.transfer.received==1 && session.transfer.request.serial==2 && session.runtime.steps==steps'
        add('mismatch_' + coordinate, body, check, 'pending', '!pending', 'A mismatched completion cannot consume or advance the exact outstanding request')

    body = start(table(b'\xa4FLD_', bits=16))
    body += 'let status:Progress=advance(&mut session,64);let first:NativeRead=session.transfer.request;let steps:u64=session.runtime.steps;'
    body += 'let one:ExecutionOutcome=complete(&mut session,Completion::Word {request:first,value:42});let second:NativeRead=session.transfer.request;'
    body += 'let duplicate:ExecutionOutcome=complete(&mut session,Completion::Word {request:first,value:99});let same:bool=same_request(second,session.transfer.request);'
    body += 'let two:ExecutionOutcome=complete(&mut session,Completion::Word {request:second,value:1});let finished:Progress=advance(&mut session,64);let late:ExecutionOutcome=complete(&mut session,Completion::Word {request:second,value:99});let result:bool=fr_result(&session,298);'
    add('duplicate_after_accepted', body, 'one==ExecutionOutcome::Success && duplicate==ExecutionOutcome::InvalidState && same && two==ExecutionOutcome::Success && late==ExecutionOutcome::InvalidState && result && session.accepted==2 && session.transfer.words[0]==42 && session.transfer.words[1]==1 && steps<=1024 && session.runtime.steps==steps+2',
        'same', '!same', 'A stale previous request cannot replace an accepted word or duplicate the decoder charge')

    body = start(table(b'\xa4FLD_'))
    body += 'let status:Progress=advance(&mut session,64);let request:NativeRead=session.transfer.request;let steps:u64=session.runtime.steps;'
    body += 'let rejected:ExecutionOutcome=complete(&mut session,Completion::Failure {request:request,outcome:ExecutionOutcome::Success});let same:bool=same_request(request,session.transfer.request);'
    add('failure_cannot_report_success', body, 'rejected==ExecutionOutcome::InvalidState && same && session.state==State::AwaitingRead && session.accepted==0 && session.transfer.received==0 && session.runtime.steps==steps',
        'same', '!same', 'An invalid success-valued failure does not consume the pending response')

    body = start(table(b'\xa4FLD_'))
    body += 'let before:u64=session.runtime.steps;let empty:Progress=advance(&mut session,0);let zero:bool=session.runtime.steps==before && session.issued==0;'
    body += f'let high:Progress=advance(&mut session,{MAX});let untouched:bool=session.state==State::Running && session.runtime.steps==before && session.issued==0;'
    body += 'let pending:Progress=advance(&mut session,64);let request:NativeRead=session.transfer.request;let steps:u64=session.runtime.steps;'
    body += 'let polled:Progress=advance(&mut session,64);let same:bool=same_request(request,session.transfer.request);'
    body += 'let direct:Progress=progress(&session);let polled_read:bool=fr_progress_read(polled,request);let direct_read:bool=fr_progress_read(direct,request);let high_error:bool=fr_progress_error(high,ExecutionOutcome::Capacity);'
    add('poll_and_budget', body, 'zero && untouched && same && polled_read && direct_read && high_error && session.runtime.steps==steps && session.issued==1 && session.accepted==0',
        'untouched', '!untouched', 'Polling and rejected quantum bounds preserve durable state and do not restart execution')

    data = table(b'\x70\x0a\x2aKEEP' + package([0xA2], b'FLD_\xa3'), extra=keep)
    for fuel in (32, 1024):
        body = start(data, fuel=fuel) + array('memory', memory, 512)
        body += f'let mut trace:Trace;let outcome:ExecutionOutcome=fr_run(&mut session,&memory,&mut trace,4096,{MAX},2);'
        body += 'let kept:bool=fr_integer(session.program.store.space.objects[2].value,42);let accepted:u64=session.accepted;let issued:u64=session.issued;let again:Progress=advance(&mut session,64);'
        add(f'cumulative_fuel_{fuel}', body, f'session.state==State::Failed && outcome==ExecutionOutcome::WorkLimit && session.runtime.steps=={fuel} && kept && session.accepted==accepted && session.issued==issued && trace.count==accepted && accepted>1',
            f'session.runtime.steps=={fuel}', f'session.runtime.steps=={fuel-1}', 'Resumption cannot reset caller-selected cumulative method fuel; completed earlier stores remain visible')

    body = 'let session:Session;let result:Progress=progress(&session);let rejected:bool=fr_progress_error(result,ExecutionOutcome::InvalidState);'
    add('default_failure', body, 'rejected && session.state==State::Failed && session.issued==0', 'rejected', '!rejected',
        'Default initialization cannot advertise a successful execution or a successful failure outcome')
    for kind in ('RefOf', 'Index'):
        data = table(b'\xa4KEEP', extra=keep)
        body = start(data, setup=f'program.store.space.objects[2].value=Value::Reference {{kind:ReferenceKind::{kind},object_id:1}};')
        body += f'let result:Progress=advance(&mut session,64);let reference:bool=fr_reference_result(&session,ReferenceKind::{kind});'
        add('explicit_carrier_' + kind.lower(), body,
            'session.state==State::Failed && session.outcome==ExecutionOutcome::UnsupportedValue && !session.result.has_value && reference && session.issued==0 && session.accepted==0',
            'reference', '!reference', 'Explicit reference carriers remain nonreading; the established Program result graph rejects their FieldUnit referent')

    # The same public Program quotas apply after resumable execution. Actual
    # graph insufficiency preserves successful reads/stores, but exposes no root.
    quota_table = table(b'\x70\x0a\x2aKEEP\xa4FLD_', bits=65, extra=keep)
    for name, quotas, error in [('objects', (0, 16384), 'WorkLimit'),
                                ('bytes', (64, 8), 'Capacity')]:
        body = start(quota_table, quotas=quotas) + array('memory', memory, 512)
        body += f'let mut trace:Trace;let outcome:ExecutionOutcome=fr_run(&mut session,&memory,&mut trace,4096,{MAX},0);'
        body += 'let kept:bool=fr_integer(session.program.store.space.objects[2].value,42);let log:bool=fr_trace(&trace,&memory,9,9,0,1,0,1,0,0,512,true);'
        body += 'let prior:u64=session.runtime.steps;let issued:u64=session.issued;let again:Progress=advance(&mut session,64);'
        check = f'session.state==State::Failed && outcome==ExecutionOutcome::{error} && session.result.outcome==ExecutionOutcome::{error} && !session.result.has_value && session.program.store.space.object_count==5 && kept && log && session.accepted==9 && session.runtime.steps==prior && session.issued==issued'
        add('result_quota_' + name, body, check, 'log', '!log',
            'Result graph quota failure retains completed Buffer allocation, prior store effects and exact read observations without publishing a result')

    for name, quotas in [('objects', (65, 16384)), ('bytes', (64, 16385))]:
        body = start(table(b'\xa4FLD_', flags=1), flags=1, quotas=quotas,
                     arguments='arguments[0]=Value::Reference {kind:ReferenceKind::RefOf,object_id:1};')
        body += 'let status:Progress=advance(&mut session,64);'
        add('invalid_result_quota_' + name, body,
            'session.state==State::Failed && session.outcome==ExecutionOutcome::Capacity && session.issued==0 && session.accepted==0 && session.runtime.steps==0 && session.program.store.space.object_count==3',
            'session.program.store.space.object_count==3', 'session.program.store.space.object_count==4',
            'Invalid result quota bounds reject before external reference allocation or any read')

    body = start(table(b'\xa4FLD_'), quotas=(0, 0)) + array('memory', memory, 512)
    body += f'let mut trace:Trace;let outcome:ExecutionOutcome=fr_run(&mut session,&memory,&mut trace,64,{MAX},0);let result:bool=fr_result(&session,19);'
    add('scalar_zero_result_quota', body,
        'session.state==State::Finished && outcome==ExecutionOutcome::Success && result && session.accepted==1 && session.program.store.space.object_count==3',
        'result', '!result', 'Inline Integer results need no object or byte graph quota')

    for name, setup in [('installed', ''), ('malformed', 'let changed:Value=fr_malformed(program.store.space.objects[1].value,5);program.store.space.objects[1].value=changed;')]:
        body = start(table(b'\xa4\x8eFLD_'), setup=setup)
        body += 'let status:Progress=advance(&mut session,64);let result:bool=fr_result(&session,5);let none:bool=fr_no_deferred(session.runtime.frames[0].deferred_read);let intact:bool=fr_field(session.program.store.space.objects[1].value,original_field);'
        add('object_type_' + name, body,
            'session.state==State::Finished && result && none && intact && session.runtime.frames[0].field_reads && session.issued==0 && session.accepted==0',
            'result', '!result', 'Upstream ObjectType bypasses ordinary Field evaluation even in a read-enabled Session and with mismatched inert binding metadata')

    data = table(b'\xa4FLD_')
    body = array('input', data) + f'let prepared:Prepared=prepare_program(input,{len(data)},7,64,64);let mut program:Program=prepared.program;'
    body += 'let mut path:Path=Path {absolute:true,count:1};path.segments[0]=1313423693;let arguments:[Value;7];let before:Value=program.store.space.objects[1].value;let result:ExecutionResult=run_program(&mut program,path,&arguments,0,IntegerSize::EightBytes,64);let intact:bool=fr_field(program.store.space.objects[1].value,before);'
    add('legacy_no_provider', body, 'result.outcome==ExecutionOutcome::UnresolvedRegion && !result.has_value && intact && program.store.space.object_count==3',
        'result.outcome==ExecutionOutcome::UnresolvedRegion', 'result.outcome==ExecutionOutcome::Success', 'Legacy run_program retains its no-provider failure and field identity')

    assert len({row['name'] for row in rows}) == len(rows)
    return rows


def render(row, control=False, entry=None):
    check = row['check'].replace(*row['mutation']) if control else row['check']
    head = 'test_result()' if entry is None else entry + '(&mut self)'
    return f'machine {head}->i32 {{\n{row["body"]}\ntransition {check} {{true -> (0) _ -> (1)}}\n}}\n'
