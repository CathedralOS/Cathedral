"""Failure preserves instruction cursor; caches are not claimed transactional."""
NAMES=['decoder_constant_full','decoder_local_full','decoder_operation_full','decoder_named_value_full','decoder_named_method_full','decoder_target_full']
SOURCE='''
use aml::model::ObjectStore;
use aml::model::NamespaceResult;
use aml::namespace::insert;
use aml::namespace::empty;
use execution::execution_model::Frame;
use execution::execution_model::Binding;
use execution::execution_model::Operation;
use execution::decode_execution::decode_term;
use execution::decode_execution::decode_target;
machine ge_decoder(mode:u64,expected_pc:u64)->i32 {
 let mut input:[u8;1024];let mut store:ObjectStore=ObjectStore {};store.space=empty();
 let mut frame:Frame=Frame {scope:Path {absolute:true},source_length:4,body:Span {unit:7},end:4,operation_count:1};
 frame.operations[0]=Operation {opcode:0x72,value_arity:1,count:1};
 let expected:ExecutionOutcome=ge_decoder_setup(&mut input,&mut store,&mut frame,mode);
 let count:u64=frame.operation_count;
 let actual:ExecutionOutcome=ge_decoder_run(&input,&store,&mut frame,mode);
 let good:bool=ge_decoder_result(actual,expected,frame.pc,expected_pc,frame.end,frame.operation_count,count);
 transition good {true -> (0) _ -> (1)}
}
machine ge_decoder_result(actual:ExecutionOutcome,expected:ExecutionOutcome,pc:u64,want:u64,end:u64,count:u64,prior:u64)->bool {actual==expected && pc==want && end==4 && count==prior}
machine ge_decoder_run(input:&[u8;1024],store:&ObjectStore,frame:&mut Frame,mode:u64)->ExecutionOutcome {
 transition mode==5 {true -> target(input,store,frame) _ -> term(input,store,frame)}
 state target(input:&[u8;1024],store:&ObjectStore,frame:&mut Frame)->ExecutionOutcome {let result:ExecutionOutcome=decode_target(input,store,frame);result}
 state term(input:&[u8;1024],store:&ObjectStore,frame:&mut Frame)->ExecutionOutcome {let result:ExecutionOutcome=decode_term(input,store,frame);result}
}
machine ge_decoder_setup(input:&mut [u8;1024],store:&mut ObjectStore,frame:&mut Frame,mode:u64)->ExecutionOutcome {
 transition mode {0 -> (ExecutionOutcome::InvalidState) 1 -> local(input,frame) 2 -> operation(input,frame) 3 -> named(input,store,frame,false) 4 -> named(input,store,frame,true) _ -> target(frame)}
 state local(input:&mut [u8;1024],frame:&mut Frame)->ExecutionOutcome {input[0]=0x60;frame.locals[0]=Binding::Integer {number:11};ExecutionOutcome::InvalidState}
 state operation(input:&mut [u8;1024],frame:&mut Frame)->ExecutionOutcome {input[0]=0x72;frame.operation_count=16;ExecutionOutcome::Depth}
 state named(input:&mut [u8;1024],store:&mut ObjectStore,frame:&mut Frame,method:bool)->ExecutionOutcome {
  input[0]=88;input[1]=95;input[2]=95;input[3]=95;
  let mut path:Path=Path {absolute:true,count:1};path.segments[0]=1600085848;
  let value:Value=ge_decoder_named_value(method);
  let inserted:NamespaceResult=insert(store.space,path,value);store.space=inserted.space;
  transition method {true -> method_frame(frame) _ -> (ExecutionOutcome::InvalidState)}
 }
 state method_frame(frame:&mut Frame)->ExecutionOutcome {frame.operation_count=16;ExecutionOutcome::Depth}
 state target(frame:&mut Frame)->ExecutionOutcome {frame.operations[0]=Operation {opcode:0x72};ExecutionOutcome::InvalidState}
}
machine ge_decoder_named_value(method:bool)->Value {transition method {true -> (Value::Method {body:Span {unit:7}}) _ -> (Value::Integer {number:11})}}
'''
def rows():return [dict(name=name,direct=index)for index,name in enumerate(NAMES)]
def render(row,control):
 key=row['name']+('_control'if control else'_positive')
 return f'machine Suite::{key}(&mut self)->i32 {{let result:i32=ge_decoder({row["direct"]},{1 if control else 0});result}}\n','Suite::'+key+'='+str(int(control))
