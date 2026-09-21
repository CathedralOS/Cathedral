"""Whole-store/frame witnesses for suspension and target retirement ordering."""
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
COMPARATOR = HERE.parent/'generic-execution/focused/bridge-atomicity/main.omg'

def pending(object=0, source=42, result=42, stored=True, second=False, target='Target::Null', value=0):
    return f'expected_frame.deferred_write=DeferredWrite::Field {{object:{object},source:Operand::Integer {{number:{source}}},result:Operand::Integer {{number:{result}}},stored:{str(stored).lower()},has_second:{str(second).lower()},second_target:{target},second_value:{value}}};'

def contributed(number):
    return f'expected_frame.deferred_write=DeferredWrite::None;expected_frame.operations[0].count=1;expected_frame.operations[0].operands[0]=Operand::Integer {{number:{number}}};'

def cases():
    rows=[]
    retire='retire_operation(&input,&mut store,&mut frame,operation)'
    complete=lambda n:f'complete_field_write(&input,&mut store,&mut frame,Operand::Integer {{number:{n}}})'
    for bits in (32,64):
        def add(name,steps,setup='',operation=''):
            rows.append(dict(name=f'{name}_{bits}',bits=bits,steps=steps,setup=setup,operation=operation))
        add('store_pending',[(retire,'Success',pending())])
        add('store_converted_result',[(retire,'Success',pending()),(complete(7),'Success',contributed(7)),(complete(7),'InvalidState','')])
        add('store_bad_completion',[(retire,'Success',pending()),('complete_field_write(&input,&mut store,&mut frame,Operand::Uninitialized)','Uninitialized','')])
        add('legacy_field',[(retire,'UnresolvedRegion','')],setup='frame.field_writes=false;')
        add('parent_full',[(retire,'InvalidState','')],setup='frame.operations[0].count=1;')
        add('arg_field',[(retire,'Success',pending()),(complete(8),'Success',contributed(8))],setup='frame.arguments[0]=Binding::Shared {shared_id:3};',operation='operation.first_target=Target::Argument {argument_index:0};')
        add('default_completion',[(complete(9),'InvalidState','')])
        add('ordinary_source_identity',[(retire,'Success','expected_frame.locals[0]=Binding::Integer {number:999};expected_frame.operations[0].count=1;expected_frame.operations[0].operands[0]=Operand::Object {object_id:2};')],operation='operation.operands[0]=Operand::Object {object_id:2};operation.first_target=Target::Local {local_index:0};')
        divide='operation.opcode=0x78;operation.value_arity=2;operation.target_arity=2;operation.operands[0]=Operand::Integer {number:10};operation.operands[1]=Operand::Integer {number:3};'
        second='Target::Named {object_id:1}'
        first_pending=pending(source=1,result=3,stored=False,second=True,target=second,value=3)
        second_pending=pending(object=1,source=3,result=3,stored=False)
        add('divide_both',[(retire,'Success',first_pending),(complete(99),'Success',second_pending),(complete(88),'Success',contributed(3)),(complete(88),'InvalidState','')],operation=divide+f'operation.second_target={second};')
        second='Target::Local {local_index:0}'
        add('divide_first_field',[(retire,'Success',pending(source=1,result=3,stored=False,second=True,target=second,value=3)),(complete(88),'Success',contributed(3)+'expected_frame.locals[0]=Binding::Integer {number:3};')],operation=divide+f'operation.second_target={second};')
        add('divide_second_field',[(retire,'Success','expected_frame.locals[0]=Binding::Integer {number:1};'+second_pending),(complete(88),'Success',contributed(3))],operation=divide+'operation.first_target=Target::Local {local_index:0};operation.second_target=Target::Named {object_id:1};')
        second='Target::Named {object_id:64}'
        add('divide_late_failure',[(retire,'Success',pending(source=1,result=3,stored=False,second=True,target=second,value=3)),(complete(88),'MissingObject','expected_frame.deferred_write=DeferredWrite::None;')],operation=divide+f'operation.second_target={second};')
        add('scalar_result',[(retire,'Success',pending(source=9,result=9,stored=False)),(complete(2),'Success',contributed(9))],operation='operation.opcode=0x72;operation.value_arity=2;operation.operands[0]=Operand::Integer {number:4};operation.operands[1]=Operand::Integer {number:5};')
        runtime_complete=lambda n:f'complete_retired_write(&input,&mut store,&mut runtime,Operand::Integer {{number:{n}}})'
        advance='advance_write_runtime(&input,&mut store,&definitions,&mut runtime)'
        second='Target::Named {object_id:64}'
        add('runtime_late_failure',[(retire,'Success',pending(source=1,result=3,stored=False,second=True,target=second,value=3)),(runtime_complete(88),'MissingObject','expected_frame.deferred_write=DeferredWrite::None;expected_runtime.outcome=ExecutionOutcome::MissingObject;'),(advance,'MissingObject','')],operation=divide+f'operation.second_target={second};')
        rows[-1]['runtime']=True
        add('runtime_retry',[(retire,'Success',pending()),('complete_retired_write(&input,&mut store,&mut runtime,Operand::Uninitialized)','Uninitialized',''),(runtime_complete(7),'Success',contributed(7)),(runtime_complete(7),'InvalidState','')])
        rows[-1]['runtime']=True
        add('runtime_pending_poll',[(retire,'Success',pending()),(advance,'Success','')])
        rows[-1]['runtime']=True
        second='Target::Named {object_id:1}'
        add('runtime_second_pending',[(retire,'Success',pending(source=1,result=3,stored=False,second=True,target=second,value=3)),(runtime_complete(88),'Success',second_pending),(advance,'Success',''),(runtime_complete(99),'Success',contributed(3))],operation=divide+f'operation.second_target={second};')
        rows[-1]['runtime']=True
    return rows

DEFERRED = '''
machine fw_deferred(a:DeferredWrite,b:DeferredWrite)->bool {
 transition a {DeferredWrite::None -> none(b)
  DeferredWrite::Field {object,source,result,stored,has_second,second_target,second_value} -> field(b,object,source,result,stored,has_second,second_target,second_value)}
 state none(b:DeferredWrite)->bool {transition b {DeferredWrite::None -> (true) _ -> (false)}}
 state field(b:DeferredWrite,id:u64,source:Operand,result:Operand,stored:bool,has_second:bool,second_target:Target,second_value:u64)->bool {
  transition b {DeferredWrite::Field {object,source as other_source,result as other_result,stored as other_stored,has_second as other_second,second_target as other_target,second_value as other_value} -> same(id,source,result,stored,has_second,second_target,second_value,object,other_source,other_result,other_stored,other_second,other_target,other_value) _ -> (false)}
 }
 state same(id:u64,source:Operand,result:Operand,stored:bool,has_second:bool,target:Target,value:u64,other_id:u64,other_source:Operand,other_result:Operand,other_stored:bool,other_second:bool,other_target:Target,other_value:u64)->bool {
  let source_same:bool=ba_operand(source,other_source);let result_same:bool=ba_operand(result,other_result);let target_same:bool=ba_target(target,other_target);
  id==other_id && stored==other_stored && has_second==other_second && value==other_value && source_same && result_same && target_same
 }
}
'''

def render(match=''):
    rows=[r for r in cases() if not match or r['name'] in match.split(',')]
    assert rows
    source=COMPARATOR.read_text().split('machine ba_check(')[0]
    source=source.replace(' let arrays:bool=ba_frame_arrays(a,b,0,16,true);',' let arrays:bool=ba_frame_arrays(a,b,0,16,true);let pending:bool=fw_deferred(a.deferred_write,b.deferred_write);')
    source=source.replace(' scope && body && result && arrays &&',' pending && a.field_writes==b.field_writes && scope && body && result && arrays &&')
    source+='\nuse execution::execution_model::DeferredWrite;\nuse execution::retire::retire_operation;\nuse execution::write_retirement::complete_field_write;\n'+DEFERRED
    source+=RUNTIME
    names=[]
    for row in rows:
        for control in (False,True):
            label='Suite::'+row['name']+('_control' if control else '_positive')
            names.append(label+'='+str(int(control)))
            source+=f'''\nmachine {label}(&mut self)->i32 {{
 let input:[u8;1024];let mut store:ObjectStore=ObjectStore {{}};store.space.object_count=4;
 store.space.objects[0].value=Value::FieldUnit {{}};store.space.objects[1].value=Value::FieldUnit {{}};
 store.space.objects[2].value=Value::Integer {{number:999}};store.space.objects[3].value=Value::Reference {{kind:ReferenceKind::RefOf,object_id:0}};
 store.bytes.blocks[63].bytes[255]=77;
 let mut frame:Frame=Frame {{field_writes:true,size:IntegerSize::{'FourBytes' if row['bits']==32 else 'EightBytes'},operation_count:1}};
 frame.operations[0]=Operation {{opcode:0xa4,value_arity:1}};frame.lookup_cache[15].object_id=71;frame.operations[15].operands[6]=Operand::Integer {{number:59}};
 {row['setup']}
 let mut operation:Operation=Operation {{opcode:0x70,value_arity:1,target_arity:1,first_target:Target::Named {{object_id:0}}}};
 operation.operands[0]=Operand::Integer {{number:42}};{row['operation']}
 let expected_store:ObjectStore=store;
'''
            runtime=row.get('runtime',False)
            if runtime:
                source+=' let definitions:[MethodDefinition;64];let mut runtime:Runtime=Runtime {count:1,steps:9};runtime.frames[0]=frame;let mut expected_runtime:Runtime=runtime;\n'
            else:
                source+=' let mut expected_frame:Frame=frame;\n'
            checks=[]
            for i,(call,outcome,expected) in enumerate(row['steps']):
                if runtime:
                    call=call.replace('&mut frame','&mut runtime.frames[0]')
                    expected=expected.replace('expected_frame','expected_runtime.frames[0]')
                source+=f' let outcome_{i}:ExecutionOutcome={call};{expected}\n'
                if control and i==len(row['steps'])-1:
                    source+=(' expected_runtime.frames[0]' if runtime else ' expected_frame')+'.lookup_cache[15].object_id=72;\n'
                comparison='fw_runtime(&runtime,&expected_runtime)' if runtime else 'ba_frame(&frame,&expected_frame)'
                source+=f' let store_{i}:bool=fx_store(&store,&expected_store);let frame_{i}:bool={comparison};\n'
                checks.append(f'outcome_{i}==ExecutionOutcome::{outcome} && store_{i} && frame_{i}')
            source+=' transition '+ ' && '.join(checks)+' {true -> (0) _ -> (1)}\n}\n'
    return rows,source,names

RUNTIME = """
use execution::runtime_model::Runtime;
use aml::model::MethodDefinition;
use execution::engine::complete_retired_write;
use execution::engine::advance_write_runtime;
machine fw_runtime(a:&Runtime,b:&Runtime)->bool {
 let frames:bool=fw_runtime_frames(&a.frames,&b.frames,0,4,true);let result:bool=ba_operand(a.result,b.result);
 frames && result && a.outcome==b.outcome && a.count==b.count && a.unit==b.unit && a.length==b.length && a.steps==b.steps && a.finished==b.finished && a.has_result==b.has_result && a.fault_offset==b.fault_offset
}
machine fw_runtime_frames(a:&[Frame;4],b:&[Frame;4],i:u64,n:u64,prior:bool)
terminates by(i,n)->Nat::BoundedDistance;
->bool {let good:bool=fw_runtime_frame(a,b,i,n);transition i<n {true -> fw_runtime_frames(a,b,i+1,n,prior && good) _ -> (prior)}}
machine fw_runtime_frame(a:&[Frame;4],b:&[Frame;4],i:u64,n:u64)->bool {
 transition i<n && i<4 {true -> selected(a.frames[i],b.frames[i]) _ -> (true)}
 state selected(a:Frame,b:Frame)->bool {let same:bool=ba_frame(&a,&b);same}
}
""".replace('a.frames[i]','a[i]').replace('b.frames[i]','b[i]')
